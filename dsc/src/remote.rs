// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Run configuration operations on another machine.
//!
//! The remote endpoint is a `dsc server` process that the target's SSH daemon starts as the
//! `dsc` subsystem. This module reads the input locally, starts the system `ssh` client, speaks
//! MCP to the server over the client's stdin and stdout, and prints the result the same way a
//! local run does.

use crate::args::{ConfigSubCommand, OutputFormat, SubCommand};
use crate::server::invoke_dsc_config::EXIT_CODE_DATA_KEY;
use crate::server::mcp_server::SERVER_NAME;
use crate::util::{
    get_input, write_object, EXIT_DSC_ERROR, EXIT_INVALID_ARGS, EXIT_INVALID_INPUT,
    EXIT_JSON_ERROR, EXIT_REMOTE_FAILED,
};
use dsc_lib::{dscresources::command_resource::TraceLevel, types::SemanticVersion};
use rmcp::{
    model::{
        CallToolRequestParams, CallToolResult, ClientCapabilities, ClientConfig, Implementation,
        ServerPeerInfo,
    },
    service::RoleClient,
    transport::async_rw::AsyncRwTransport,
    ServiceError, ServiceExt,
};
use rust_i18n::t;
use serde::Deserialize;
use serde_json::{Map, Value};
use std::{
    ffi::OsString,
    fmt::{self, Display, Formatter},
    process::{ExitCode, Stdio},
    str::FromStr,
    sync::{Arc, Mutex, OnceLock},
    time::Duration,
};
use thiserror::Error;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::{Child, ChildStderr, Command},
    task::JoinHandle,
    time::timeout,
};
use tracing::{debug, error, info, trace, warn};
use url::{Host, Url};

/// The only URI scheme a target can use in this version.
const SSH_SCHEME: &str = "ssh";
/// The SSH subsystem that starts `dsc server` on the target.
const SUBSYSTEM_NAME: &str = "dsc";
/// The server tool that runs a configuration operation.
const CONFIG_TOOL_NAME: &str = "invoke_dsc_config";
/// The resource type whose file paths resolve on the target instead of on the client.
const INCLUDE_RESOURCE_TYPE: &str = "Microsoft.DSC/Include";
/// The first version of DSC whose server reports its identity in the handshake. Older servers
/// silently ignore arguments they don't know, so `--what-if` would apply changes for real.
const MINIMUM_SERVER_VERSION: &str = "3.4.0-preview.3";
/// The environment variable that replaces the `ssh` executable found on `PATH`.
const SSH_EXECUTABLE_ENV_VAR: &str = "DSC_SSH_EXECUTABLE";
/// How long the target has to answer the MCP `initialize` request after `ssh` is started.
const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(60);
/// How long `ssh` has to exit after the connection is closed before it's terminated.
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(10);
/// The largest message `dsc server` reads. It drops a longer one without answering.
const MAX_REQUEST_BYTES: usize = 16 * 1024 * 1024;
/// The largest response the client accepts.
const MAX_RESPONSE_BYTES: usize = 64 * 1024 * 1024;
/// How many lines of `ssh`'s own stderr output are kept for a connection error message.
const MAX_SSH_STDERR_LINES: usize = 20;

/// A machine that runs `dsc server` as an SSH subsystem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteTarget {
    user: Option<String>,
    host: String,
    port: Option<u16>,
}

/// The reasons a `--target` value is rejected.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum RemoteTargetError {
    #[error("{t}: {0}", t = t!("remote.targetNotUri"))]
    NotUri(String),
    #[error("{t}", t = t!("remote.targetUnsupportedScheme", scheme = .0, supported = SSH_SCHEME))]
    UnsupportedScheme(String),
    #[error("{t}", t = t!("remote.targetMissingHost"))]
    MissingHost,
    #[error("{t}", t = t!("remote.targetInvalidHost", host = .0))]
    InvalidHost(String),
    #[error("{t}", t = t!("remote.targetInvalidUser", user = .0))]
    InvalidUser(String),
    #[error("{t}", t = t!("remote.targetPasswordNotAllowed"))]
    PasswordNotAllowed,
    #[error("{t}", t = t!("remote.targetUnexpectedComponent"))]
    UnexpectedComponent,
}

impl FromStr for RemoteTarget {
    type Err = RemoteTargetError;

