// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Write},
    process::{Child, Command, Stdio},
    sync::mpsc::{self, Receiver},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use rust_i18n::t;
use serde_json::{Value, json};

use crate::{
    DscError,
    configure::{config_doc::ExecutionKind, context::Context},
    progress::ResourceProgress,
    util::canonicalize_which,
    settings::DscSettingsScope,
};
use super::{
    command_resource::{log_stderr_line, verify_executable_is_invokable},
    dscresource::{DscResource, redact},
    resource_manifest::{JsonRpc, ReturnKind},
};

#[derive(Debug, Default)]
pub(crate) struct ProcessPool {
    processes: HashMap<String, RpcProcess>,
    failed: bool,
}

fn protocol_error() -> DscError {
    DscError::Operation(t!("dscresources.jsonrpc.protocolError").to_string())
}

impl ProcessPool {
    pub(crate) fn finish(&mut self) -> Result<(), DscError> {
        let mut failure = None;
        for (_, mut process) in self.processes.drain() {
            if let Err(err) = process.shutdown() {
                if failure.is_none() {
                    failure = Some(err);
                } else {
                    tracing::warn!("{err}");
                }
            }
        }
        self.failed = false;
        failure.map_or(Ok(()), Err)
    }

    fn invoke(&mut self, resource: &DscResource, operation: &str, input: Option<&str>, schema_kind: Option<&str>, context: &Context) -> Result<Value, DscError> {
        if self.failed {
            return Err(DscError::Operation(t!("dscresources.jsonrpc.runFailed").to_string()));
        }
        let result = self.invoke_inner(resource, operation, input, schema_kind, context);
        if result.is_err() {
            self.failed = true;
        }
        result
    }

    fn invoke_inner(&mut self, resource: &DscResource, operation: &str, input: Option<&str>, schema_kind: Option<&str>, context: &Context) -> Result<Value, DscError> {
        for process in self.processes.values_mut() {
            process.check_alive()?;
        }
        let manifest = resource.manifest.as_ref().ok_or_else(|| DscError::MissingManifest(resource.type_name.to_string()))?;
        let rpc = manifest.jsonrpc.as_ref().ok_or_else(protocol_error)?;
        manifest.validate_transport()?;
        let target = context.target_resource.as_ref();
        let resource_type = target.map_or(&resource.type_name, |target| &target.type_name);
        let target_descriptor = target.map(|target| json!({
            "type": target.type_name,
            "version": target.version,
            "path": target.path,
            "directory": target.directory,
            "manifest": target.manifest,
            "adaptedManifest": target.adapted_manifest,
            "adaptedContent": target.adapted_content,
            "schema": target.schema,
        }));
        let mut params = json!({
            "operation": operation,
            "executionType": if matches!(operation, "set" | "whatIf" | "delete")
                && context.execution_type == ExecutionKind::WhatIf { "whatIf" } else { "actual" },
            "resourceType": resource_type,
            "context": { "targetResource": target_descriptor },
        });
        if let Some(input) = input.filter(|input| !input.is_empty()) {
            let properties: Value = serde_json::from_str(input)?;
            if !properties.is_object() {
                return Err(protocol_error());
            }
            params["properties"] = properties;
        }
        if let Some(kind) = schema_kind {
            params["context"]["schemaKind"] = json!(kind);
        }
        if context.reuse_resources && context.reuse_resource {
            let key = serde_json::to_string(&(
                &resource.type_name, &resource.version, &resource.path, &resource.directory,
                manifest, &context.security_context,
            ))?;
            let process = match self.processes.entry(key) {
                std::collections::hash_map::Entry::Occupied(entry) => entry.into_mut(),
                std::collections::hash_map::Entry::Vacant(entry) => entry.insert(RpcProcess::start(resource, rpc, context)?),
            };
            process.request("dsc_invoke", Some(params), process.request_timeout, Some(context))
        } else {
            let mut process = RpcProcess::start(resource, rpc, context)?;
            let result = process.request("dsc_invoke", Some(params), process.request_timeout, Some(context));
            let cleanup = process.shutdown();
            match (result, cleanup) {
                (Err(err), cleanup) => {
                    if let Err(cleanup) = cleanup { tracing::warn!("{cleanup}"); }
                    Err(err)
                },
                (Ok(_), Err(err)) => Err(err),
                (Ok(value), Ok(())) => Ok(value),
            }
        }
    }
}

