// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Command plug-in support for the `dsc` command line.
//!
//! A command plug-in adds a top-level command to `dsc` without being compiled into it. This module
//! parses the command line with the adjustments plug-in commands need, runs the plug-in that
//! provides a command, and adds plug-in commands to the top-level help and the completion scripts.
//!
//! Plug-ins are only discovered when the result is needed: when the first argument isn't a
//! built-in command, when the top-level help is shown, when plug-ins are listed, and when a
//! completion script is generated. A built-in command never triggers plug-in discovery.

use crate::args::{Args, SubCommand};
use crate::util::{current_dsc_version, EXIT_DSC_ERROR};
use clap::{Arg, ArgAction, Command, CommandFactory, FromArgMatches, ValueEnum, ValueHint};
use clap::builder::ValueParser;
use clap::error::ErrorKind;
use dsc_lib::plugins::discovery::{discover_plugins, find_plugin, find_plugin_by_type};
use dsc_lib::plugins::dscplugin::DscPlugin;
use dsc_lib::progress::ProgressFormat;
use dsc_lib::settings::{global as global_settings, sources::EnvironmentData};
use dsc_lib::util::get_exe_path;
use rust_i18n::t;
use std::env;
use std::ffi::OsString;
use std::fmt::Write as _;
use std::process::{exit, ExitCode, ExitStatus};
use tracing::{debug, error, warn};

/// The environment variable that holds the absolute path of the `dsc` executable that started a
/// plug-in.
pub const DSC_EXE_ENV_VAR: &str = "DSC_EXE";
/// The environment variable that holds the semantic version of the `dsc` executable that started
/// a plug-in.
pub const DSC_VERSION_ENV_VAR: &str = "DSC_VERSION";
/// The environment variable that holds the command name a plug-in was invoked as.
pub const DSC_PLUGIN_COMMAND_ENV_VAR: &str = "DSC_PLUGIN_COMMAND";
/// The environment variable that holds the directory that contains the manifest of a plug-in.
pub const DSC_PLUGIN_DIR_ENV_VAR: &str = "DSC_PLUGIN_DIR";
/// The environment variable that holds the value of the `--progress-format` option, when the user
/// specified it.
pub const DSC_PROGRESS_FORMAT_ENV_VAR: &str = "DSC_PROGRESS_FORMAT";

/// The name of the help subcommand that clap adds to every command with subcommands.
const HELP_COMMAND: &str = "help";
/// The exit code clap uses for a usage error, like an unrecognized subcommand.
const EXIT_USAGE_ERROR: i32 = 2;

/// Parses the command line arguments of the current process.
///
/// This function behaves like [`clap::Parser::parse()`], with two additions for plug-ins:
///
/// - `dsc help <command>` for a command that isn't built in is parsed as the plug-in command
///   followed by `--help`, so the plug-in prints its own help.
/// - The top-level help lists the discovered plug-in commands after the built-in commands.
///
/// Like `parse()`, this function prints the message and exits the process when the arguments are
/// invalid or ask for help or the version.
///
/// # Returns
///
/// The parsed arguments. An argument that isn't a built-in command is returned as
/// [`SubCommand::External`] without checking whether a plug-in provides it. [`dispatch()`] does
/// that.
pub fn parse_args() -> Args {
    let raw_args: Vec<OsString> = env::args_os().collect();

    if let Some(args) = parse_plugin_help_request(&raw_args) {
        return args;
    }

    let mut command = Args::command();
    let mut matches = match command.try_get_matches_from_mut(&raw_args) {
        Ok(matches) => matches,
        Err(err) => {
            if is_top_level_help(&err, &mut command) {
                print_top_level_help_with_plugins(&raw_args);
            }
            err.exit()
        }
    };

    match Args::from_arg_matches_mut(&mut matches) {
        Ok(args) => args,
        Err(err) => err.format(&mut command).exit(),
    }
}

/// Returns the names and aliases of the commands that are built into `dsc`.
///
/// A plug-in can't use any of these names, because built-in commands are matched first.
///
/// # Returns
///
/// The name and every alias of each built-in command, and the `help` command that clap adds.
#[must_use]
pub fn reserved_commands() -> Vec<String> {
    let command = Args::command();
    let mut reserved: Vec<String> = command
        .get_subcommands()
        .flat_map(|subcommand| {
            std::iter::once(subcommand.get_name()).chain(subcommand.get_all_aliases())
        })
        .map(str::to_string)
        .collect();
    reserved.push(HELP_COMMAND.to_string());
    reserved
}