    /// Parse a target of the form `ssh://[<user>@]<host>[:<port>]`.
    ///
    /// # Errors
    ///
    /// Returns an error when the value isn't a URI, uses another scheme, carries a password,
    /// path, query or fragment, or names a user or host that could be read as an `ssh` option.
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let url = Url::parse(value).map_err(|err| RemoteTargetError::NotUri(err.to_string()))?;
        if url.scheme() != SSH_SCHEME {
            return Err(RemoteTargetError::UnsupportedScheme(url.scheme().to_string()));
        }
        if url.password().is_some() {
            return Err(RemoteTargetError::PasswordNotAllowed);
        }
        if !matches!(url.path(), "" | "/") || url.query().is_some() || url.fragment().is_some() {
            return Err(RemoteTargetError::UnexpectedComponent);
        }

        let host = match url.host() {
            Some(Host::Domain(name)) => {
                if !is_valid_host_name(name) {
                    return Err(RemoteTargetError::InvalidHost(name.to_string()));
                }
                name.to_string()
            },
            Some(Host::Ipv4(address)) => address.to_string(),
            // `ssh` takes an IPv6 address without the brackets a URI requires.
            Some(Host::Ipv6(address)) => address.to_string(),
            None => return Err(RemoteTargetError::MissingHost),
        };

        let user = match url.username() {
            "" => None,
            encoded => {
                let Ok(user) = urlencoding::decode(encoded) else {
                    return Err(RemoteTargetError::InvalidUser(encoded.to_string()));
                };
                if !is_valid_user_name(&user) {
                    return Err(RemoteTargetError::InvalidUser(user.into_owned()));
                }
                Some(user.into_owned())
            },
        };

        Ok(Self { user, host, port: url.port() })
    }
}

impl Display for RemoteTarget {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{SSH_SCHEME}://")?;
        if let Some(user) = &self.user {
            write!(f, "{user}@")?;
        }
        if self.host.contains(':') {
            write!(f, "[{}]", self.host)?;
        } else {
            write!(f, "{}", self.host)?;
        }
        if let Some(port) = self.port {
            write!(f, ":{port}")?;
        }
        Ok(())
    }
}

impl RemoteTarget {
    /// Build the arguments for the `ssh` client.
    ///
    /// The host always follows `--`, so a target can never add an option to the command line.
    ///
    /// # Returns
    ///
    /// The arguments that request the `dsc` subsystem on this target.
    fn ssh_args(&self) -> Vec<String> {
        let mut args = vec!["-s".to_string()];
        if let Some(port) = self.port {
            args.push("-p".to_string());
            args.push(port.to_string());
        }
        if let Some(user) = &self.user {
            args.push("-l".to_string());
            args.push(user.clone());
        }
        args.push("--".to_string());
        args.push(self.host.clone());
        args.push(SUBSYSTEM_NAME.to_string());
        args
    }
}

/// Check that a host is a DNS name or an SSH configuration alias.
///
/// # Arguments
///
/// * `name` - The host part of the target URI.
///
/// # Returns
///
/// `true` when the name starts with a letter, digit or underscore and contains only letters,
/// digits, `.`, `-` and `_`.
fn is_valid_host_name(name: &str) -> bool {
    let starts_safely = name.chars().next().is_some_and(|c| c.is_ascii_alphanumeric() || c == '_');
    starts_safely && name.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
}

/// Check that a user name can't be read as an `ssh` option.
///
/// # Arguments
///
/// * `name` - The decoded user part of the target URI.
///
/// # Returns
///
/// `true` when the name isn't empty, doesn't start with `-`, and has no whitespace or control
/// characters.
fn is_valid_user_name(name: &str) -> bool {
    !name.is_empty()
        && !name.starts_with('-')
        && !name.chars().any(|c| c.is_whitespace() || c.is_control())
}

/// Validate a `--target` value together with the command it's used with.
///
/// # Arguments
///
/// * `target` - The value of the `--target` option.
/// * `subcommand` - The subcommand the user invoked.
///
/// # Errors
///
/// Returns the invalid arguments exit code when the command can't run remotely or the target
/// isn't valid. The reason is logged as an error.
pub fn resolve_target(target: &str, subcommand: &SubCommand) -> Result<RemoteTarget, ExitCode> {
    if !supports_remote(subcommand) {
        error!("{}", t!("remote.unsupportedCommand"));
        return Err(ExitCode::from(EXIT_INVALID_ARGS));
    }

    match target.parse::<RemoteTarget>() {
        Ok(target) => Ok(target),
        Err(err) => {
            error!("{}", t!("remote.invalidTarget", target = target, error = err));
            Err(ExitCode::from(EXIT_INVALID_ARGS))
        }
    }
}