impl Drop for ProcessPool {
    fn drop(&mut self) {
        if let Err(err) = self.finish() {
            tracing::warn!("{err}");
        }
    }
}

#[derive(Debug)]
struct RpcProcess {
    child: Child,
    writer: Option<mpsc::Sender<WriteRequest>>,
    stdout: Receiver<std::io::Result<Option<String>>>,
    readers: Vec<JoinHandle<()>>,
    next_id: u64,
    initialized: bool,
    terminated: bool,
    request_timeout: u64,
    shutdown_timeout: u64,
    resource: DscResource,
}

type WriteRequest = (Value, mpsc::SyncSender<std::io::Result<()>>);

impl RpcProcess {
    fn start(resource: &DscResource, rpc: &JsonRpc, context: &Context) -> Result<Self, DscError> {
        let executable = canonicalize_which(&rpc.executable, resource.directory.as_deref())?;
        verify_executable_is_invokable(std::path::Path::new(&executable))?;
        let mut command = Command::new(&executable);
        command.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
        if let Some(args) = &rpc.args { command.args(args); }
        if let Some(directory) = &resource.directory { command.current_dir(directory); }
        if !context.reuse_resources { command.env("DSC_RESOURCE_REUSE_FORBIDDEN", "1"); }
        command.env_remove("DEBUG_DSC");
        let mut child = command.spawn().map_err(|err| DscError::CommandOperation(err.to_string(), executable))?;
        let mut stdin = child.stdin.take().ok_or_else(protocol_error)?;
        let stdout = child.stdout.take().ok_or_else(protocol_error)?;
        let stderr = child.stderr.take().ok_or_else(protocol_error)?;
        let (sender, receiver) = mpsc::sync_channel(32);
        let (writer, writes) = mpsc::channel::<WriteRequest>();
        let stdin_writer = thread::spawn(move || {
            for (value, completion) in writes {
                let result = (|| {
                    serde_json::to_writer(&mut stdin, &value).map_err(std::io::Error::other)?;
                    stdin.write_all(b"\n")?;
                    stdin.flush()
                })();
                let failed = result.is_err();
                if completion.send(result).is_err() || failed { break; }
            }
        });
        let stdout_reader = thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            loop {
                let mut line = String::new();
                let event = match reader.read_line(&mut line) {
                    Ok(0) => Ok(None),
                    Ok(_) if line.ends_with('\n') => Ok(Some(line)),
                    Ok(_) => Err(std::io::Error::from(std::io::ErrorKind::UnexpectedEof)),
                    Err(err) => Err(err),
                };
                let finished = !matches!(&event, Ok(Some(_)));
                if sender.send(event).is_err() || finished { break; }
            }
        });
        let pid = child.id();
        let dispatcher = tracing::dispatcher::get_default(Clone::clone);
        let stderr_reader = thread::spawn(move || tracing::dispatcher::with_default(&dispatcher, || {
            for line in BufReader::new(stderr).lines() {
                match line {
                    Ok(line) => {
                        let diagnostic = log_stderr_line(&pid, &line);
                        if !diagnostic.is_empty() { tracing::warn!(pid, "{diagnostic}"); }
                    },
                    Err(err) => { tracing::warn!(pid, "{err}"); break; },
                }
            }
        }));
        let settings = crate::settings::global::get_or_init().resolved().resource_timeouts.clone();
        let mut process = Self {
            child, writer: Some(writer), stdout: receiver, readers: vec![stdin_writer, stdout_reader, stderr_reader],
            next_id: 1, initialized: false, terminated: false,
            request_timeout: if settings.request_timeout_ms.scope == DscSettingsScope::Default {
                rpc.request_timeout_ms.unwrap_or(0)
            } else { settings.request_timeout_ms.value },
            shutdown_timeout: if settings.shutdown_timeout_ms.scope == DscSettingsScope::Default {
                rpc.shutdown_timeout_ms.unwrap_or(250)
            } else { settings.shutdown_timeout_ms.value.get() },
            resource: resource.clone(),
        };
        let tracing = crate::settings::global::get_or_init().resolved().tracing.clone();
        let mut params = json!({
            "dscVersion": context.dsc_version,
            "protocolVersions": [1],
            "traceLevel": tracing.level.value,
            "traceFormat": tracing.format.value,
            "securityContext": context.security_context,
        });
        if let Ok(root) = std::env::var("DSC_CONFIG_ROOT") { params["configRoot"] = json!(root); }
        let initialize_timeout = if settings.initialize_timeout_ms.scope == DscSettingsScope::Default {
            rpc.initialize_timeout_ms.unwrap_or(250)
        } else { settings.initialize_timeout_ms.value.get() };
        let result = process.request("initialize", Some(params), initialize_timeout, Some(context))?;
        if result.get("protocolVersion").and_then(Value::as_u64) != Some(1) {
            process.terminate();
            return Err(protocol_error());
        }
        if let Some(operations) = result.get("operations") {
            let Some(operations) = operations.as_array() else { return Err(protocol_error()); };
            if operations.iter().any(|operation| !operation.is_string()) { return Err(protocol_error()); }
            let manifest = serde_json::to_value(resource.manifest.as_ref())?;
            let declared: Vec<&str> = ["get", "set", "whatIf", "test", "delete", "export", "validate", "resolve"]
                .into_iter().filter(|name| manifest.get(*name).is_some_and(|value| !value.is_null())).collect();
            if declared.iter().any(|name| !operations.contains(&json!(name)))
                || operations.iter().any(|name| !declared.iter().any(|declared| name.as_str() == Some(declared))
                    && !(name == "schema" && manifest.pointer("/schema/command").is_some()))
            {
                tracing::warn!("{}", t!("dscresources.jsonrpc.operationsMismatch", resource = resource.type_name));
            }
        }
        process.initialized = true;
        Ok(process)
    }

    fn check_alive(&mut self) -> Result<(), DscError> {
        if let Some(status) = self.child.try_wait()? {
            self.terminated = true;
            return Err(DscError::Operation(t!("dscresources.jsonrpc.unexpectedExit",
                resource = self.resource.type_name, status = status).to_string()));
        }
        Ok(())
    }

    fn send(&mut self, value: &Value, deadline: Option<Instant>, method: &str, timeout_ms: u64) -> Result<(), DscError> {
        let (completion, result) = mpsc::sync_channel(1);
        self.writer.as_ref().ok_or_else(protocol_error)?.send((value.clone(), completion)).map_err(|_| protocol_error())?;
        let write = if let Some(deadline) = deadline {
            result.recv_timeout(deadline.saturating_duration_since(Instant::now())).map_err(|err| match err {
                mpsc::RecvTimeoutError::Timeout => DscError::Operation(t!("dscresources.jsonrpc.timeout",
                    resource = self.resource.type_name, method = method, timeout = timeout_ms).to_string()),
                mpsc::RecvTimeoutError::Disconnected => protocol_error(),
            })?
        } else {
            result.recv().map_err(|_| protocol_error())?
        };
        write?;
        Ok(())
    }

    fn request(&mut self, method: &str, params: Option<Value>, timeout_ms: u64, context: Option<&Context>) -> Result<Value, DscError> {
        let result = self.request_inner(method, params, timeout_ms, context);
        if method == "dsc_invoke"
            && let Some(progress) = context.and_then(|context| context.resource_progress.as_ref()) {
                progress.write_resource_progress(None, None);
        }
        if result.is_err() && !matches!(&result, Err(DscError::JsonRpc { .. })) {
            self.terminate();
        }
        result
    }

    fn request_inner(&mut self, method: &str, params: Option<Value>, timeout_ms: u64, context: Option<&Context>) -> Result<Value, DscError> {
        self.check_alive()?;
        match self.stdout.try_recv() {
            Err(mpsc::TryRecvError::Empty) => {},
            _ => return Err(protocol_error()),
        }
        let id = self.next_id;
        self.next_id = self.next_id.checked_add(1).ok_or_else(protocol_error)?;
        let mut request = json!({"jsonrpc": "2.0", "id": id, "method": method});
        if let Some(params) = params { request["params"] = params; }
        let start = Instant::now();
        let deadline = if timeout_ms == 0 { None } else {
            Some(start.checked_add(Duration::from_millis(timeout_ms)).ok_or_else(|| DscError::Operation(
                t!("dscresources.jsonrpc.timeoutOutOfRange", timeout = timeout_ms).to_string()
            ))?)
        };
        self.send(&request, deadline, method, timeout_ms)?;
        let mut previous_completed = None;
        let mut previous_total = None;
        loop {
            if deadline.is_some_and(|deadline| Instant::now() >= deadline) {
                return Err(DscError::Operation(t!("dscresources.jsonrpc.timeout",
                    resource = self.resource.type_name, method = method, timeout = timeout_ms).to_string()));
            }
            let event = if timeout_ms == 0 {
                self.stdout.recv().map_err(|_| protocol_error())?
            } else {
                let remaining = Duration::from_millis(timeout_ms).saturating_sub(start.elapsed());
                self.stdout.recv_timeout(remaining).map_err(|err| match err {
                    mpsc::RecvTimeoutError::Timeout => DscError::Operation(t!("dscresources.jsonrpc.timeout",
                        resource = self.resource.type_name, method = method, timeout = timeout_ms).to_string()),
                    mpsc::RecvTimeoutError::Disconnected => protocol_error(),
                })?
            };
            let Some(line) = event.map_err(|_| protocol_error())? else {
                self.check_alive()?;
                return Err(protocol_error());
            };
            let value: Value = serde_json::from_str(&line).map_err(|_| protocol_error())?;
            if !value.is_object() || value["jsonrpc"] != "2.0" { return Err(protocol_error()); }
            if value.get("method").is_some() {
                if method != "dsc_invoke" || value.get("id").is_some()
                    || value.get("result").is_some() || value.get("error").is_some()
                    || !value["params"].is_object() || value["params"]["requestId"].as_u64() != Some(id)
                {
                    return Err(protocol_error());
                }
                let params = &value["params"];
                let message = params["message"].as_str().ok_or_else(protocol_error)?;
                match value["method"].as_str() {
                    Some("dsc_trace") => {
                        let pid = self.child.id();
                        match params["level"].as_str() {
                            Some("error") => tracing::error!(pid, "{message}"),
                            Some("warn") => tracing::warn!(pid, "{message}"),
                            Some("info") => tracing::info!(pid, "{message}"),
                            Some("debug") => tracing::debug!(pid, "{message}"),
                            Some("trace") => tracing::trace!(pid, "{message}"),
                            _ => return Err(protocol_error()),
                        }
                    },
                    Some("dsc_progress") => {
                        let number = |name| -> Result<Option<f64>, DscError> {
                            params.get(name).map(|value| value.as_f64()
                                .filter(|number| number.is_finite()).ok_or_else(protocol_error)).transpose()
                        };
                        let completed = number("completed")?;
                        let total = number("total")?;
                        if completed.is_some_and(|value| value < 0.0 || previous_completed.is_some_and(|previous| value < previous))
                            || completed.is_some_and(|value| previous_total.is_some_and(|total| value > total))
                            || total.is_some_and(|value| value <= 0.0 || completed.is_none_or(|completed| completed > value)
                                || previous_total.is_some_and(|previous| value != previous))
                        {
                            return Err(protocol_error());
                        }
                        if completed.is_some() { previous_completed = completed; }
                        if total.is_some() { previous_total = total; }
                        if let Some(progress) = context.and_then(|context| context.resource_progress.as_ref()) {
                            let numbers = (completed.is_some() || total.is_some()).then_some(ResourceProgress { completed, total });
                            progress.write_resource_progress(Some(message), numbers);
                        }
                    },
                    _ => return Err(protocol_error()),
                }
                continue;
            }
            if value.get("id").is_none() || value.get("result").is_some() == value.get("error").is_some()
                || (value["id"].as_u64() != Some(id) && !(value["id"].is_null() && value.get("error").is_some()))
            {
                return Err(protocol_error());
            }
            if let Some(error) = value.get("error") {
                let code = error["code"].as_i64().and_then(|code| i32::try_from(code).ok()).ok_or_else(protocol_error)?;
                if value["id"].is_null() && !matches!(code, -32700 | -32600) { return Err(protocol_error()); }
                let message = error["message"].as_str().ok_or_else(protocol_error)?;
                let mut message = message.to_string();
                if code != 0 && !(-32768..=-32000).contains(&code)
                    && let Some(description) = self.resource.manifest.as_ref().and_then(|manifest| manifest.exit_codes.get_code(code)) {
                        message.push_str(": ");
                        message.push_str(description);
                }
                return Err(DscError::JsonRpc { resource: self.resource.type_name.to_string(), code,
                    message, data: error.get("data").map(|data| Box::new(redact(data))) });
            }
            if value["id"].as_u64() != Some(id) { return Err(protocol_error()); }
            return Ok(value["result"].clone());
        }
    }

    fn shutdown(&mut self) -> Result<(), DscError> {
        if self.terminated { return Ok(()); }
        self.check_alive()?;
        if !self.initialized { self.terminate(); return Ok(()); }
        let result = self.request("shutdown", None, self.shutdown_timeout, None)?;
        if !result.is_null() { self.terminate(); return Err(protocol_error()); }
        let deadline = Instant::now().checked_add(Duration::from_millis(self.shutdown_timeout)).ok_or_else(|| DscError::Operation(
            t!("dscresources.jsonrpc.timeoutOutOfRange", timeout = self.shutdown_timeout).to_string()
        ))?;
        self.send(&json!({"jsonrpc": "2.0", "method": "exit"}), Some(deadline), "exit", self.shutdown_timeout)?;
        loop {
            if let Some(status) = self.child.try_wait()? {
                self.terminated = true;
                if !status.success() { return Err(protocol_error()); }
                // Wait for the reader's EOF so scheduling cannot hide trailing output.
                return match self.stdout.recv_timeout(deadline.saturating_duration_since(Instant::now())) {
                    Ok(Ok(None)) => Ok(()),
                    Err(mpsc::RecvTimeoutError::Timeout) => Err(DscError::Operation(t!("dscresources.jsonrpc.timeout",
                        resource = self.resource.type_name, method = "exit", timeout = self.shutdown_timeout).to_string())),
                    _ => Err(protocol_error()),
                };
            }
            if Instant::now() >= deadline {
                self.terminate();
                return Err(DscError::Operation(t!("dscresources.jsonrpc.timeout",
                    resource = self.resource.type_name, method = "exit", timeout = self.shutdown_timeout).to_string()));
            }
            thread::sleep(Duration::from_millis(1));
        }
    }

    fn terminate(&mut self) {
        if !self.terminated {
            if let Err(err) = self.child.kill() {
                tracing::warn!("{err}");
            }
            if let Err(err) = self.child.wait() {
                tracing::warn!("{err}");
            }
            self.terminated = true;
        }
    }
}