/// Discovers the plug-ins that are available to `dsc`.
///
/// # Returns
///
/// The discovered plug-ins, including shadowed plug-ins. When discovery fails, the function logs
/// the error as a warning and returns an empty list, so that a problem with the resource path
/// settings doesn't hide the built-in commands.
#[must_use]
pub fn discover() -> Vec<DscPlugin> {
    match discover_plugins(&reserved_commands()) {
        Ok(plugins) => plugins,
        Err(err) => {
            warn!("{}", t!("plugin.discoveryFailed", error = err));
            Vec::new()
        }
    }
}

/// Returns the `dsc` command definition with a subcommand for every plug-in command.
///
/// The returned definition describes plug-in commands the way it describes built-in commands, so
/// clap can suggest them for a mistyped command and include them in completion scripts. It doesn't
/// accept other external subcommands. It must not be used to run a plug-in, because the
/// subcommands only describe the name and description of each plug-in command, not its arguments.
///
/// # Arguments
///
/// * `plugins` - The discovered plug-ins. Shadowed plug-ins are ignored.
#[must_use]
pub fn command_with_plugins(plugins: &[DscPlugin]) -> Command {
    // Clap turns external subcommands back on while it builds a command that has a value parser
    // for them, so the value parser has to be reset together with the setting.
    let mut command = Args::command()
        .allow_external_subcommands(false)
        .external_subcommand_value_parser(None::<ValueParser>);
    for plugin in plugins.iter().filter(|plugin| !plugin.shadowed) {
        command = command.subcommand(plugin_subcommand(plugin));
    }
    command
}

/// Runs the plug-in that provides a command and exits with the plug-in's exit code.
///
/// The plug-in runs in the caller's working directory with the standard streams of `dsc`. DSC
/// passes the arguments from the manifest followed by the arguments the user specified, and sets
/// the environment variables that tell the plug-in how it was invoked.
///
/// When no plug-in provides the command, the function reports the same unrecognized subcommand
/// error as for any unknown command and exits with the exit code for a usage error.
///
/// # Arguments
///
/// * `external_args` - The command name followed by the arguments the user specified after it.
/// * `progress_format` - The value of the `--progress-format` option, if the user specified it.
///
/// # Returns
///
/// This function only returns when the plug-in couldn't be started. Otherwise it exits the
/// process.
///
/// # Errors
///
/// Returns the exit code for a DSC error when the executable of the plug-in can't be resolved or
/// started.
pub fn dispatch(external_args: &[OsString], progress_format: Option<ProgressFormat>) -> Result<(), ExitCode> {
    let plugins = discover();

    let invoked_as = external_args.first().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default();
    // A type name always contains a slash and a command name never does, so the name itself
    // tells which of the two the user specified.
    let by_type_name = invoked_as.contains('/');
    let found = if by_type_name {
        find_plugin_by_type(&plugins, &invoked_as)
    } else {
        find_plugin(&plugins, &invoked_as)
    };
    let Some(plugin) = found else {
        exit_with_unrecognized_subcommand(&plugins);
    };
    let user_args = &external_args[1..];

    // Naming a plug-in by its type is unambiguous, so only warn when the command name was used.
    if !by_type_name {
        for shadowed in plugins.iter().filter(|other| other.shadowed && other.command == plugin.command) {
            warn!("{}", t!(
                "plugin.shadowedPlugin",
                command = plugin.command,
                plugin = plugin.type_name,
                shadowed = shadowed.type_name
            ));
        }
    }

    if let Some(message) = &plugin.deprecation_message {
        warn!("{}", t!("plugin.deprecationMessage", plugin = plugin.type_name, message = message));
    }

    let invocation = match plugin.resolve_invocation() {
        Ok(invocation) => invocation,
        Err(err) => {
            error!("{}", t!("plugin.failedToResolve", command = plugin.command, plugin = plugin.type_name, error = err));
            return Err(ExitCode::from(EXIT_DSC_ERROR));
        }
    };

    let mut process = std::process::Command::new(&invocation.executable);
    process
        .args(&invocation.args)
        .args(user_args)
        .env(DSC_VERSION_ENV_VAR, current_dsc_version().to_string())
        .env(DSC_PLUGIN_COMMAND_ENV_VAR, &invoked_as)
        .env(DSC_PLUGIN_DIR_ENV_VAR, &plugin.directory)
        .env(
            EnvironmentData::DSC_TRACE_FORMAT_ENV_VAR,
            global_settings::get_or_init().resolved().tracing.format.value.to_string()
        );
    match get_exe_path() {
        Ok(exe_path) => { process.env(DSC_EXE_ENV_VAR, exe_path); },
        Err(err) => debug!("{}", t!("plugin.failedToGetExePath", error = err)),
    }
    if let Some(name) = progress_format.and_then(|format| format.to_possible_value()) {
        process.env(DSC_PROGRESS_FORMAT_ENV_VAR, name.get_name());
    }

    debug!("{}", t!(
        "plugin.invoking",
        command = plugin.command,
        plugin = plugin.type_name,
        executable = invocation.executable.display()
    ));
    match process.status() {
        Ok(status) => exit(exit_code_for(status)),
        Err(err) => {
            error!("{}", t!(
                "plugin.failedToStart",
                command = plugin.command,
                executable = invocation.executable.display(),
                error = err
            ));
            Err(ExitCode::from(EXIT_DSC_ERROR))
        }
    }
}