/// Determine whether a command can run against a remote target.
///
/// Only `config get`, `set`, `test` and `export` are supported, and only without the options
/// that depend on the local machine or that DSC uses internally.
///
/// # Arguments
///
/// * `subcommand` - The subcommand the user invoked.
///
/// # Returns
///
/// `true` when the command can be sent to a remote server.
fn supports_remote(subcommand: &SubCommand) -> bool {
    let SubCommand::Config { subcommand, system_root, as_group, as_assert, as_include, .. } = subcommand else {
        return false;
    };
    if system_root.is_some() || *as_group || *as_assert || *as_include {
        return false;
    }

    match subcommand {
        ConfigSubCommand::Get { .. } | ConfigSubCommand::Set { .. } | ConfigSubCommand::Export { .. } => true,
        ConfigSubCommand::Test { as_get, as_config, .. } => !*as_get && !*as_config,
        ConfigSubCommand::Validate { .. } | ConfigSubCommand::Resolve { .. } => false,
    }
}

/// A configuration operation the server can run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RemoteOperation {
    Get,
    Set,
    Test,
    Export,
}

impl RemoteOperation {
    /// The value of the tool's `operation` argument.
    fn as_str(self) -> &'static str {
        match self {
            RemoteOperation::Get => "get",
            RemoteOperation::Set => "set",
            RemoteOperation::Test => "test",
            RemoteOperation::Export => "export",
        }
    }
}

/// A remote operation that didn't produce a result.
#[derive(Debug, PartialEq, Eq)]
struct RemoteFailure {
    /// The exit code `dsc` returns for the failure.
    exit_code: u8,
    /// The message logged for the failure.
    message: String,
}

/// Run a `dsc config` operation on a remote target and print its result.
///
/// The document and parameters are read on this machine, exactly as for a local run, and sent
/// inline. Nothing has to exist on the target except `dsc`.
///
/// # Arguments
///
/// * `target` - The machine to run the operation on.
/// * `subcommand` - The `config` subcommand with its input and output options.
/// * `parameters` - The merged parameters from `--parameters` and `--parameters-file`.
///
/// # Errors
///
/// Returns the exit code for the failure: the usual codes for input errors and for failures the
/// target reports, and the remote failure code when the target can't be reached or is too old.
pub fn config(target: &RemoteTarget, subcommand: &ConfigSubCommand, parameters: Option<&str>) -> Result<(), ExitCode> {
    let (operation, input, file, output_format, what_if) = match subcommand {
        ConfigSubCommand::Get { input, file, output_format } => (RemoteOperation::Get, input, file, output_format, false),
        ConfigSubCommand::Set { input, file, output_format, what_if } => (RemoteOperation::Set, input, file, output_format, *what_if),
        ConfigSubCommand::Test { input, file, output_format, .. } => (RemoteOperation::Test, input, file, output_format, false),
        ConfigSubCommand::Export { input, file, output_format } => (RemoteOperation::Export, input, file, output_format, false),
        ConfigSubCommand::Validate { .. } | ConfigSubCommand::Resolve { .. } => {
            error!("{}", t!("remote.unsupportedCommand"));
            return Err(ExitCode::from(EXIT_INVALID_ARGS));
        }
    };

    let document = get_input(input.as_ref(), file.as_ref())?;
    if document.is_empty() {
        error!("{}", t!("remote.noDocument"));
        return Err(ExitCode::from(EXIT_INVALID_INPUT));
    }
    let parameters = match parameters {
        Some(parameters) => Some(normalize_parameters(parameters)?),
        None => None,
    };

    if serde_json::from_str::<Value>(&document).is_ok_and(|value| contains_include(&value)) {
        warn!("{}", t!("remote.includeResolvesOnTarget", resource = INCLUDE_RESOURCE_TYPE, target = target));
    }

    let arguments = build_arguments(operation, document, parameters, what_if);
    let request_size = serde_json::to_string(&arguments).map_or(0, |json| json.len());
    if request_size >= MAX_REQUEST_BYTES {
        error!("{}", t!("remote.requestTooLarge", size = request_size, limit = MAX_REQUEST_BYTES));
        return Err(ExitCode::from(EXIT_INVALID_INPUT));
    }

    let runtime = match tokio::runtime::Runtime::new() {
        Ok(runtime) => runtime,
        Err(err) => {
            error!("{}", t!("remote.failedToCreateRuntime", error = err));
            return Err(ExitCode::from(EXIT_DSC_ERROR));
        }
    };

    info!("{}", t!("remote.connecting", target = target));
    match runtime.block_on(invoke_config_tool(target, arguments)) {
        Ok(result) => write_result(operation, &result, output_format.as_ref()),
        Err(failure) => {
            error!("{}", failure.message);
            Err(ExitCode::from(failure.exit_code))
        }
    }
}