impl Drop for RpcProcess {
    fn drop(&mut self) {
        self.terminate();
        self.writer.take();
        for reader in self.readers.drain(..) {
            // Descendants may inherit a pipe; never wait indefinitely for its EOF.
            if reader.is_finished() && reader.join().is_err() {
                tracing::warn!("{}", t!("dscresources.jsonrpc.readerFailed"));
            }
        }
    }
}

/// Normalize the wire value into the command runner's semantic input format.
pub(crate) fn invoke(resource: &DscResource, operation: &str, input: Option<&str>, schema_kind: Option<&str>,
    returns: Option<&ReturnKind>, context: &Context) -> Result<(i32, String, String), DscError>
{
    let mut pool = context.resource_processes.lock().unwrap();
    let value = pool.invoke(resource, operation, input, schema_kind, context)?;
    let result = normalize_result(resource, operation, returns, context, value);
    if result.is_err() { pool.failed = true; }
    result
}

fn normalize_result(resource: &DscResource, operation: &str, returns: Option<&ReturnKind>, context: &Context,
    value: Value) -> Result<(i32, String, String), DscError>
{
    let group = match operation {
        "set" | "whatIf" => serde_json::from_value::<Vec<crate::configure::config_result::ResourceSetResult>>(value.clone()).is_ok(),
        "test" => serde_json::from_value::<Vec<crate::configure::config_result::ResourceTestResult>>(value.clone()).is_ok(),
        _ => false,
    };
    if group { return Ok((0, serde_json::to_string(&value)?, String::new())); }
    let stdout = match operation {
        "get" => {
            if !value.is_object() {
                let _: Vec<crate::configure::config_result::ResourceGetResult> =
                    serde_json::from_value(value.clone()).map_err(|_| protocol_error())?;
            }
            serde_json::to_string(&value)?
        },
        "export" => {
            let instances = value.as_array().ok_or_else(protocol_error)?;
            if instances.iter().any(|value| !value.is_object()) { return Err(protocol_error()); }
            let lines: Result<Vec<_>, _> = instances.iter().map(serde_json::to_string).collect();
            lines?.join("\n")
        },
        "set" | "whatIf" | "test" if returns == Some(&ReturnKind::StateAndDiff) => {
            let state = value.get("actualState").ok_or_else(protocol_error)?;
            if !state.is_object() && resource.kind == super::resource_manifest::Kind::Resource { return Err(protocol_error()); }
            let diff = value.get("diff").ok_or_else(protocol_error)?;
            let _: Vec<String> = serde_json::from_value(diff.clone()).map_err(|_| protocol_error())?;
            format!("{}\n{}", serde_json::to_string(state)?, serde_json::to_string(diff)?)
        },
        "set" | "whatIf" | "test" if returns.is_none() && resource.kind == super::resource_manifest::Kind::Resource => {
            if !value.is_null() { return Err(protocol_error()); }
            String::new()
        },
        "delete" if context.execution_type == ExecutionKind::Actual => {
            if !value.is_null() { return Err(protocol_error()); }
            String::new()
        },
        "schema" => {
            if !value.is_object() && !value.is_boolean() { return Err(protocol_error()); }
            serde_json::to_string(&value)?
        },
        "validate" => {
            let _: crate::resources::invoke_result::ValidateResult =
                serde_json::from_value(value.clone()).map_err(|_| protocol_error())?;
            serde_json::to_string(&value)?
        },
        "resolve" => {
            let _: crate::resources::invoke_result::ResolveResult =
                serde_json::from_value(value.clone()).map_err(|_| protocol_error())?;
            serde_json::to_string(&value)?
        },
        "set" | "whatIf" | "test" | "delete" => {
            if !value.is_object() && resource.kind == super::resource_manifest::Kind::Resource { return Err(protocol_error()); }
            serde_json::to_string(&value)?
        },
        _ => serde_json::to_string(&value)?,
    };
    Ok((0, stdout, String::new()))
}