/// Parses `dsc help <command>` for a command that isn't built in.
///
/// Clap's `help` subcommand only knows the built-in commands. To recognize the request, this
/// function parses the arguments with the `help` subcommand disabled, which turns `help` into an
/// external subcommand whose arguments are the command path to show help for.
///
/// # Arguments
///
/// * `raw_args` - The arguments of the process, including the program name.
///
/// # Returns
///
/// The parsed arguments with the plug-in command as the subcommand and `--help` appended to its
/// arguments, or [`None`] when the arguments aren't a help request for a command that isn't built
/// in. Whether a plug-in provides the command is checked by [`dispatch()`].
fn parse_plugin_help_request(raw_args: &[OsString]) -> Option<Args> {
    // Building the command definition a second time isn't free, so only do it when the arguments
    // can be a help request.
    if !raw_args.iter().any(|arg| arg == HELP_COMMAND) {
        return None;
    }

    let mut command = Args::command().disable_help_subcommand(true);
    let mut matches = command.try_get_matches_from_mut(raw_args).ok()?;
    let mut args = Args::from_arg_matches_mut(&mut matches).ok()?;

    let SubCommand::External(external_args) = &args.subcommand else {
        return None;
    };
    let (first, command_path) = external_args.split_first()?;
    if first != HELP_COMMAND {
        return None;
    }
    let target = command_path.first()?.to_string_lossy();
    if reserved_commands().iter().any(|reserved| *reserved == target) {
        return None;
    }

    let mut plugin_args = command_path.to_vec();
    plugin_args.push(OsString::from("--help"));
    args.subcommand = SubCommand::External(plugin_args);
    Some(args)
}

/// Indicates whether a parsing error is the request to show the top-level help of `dsc`.
///
/// Clap reports a help request as an error that carries the rendered help, without saying which
/// command the help is for. This function compares that text to the help of the top-level command.
///
/// # Arguments
///
/// * `err` - The error that parsing the arguments returned.
/// * `command` - The command definition that parsed the arguments. It must be the same instance,
///   because parsing records the program name the usage line shows.
fn is_top_level_help(err: &clap::Error, command: &mut Command) -> bool {
    if err.kind() != ErrorKind::DisplayHelp {
        return false;
    }

    let rendered = err.render().to_string();
    rendered == command.render_help().to_string() || rendered == command.render_long_help().to_string()
}

/// Prints the top-level help with the plug-in commands listed after the built-in commands, then
/// exits the process.
///
/// # Arguments
///
/// * `raw_args` - The arguments of the process, including the program name. They are parsed again
///   so that clap renders the short or the long help, as requested.
///
/// # Returns
///
/// This function returns without printing anything when there are no plug-in commands to list, so
/// the caller can print the unmodified help.
fn print_top_level_help_with_plugins(raw_args: &[OsString]) {
    // The arguments never got parsed, so the settings haven't been loaded yet. Tracing isn't
    // enabled either, which keeps messages about invalid manifests out of the help.
    let mut settings = global_settings::get_or_init();
    settings.load();
    settings.resolve_all();
    drop(settings);

    let plugins = discover();
    let mut provided: Vec<&DscPlugin> = plugins.iter().filter(|plugin| !plugin.shadowed).collect();
    if provided.is_empty() {
        return;
    }
    provided.sort_by(|a, b| a.command.cmp(&b.command));

    let command = Args::command();
    let template = top_level_help_template(&command, &provided);
    if let Err(err) = command.help_template(template).try_get_matches_from_mut(raw_args) {
        err.exit();
    }
}