/// Convert parameters given as JSON or YAML into JSON text.
///
/// # Arguments
///
/// * `parameters` - The parameters as the user provided them.
///
/// # Errors
///
/// Returns the same exit codes a local run uses when the parameters can't be parsed.
fn normalize_parameters(parameters: &str) -> Result<String, ExitCode> {
    let value: Value = match serde_json::from_str(parameters) {
        Ok(json) => json,
        Err(_) => match serde_yaml::from_str::<serde_yaml::Value>(parameters) {
            Ok(yaml) => match serde_json::to_value(yaml) {
                Ok(json) => json,
                Err(err) => {
                    error!("{}: {err}", t!("subcommand.failedConvertJson"));
                    return Err(ExitCode::from(EXIT_DSC_ERROR));
                }
            },
            Err(err) => {
                error!("{}: {err}", t!("subcommand.invalidParameters"));
                return Err(ExitCode::from(EXIT_INVALID_INPUT));
            }
        },
    };

    Ok(value.to_string())
}

/// Find out whether a document uses the include resource anywhere, including in nested groups.
///
/// # Arguments
///
/// * `value` - The configuration document, or a part of it.
///
/// # Returns
///
/// `true` when any object in the document has the include resource's type.
fn contains_include(value: &Value) -> bool {
    match value {
        Value::Object(map) => {
            map.get("type")
                .and_then(Value::as_str)
                .is_some_and(|resource_type| resource_type.eq_ignore_ascii_case(INCLUDE_RESOURCE_TYPE))
                || map.values().any(contains_include)
        },
        Value::Array(items) => items.iter().any(contains_include),
        _ => false,
    }
}

/// Build the arguments for the configuration tool.
///
/// # Arguments
///
/// * `operation` - The operation to run.
/// * `document` - The configuration document as JSON text.
/// * `parameters` - The parameters as JSON text, when there are any.
/// * `what_if` - Whether a set should only be simulated.
///
/// # Returns
///
/// The tool arguments. `parameters` and `what_if` are left out when they aren't used.
fn build_arguments(operation: RemoteOperation, document: String, parameters: Option<String>, what_if: bool) -> Map<String, Value> {
    let mut arguments = Map::new();
    arguments.insert("operation".to_string(), Value::String(operation.as_str().to_string()));
    arguments.insert("configuration".to_string(), Value::String(document));
    if let Some(parameters) = parameters {
        arguments.insert("parameters".to_string(), Value::String(parameters));
    }
    if what_if {
        arguments.insert("what_if".to_string(), Value::Bool(true));
    }
    arguments
}

/// The `ssh` child process and the task that reads its stderr.
struct SshProcess {
    child: Child,
    stderr_task: JoinHandle<()>,
    /// Lines on stderr that aren't traces from the remote `dsc`, so they come from `ssh`.
    ssh_lines: Arc<Mutex<Vec<String>>>,
}

impl SshProcess {
    /// Start `ssh` for a target with its standard streams connected to pipes.
    ///
    /// # Arguments
    ///
    /// * `target` - The machine to connect to.
    ///
    /// # Errors
    ///
    /// Returns a remote failure when the `ssh` executable can't be started.
    fn spawn(target: &RemoteTarget) -> Result<(Self, tokio::process::ChildStdin, tokio::process::ChildStdout), RemoteFailure> {
        let executable = ssh_executable();
        let args = target.ssh_args();
        debug!("{}", t!("remote.startingSsh", executable = executable.to_string_lossy(), args = args.join(" ")));

        let mut command = Command::new(&executable);
        command
            .args(&args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        let mut child = command.spawn().map_err(|err| RemoteFailure {
            exit_code: EXIT_REMOTE_FAILED,
            message: t!("remote.failedToStartSsh", executable = executable.to_string_lossy(), error = err).to_string(),
        })?;

        let (Some(stdin), Some(stdout), Some(stderr)) = (child.stdin.take(), child.stdout.take(), child.stderr.take()) else {
            unreachable!("{}", t!("remote.missingSshPipes"));
        };

        let ssh_lines = Arc::new(Mutex::new(Vec::new()));
        let stderr_task = tokio::spawn(forward_stderr(stderr, target.host.clone(), Arc::clone(&ssh_lines)));
        Ok((Self { child, stderr_task, ssh_lines }, stdin, stdout))
    }

    /// Wait for `ssh` to exit and collect what it wrote on stderr itself.
    ///
    /// # Arguments
    ///
    /// * `terminate` - Whether to end the process immediately instead of waiting for it to exit
    ///   after its stdin was closed.
    ///
    /// # Returns
    ///
    /// The stderr lines that weren't traces from the remote `dsc`.
    async fn finish(mut self, terminate: bool) -> Vec<String> {
        if terminate || timeout(SHUTDOWN_TIMEOUT, self.child.wait()).await.is_err() {
            let _ = self.child.kill().await;
        }
        // A process further down the chain can keep the pipe open, so don't wait forever.
        if timeout(SHUTDOWN_TIMEOUT, &mut self.stderr_task).await.is_err() {
            self.stderr_task.abort();
        }
        self.ssh_lines.lock().map(|lines| lines.clone()).unwrap_or_default()
    }
}

/// Get the `ssh` executable to start.
///
/// # Returns
///
/// The value of the `DSC_SSH_EXECUTABLE` environment variable when it's set, otherwise `ssh`,
/// which is resolved from `PATH`.
fn ssh_executable() -> OsString {
    match std::env::var_os(SSH_EXECUTABLE_ENV_VAR) {
        Some(value) if !value.is_empty() => value,
        _ => OsString::from("ssh"),
    }
}

/// Read the `ssh` child's stderr until it closes.
///
/// Traces from the remote `dsc` are emitted through the local tracing configuration. Any other
/// line comes from `ssh` itself and is kept for a connection error message.
///
/// # Arguments
///
/// * `stderr` - The child's stderr pipe.
/// * `host` - The target host, used to attribute remote traces.
/// * `ssh_lines` - Receives the lines that aren't remote traces.
async fn forward_stderr(stderr: ChildStderr, host: String, ssh_lines: Arc<Mutex<Vec<String>>>) {
    let mut reader = BufReader::new(stderr);
    let mut buffer = Vec::new();
    loop {
        buffer.clear();
        match reader.read_until(b'\n', &mut buffer).await {
            Ok(0) | Err(_) => break,
            Ok(_) => {},
        }
        let text = String::from_utf8_lossy(&buffer);
        let line = text.trim();
        if line.is_empty() || log_remote_trace(&host, line) {
            continue;
        }

        debug!("{}", t!("remote.sshOutput", line = line));
        if let Ok(mut lines) = ssh_lines.lock() {
            if lines.len() == MAX_SSH_STDERR_LINES {
                lines.remove(0);
            }
            lines.push(line.to_string());
        }
    }
}

/// A trace line written by `dsc --trace-format json`.
#[derive(Deserialize)]
struct RemoteTrace {
    level: TraceLevel,
    fields: Map<String, Value>,
}

/// Emit a stderr line as a local trace when it's a trace from the remote `dsc`.
///
/// # Arguments
///
/// * `host` - The target host, prefixed to the message.
/// * `line` - A line from the `ssh` child's stderr.
///
/// # Returns
///
/// `true` when the line was a remote trace and has been emitted.
fn log_remote_trace(host: &str, line: &str) -> bool {
    let Ok(remote_trace) = serde_json::from_str::<RemoteTrace>(line) else {
        return false;
    };

    // Traces that DSC relays from a resource carry the text in `trace_message`.
    let text = ["message", "trace_message"]
        .iter()
        .find_map(|key| remote_trace.fields.get(*key).and_then(Value::as_str))
        .map_or_else(|| Value::Object(remote_trace.fields.clone()).to_string(), str::to_string);
    let message = t!("remote.trace", host = host, message = text);
    match remote_trace.level {
        TraceLevel::Error => error!("{message}"),
        TraceLevel::Warn => warn!("{message}"),
        TraceLevel::Info => info!("{message}"),
        TraceLevel::Debug => debug!("{message}"),
        TraceLevel::Trace => trace!("{message}"),
    }
    true
}

/// Get the lowest server version this client talks to.
///
/// # Panics
///
/// Panics if the built-in minimum version isn't a valid semantic version, which a unit test
/// rules out.
fn minimum_server_version() -> &'static SemanticVersion {
    static VERSION: OnceLock<SemanticVersion> = OnceLock::new();
    VERSION.get_or_init(|| match SemanticVersion::parse(MINIMUM_SERVER_VERSION) {
        Ok(version) => version,
        Err(err) => unreachable!("{}", t!("remote.invalidMinimumVersion", error = err)),
    })
}