/// Builds the help template for the top-level command with a section for plug-in commands.
///
/// The template reproduces clap's default layout and adds the plug-in commands between the
/// built-in commands and the options.
///
/// # Arguments
///
/// * `command` - The top-level command definition, for its styles and the names of its
///   subcommands.
/// * `plugins` - The plug-ins that provide a command, in the order to list them.
fn top_level_help_template(command: &Command, plugins: &[&DscPlugin]) -> String {
    let styles = command.get_styles();
    let header = styles.get_header();
    let literal = styles.get_literal();

    // Use the same column for the descriptions as the built-in commands when the names allow it.
    let name_width = command
        .get_subcommands()
        .filter(|subcommand| !subcommand.is_hide_set())
        .map(|subcommand| subcommand.get_name().len())
        .chain(std::iter::once(HELP_COMMAND.len()))
        .chain(plugins.iter().map(|plugin| plugin.command.len()))
        .max()
        .unwrap_or_default();

    let mut plugin_lines = String::new();
    for plugin in plugins {
        let name = plugin.command.as_ref();
        let description = plugin.description.as_deref().unwrap_or_default();
        let line = format!("  {literal}{name}{literal:#}{:width$}  {description}", "", width = name_width - name.len());
        let _ = writeln!(plugin_lines, "{}", line.trim_end());
    }

    format!(
        "{{before-help}}{{about-with-newline}}\n\
        {{usage-heading}} {{usage}}\n\
        \n\
        {header}{commands}:{header:#}\n\
        {{subcommands}}\n\
        \n\
        {header}{plugin_commands}:{header:#}\n\
        {plugin_lines}\
        \n\
        {header}{options}:{header:#}\n\
        {{options}}{{after-help}}",
        commands = t!("plugin.helpHeadingCommands"),
        plugin_commands = t!("plugin.helpHeadingPluginCommands"),
        options = t!("plugin.helpHeadingOptions"),
    )
}

/// Returns the subcommand definition that describes a plug-in command.
///
/// The definition has the name and description of the plug-in command. It accepts `--help`, which
/// every plug-in must support, and any other arguments, completed as paths because DSC doesn't
/// know the arguments of the plug-in.
///
/// # Arguments
///
/// * `plugin` - The plug-in that provides the command.
fn plugin_subcommand(plugin: &DscPlugin) -> Command {
    // Clap only takes names with a static lifetime unless its `string` feature is enabled. The
    // process ends right after the command definition is used, so leaking a few short names is
    // cheaper than enabling the feature for every command definition in `dsc`.
    let name: &'static str = Box::leak(plugin.command.to_string().into_boxed_str());

    let mut subcommand = Command::new(name)
        .disable_help_flag(true)
        .arg(
            Arg::new("help")
                .long("help")
                .action(ArgAction::SetTrue)
                .help(t!("plugin.helpOption").to_string())
        )
        .arg(
            Arg::new("args")
                .num_args(0..)
                .trailing_var_arg(true)
                .allow_hyphen_values(true)
                .value_hint(ValueHint::AnyPath)
        );
    if let Some(description) = &plugin.description {
        subcommand = subcommand.about(description.clone());
    }
    subcommand
}

/// Reports the unrecognized subcommand error for the arguments of the process and exits.
///
/// The arguments are parsed again with a command definition that doesn't accept external
/// subcommands, so clap reports the error exactly as it does for any unknown command. The
/// definition includes the plug-in commands so the suggestion in the error covers them.
///
/// # Arguments
///
/// * `plugins` - The discovered plug-ins.
fn exit_with_unrecognized_subcommand(plugins: &[DscPlugin]) -> ! {
    let raw_args: Vec<OsString> = env::args_os().collect();
    match command_with_plugins(plugins).try_get_matches_from_mut(&raw_args) {
        Err(err) => err.exit(),
        Ok(_) => {
            // The arguments only parse when they name a built-in or plug-in command, and this
            // function is only called when they don't.
            let name = raw_args.get(1).map(|arg| arg.to_string_lossy().into_owned()).unwrap_or_default();
            error!("{}", t!("plugin.unrecognizedSubcommand", command = name));
            exit(EXIT_USAGE_ERROR)
        }
    }
}

/// Returns the exit code for `dsc` to exit with after a plug-in finished.
///
/// # Arguments
///
/// * `status` - The exit status of the plug-in process.
///
/// # Returns
///
/// The exit code of the plug-in. When a signal ended the plug-in, the function returns `128` plus
/// the signal number, which is how shells report it.
fn exit_code_for(status: ExitStatus) -> i32 {
    if let Some(code) = status.code() {
        return code;
    }

    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(signal) = status.signal() {
            return 128 + signal;
        }
    }

    i32::from(EXIT_DSC_ERROR)
}