/// Check that the peer is a DSC server this client can safely send operations to.
///
/// # Arguments
///
/// * `peer` - What the server reported about itself in the `initialize` response.
///
/// # Errors
///
/// Returns a description of the mismatch when the server isn't DSC or is older than the
/// minimum version.
fn check_server(peer: Option<&ServerPeerInfo>) -> Result<(), String> {
    let Some(server) = peer.and_then(|peer| peer.server_info.as_ref()) else {
        return Err(t!("remote.serverNotIdentified").to_string());
    };

    let minimum = minimum_server_version();
    if server.name != SERVER_NAME {
        return Err(t!("remote.serverTooOld", name = server.name, version = server.version, minimum = minimum).to_string());
    }
    match SemanticVersion::parse(&server.version) {
        Ok(version) if version >= *minimum => {
            debug!("{}", t!("remote.serverVersion", version = version));
            Ok(())
        },
        _ => Err(t!("remote.serverTooOld", name = server.name, version = server.version, minimum = minimum).to_string()),
    }
}

/// Describe a connection problem, including what `ssh` reported.
///
/// # Arguments
///
/// * `target` - The machine that couldn't be reached.
/// * `detail` - What the client observed.
/// * `ssh_lines` - What `ssh` wrote on stderr.
///
/// # Returns
///
/// A failure with the remote failure exit code.
fn connection_failure(target: &RemoteTarget, detail: &str, ssh_lines: &[String]) -> RemoteFailure {
    let message = if ssh_lines.is_empty() {
        t!("remote.connectionFailed", target = target, detail = detail)
    } else {
        t!("remote.connectionFailed", target = target, detail = ssh_lines.join("; "))
    };
    RemoteFailure { exit_code: EXIT_REMOTE_FAILED, message: message.to_string() }
}

/// Connect to a target, run the configuration tool once and disconnect.
///
/// # Arguments
///
/// * `target` - The machine to run the operation on.
/// * `arguments` - The arguments for the configuration tool.
///
/// # Errors
///
/// Returns a failure with the exit code to use when the connection, the handshake, the version
/// check or the operation fails.
async fn invoke_config_tool(target: &RemoteTarget, arguments: Map<String, Value>) -> Result<Value, RemoteFailure> {
    let (process, stdin, stdout) = SshProcess::spawn(target)?;

    let transport = AsyncRwTransport::<RoleClient, _, _>::new(stdout, stdin).with_max_line_length(MAX_RESPONSE_BYTES);
    let client = ClientConfig::new(
        ClientCapabilities::default(),
        Implementation::new(SERVER_NAME, env!("CARGO_PKG_VERSION")),
    );
    let service = match timeout(HANDSHAKE_TIMEOUT, client.serve(transport)).await {
        Ok(Ok(service)) => service,
        Ok(Err(err)) => {
            let ssh_lines = process.finish(true).await;
            return Err(connection_failure(target, &err.to_string(), &ssh_lines));
        },
        Err(_) => {
            let ssh_lines = process.finish(true).await;
            let detail = t!("remote.handshakeTimeout", seconds = HANDSHAKE_TIMEOUT.as_secs());
            return Err(connection_failure(target, &detail, &ssh_lines));
        },
    };

    if let Err(reason) = check_server(service.peer_info().as_deref()) {
        let _ = service.cancel().await;
        process.finish(false).await;
        return Err(RemoteFailure {
            exit_code: EXIT_REMOTE_FAILED,
            message: t!("remote.incompatibleServer", target = target, reason = reason).to_string(),
        });
    }

    let response = service.call_tool(CallToolRequestParams::new(CONFIG_TOOL_NAME).with_arguments(arguments)).await;
    let _ = service.cancel().await;
    let ssh_lines = process.finish(false).await;

    match response {
        Ok(result) => tool_result(result),
        Err(ServiceError::McpError(err)) => Err(RemoteFailure {
            exit_code: tool_error_exit_code(err.data.as_ref()),
            message: err.message.to_string(),
        }),
        Err(err) => Err(connection_failure(target, &err.to_string(), &ssh_lines)),
    }
}

/// Get the exit code for a tool error.
///
/// # Arguments
///
/// * `data` - The `data` member of the JSON-RPC error.
///
/// # Returns
///
/// The exit code the server put in the error data, or the general DSC error code when the data
/// has none.
fn tool_error_exit_code(data: Option<&Value>) -> u8 {
    data.and_then(|data| data.get(EXIT_CODE_DATA_KEY))
        .and_then(Value::as_u64)
        .and_then(|code| u8::try_from(code).ok())
        .filter(|code| *code != 0)
        .unwrap_or(EXIT_DSC_ERROR)
}

/// Extract the operation result from a tool response.
///
/// # Arguments
///
/// * `result` - The response to the tool call.
///
/// # Errors
///
/// Returns a failure when the tool flagged an error or the response has no structured result.
fn tool_result(result: CallToolResult) -> Result<Value, RemoteFailure> {
    if result.is_error == Some(true) {
        // Read the text blocks through their JSON form, which is stable across content types.
        let text = serde_json::to_value(&result.content)
            .ok()
            .and_then(|content| content.as_array().map(|blocks| {
                blocks.iter()
                    .filter_map(|block| block.get("text").and_then(Value::as_str))
                    .collect::<Vec<&str>>()
                    .join("; ")
            }))
            .unwrap_or_default();
        return Err(RemoteFailure { exit_code: EXIT_DSC_ERROR, message: t!("remote.toolFailed", detail = text).to_string() });
    }

    match result.structured_content {
        Some(Value::Object(mut content)) if content.contains_key("result") => {
            Ok(content.remove("result").unwrap_or(Value::Null))
        },
        _ => Err(RemoteFailure { exit_code: EXIT_DSC_ERROR, message: t!("remote.missingResult").to_string() }),
    }
}

/// Print the result of a remote operation the way the local command does.
///
/// # Arguments
///
/// * `operation` - The operation that produced the result.
/// * `result` - The result object the target returned.
/// * `format` - The requested output format.
///
/// # Errors
///
/// Returns the DSC error exit code when the result reports errors, after printing it.
fn write_result(operation: RemoteOperation, result: &Value, format: Option<&OutputFormat>) -> Result<(), ExitCode> {
    // A local `config export` prints only the exported configuration.
    let output = if operation == RemoteOperation::Export {
        result.get("result").unwrap_or(result)
    } else {
        result
    };
    let json = match serde_json::to_string(output) {
        Ok(json) => json,
        Err(err) => {
            error!("JSON: {err}");
            return Err(ExitCode::from(EXIT_JSON_ERROR));
        }
    };
    write_object(&json, format, false)?;

    if result.get("hadErrors").and_then(Value::as_bool).unwrap_or(false) {
        if operation == RemoteOperation::Export {
            for message in result.get("messages").and_then(Value::as_array).into_iter().flatten() {
                let level = message.get("level").and_then(Value::as_str).unwrap_or_default();
                let text = message.get("message").and_then(Value::as_str).unwrap_or_default();
                error!("{level} {} {text}", t!("subcommand.message"));
            }
        }
        return Err(ExitCode::from(EXIT_DSC_ERROR));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn target_with_user_and_port_parses() {
        let target: RemoteTarget = "ssh://admin@web01:2222".parse().unwrap();
        assert_eq!(target.ssh_args(), ["-s", "-p", "2222", "-l", "admin", "--", "web01", "dsc"]);
        assert_eq!(target.to_string(), "ssh://admin@web01:2222");
    }

    #[test]
    fn target_with_host_only_uses_ssh_defaults() {
        let target: RemoteTarget = "ssh://web01.contoso.com".parse().unwrap();
        assert_eq!(target.ssh_args(), ["-s", "--", "web01.contoso.com", "dsc"]);
    }

    #[test]
    fn ipv6_target_loses_brackets_for_ssh() {
        let target: RemoteTarget = "ssh://[::1]:22".parse().unwrap();
        assert_eq!(target.ssh_args(), ["-s", "-p", "22", "--", "::1", "dsc"]);
        assert_eq!(target.to_string(), "ssh://[::1]:22");
    }

    #[test]
    fn encoded_user_is_decoded() {
        let target: RemoteTarget = "ssh://CONTOSO%5Cadmin@web01".parse().unwrap();
        assert_eq!(target.ssh_args(), ["-s", "-l", "CONTOSO\\admin", "--", "web01", "dsc"]);
    }

    #[test]
    fn host_that_looks_like_an_option_is_rejected() {
        let result = "ssh://-oProxyCommand=calc".parse::<RemoteTarget>();
        assert_eq!(result, Err(RemoteTargetError::InvalidHost("-oProxyCommand=calc".to_string())));
    }

    #[test]
    fn user_that_looks_like_an_option_is_rejected() {
        let result = "ssh://-oProxyCommand=calc@web01".parse::<RemoteTarget>();
        assert_eq!(result, Err(RemoteTargetError::InvalidUser("-oProxyCommand=calc".to_string())));
    }

    #[test]
    fn other_schemes_are_rejected() {
        let result = "https://web01".parse::<RemoteTarget>();
        assert_eq!(result, Err(RemoteTargetError::UnsupportedScheme("https".to_string())));
    }

    #[test]
    fn bare_host_is_not_a_uri() {
        assert!(matches!("web01".parse::<RemoteTarget>(), Err(RemoteTargetError::NotUri(_))));
    }

    #[test]
    fn password_path_query_and_fragment_are_rejected() {
        assert_eq!("ssh://admin:secret@web01".parse::<RemoteTarget>(), Err(RemoteTargetError::PasswordNotAllowed));
        assert_eq!("ssh://web01/etc".parse::<RemoteTarget>(), Err(RemoteTargetError::UnexpectedComponent));
        assert_eq!("ssh://web01?x=1".parse::<RemoteTarget>(), Err(RemoteTargetError::UnexpectedComponent));
        assert_eq!("ssh://web01#x".parse::<RemoteTarget>(), Err(RemoteTargetError::UnexpectedComponent));
    }

    #[test]
    fn minimum_server_version_is_valid() {
        assert_eq!(minimum_server_version().to_string(), MINIMUM_SERVER_VERSION);
    }

    #[test]
    fn exit_code_comes_from_error_data() {
        assert_eq!(tool_error_exit_code(Some(&json!({ "exitCode": 4 }))), 4);
        assert_eq!(tool_error_exit_code(Some(&json!({ "exitCode": 0 }))), EXIT_DSC_ERROR);
        assert_eq!(tool_error_exit_code(Some(&json!({ "exitCode": 4096 }))), EXIT_DSC_ERROR);
        assert_eq!(tool_error_exit_code(None), EXIT_DSC_ERROR);
    }

    #[test]
    fn include_is_found_in_nested_groups() {
        let document = json!({
            "resources": [{
                "name": "group",
                "type": "Microsoft.DSC/Group",
                "properties": { "resources": [{ "name": "inc", "type": "microsoft.dsc/include" }] }
            }]
        });
        assert!(contains_include(&document));
        assert!(!contains_include(&json!({ "resources": [{ "name": "a", "type": "Microsoft.DSC.Debug/Echo" }] })));
    }

    #[test]
    fn optional_arguments_are_left_out() {
        let arguments = build_arguments(RemoteOperation::Get, "{}".to_string(), None, false);
        assert_eq!(Value::Object(arguments), json!({ "operation": "get", "configuration": "{}" }));

        let arguments = build_arguments(RemoteOperation::Set, "{}".to_string(), Some("{}".to_string()), true);
        assert_eq!(Value::Object(arguments), json!({ "operation": "set", "configuration": "{}", "parameters": "{}", "what_if": true }));
    }
}
