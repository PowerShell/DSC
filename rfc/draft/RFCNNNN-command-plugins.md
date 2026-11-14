---
RFC:          RFCNNNN
Author:       @Gijsreyn
Sponsor:      @Gijsreyn
Status:       Draft
SupercededBy: null
Version:      1.0
Area:         CLI
CommentsDue:  2026-12-14
---

# Command plug-ins for `dsc`

This RFC proposes command plug-ins: executables that add a top-level command to `dsc` without
being compiled into it. A plug-in is described by a plug-in manifest, a `*.dsc.plugin.json` file
that DSC finds the same way it finds resource and extension manifests. When the first argument to
`dsc` isn't a built-in command, DSC looks for a plug-in that provides that command, runs the
plug-in's executable with the remaining arguments, and exits with the plug-in's exit code.

Dispatch works the way external commands work in `git`, `dotnet`, `cargo`, and `kubectl`.
Discovery uses a manifest instead of a file name convention, as discussed in [#461][01]. The first
plug-in is the linter from [#1768][02], which ships as the `dsclint` executable and becomes
available as `dsc lint`.

## Motivation

> As a configuration author,
> I want tools that work with DSC, such as the linter, to be commands of `dsc`,
> so that I find them in `dsc --help` and in shell completion instead of having to know that a
> separate executable exists.

> As an operator who installs `dsc` on servers or embeds it in a higher-order tool,
> I want author-time tooling to stay out of the `dsc` executable,
> so that the engine I deploy stays small and I decide per machine which commands exist.

> As a tool developer,
> I want to add a command to `dsc` without contributing it to the DSC repository,
> so that I can ship and version my tool on my own schedule, in the language I prefer.

Everything `dsc` can do today is compiled into one executable. [#461][01] has asked for plug-ins
since 2024, and the discussion there makes the case for a small core: `dsc` runs on clients and
servers and inside higher-order tools, so features that only matter while authoring, such as a
linter, a documentation viewer, or a manifest scaffolder, shouldn't add dependencies and attack
surface to every installation.

The linter made the need concrete. The RFC in [#1766][03] proposed `dsc lint` as a built-in
command. The implementation in [#1768][02] ships it as a separate `dsclint` executable instead,
and the working group suggested a plug-in system to make it reachable from `dsc` again. A separate
executable keeps the engine small, but the user pays for it:

- `dsc --help` doesn't list `dsclint`, and the script from `dsc completer` doesn't complete it.
- `dsclint` has to redefine the options that `dsc` accepts before the command, such as
  `--trace-level` and `--trace-format`.
- Every further tool adds another executable name to remember.

Plug-ins aren't extensions. An extension gives the engine a capability (`discover`, `import`,
`secret`) that DSC calls with a fixed contract while it processes an operation. A plug-in adds a
command that a person or a script calls, with arguments and output that the plug-in defines.

## Proposed experience

### Run a plug-in command

A DSC release that includes `dsclint` and its plug-in manifest has a `lint` command:

```sh
dsc lint --file ./demo.dsc.yaml
```

The output and the exit code are those of `dsclint --file ./demo.dsc.yaml`. Options before the
command belong to `dsc` and are handed to the plug-in. Options after the command belong to the
plug-in:

```sh
dsc --trace-level debug lint --file ./demo.dsc.yaml --fail-on warning
```

### Find plug-in commands

`dsc --help` lists plug-in commands after the built-in commands:

```text
Apply configuration or invoke specific DSC resources

Usage: dsc [OPTIONS] <COMMAND>

Commands:
  completer  Generate a shell completion script
  config     Apply a configuration document
  action     Operations on DSC actions
  extension  Operations on DSC extensions
  function   Operations on DSC functions
  plugin     Operations on DSC plug-ins
  server     Use DSC as a server over JSON-RPC (useful as MCP server)
  resource   Invoke a specific DSC resource
  schema     Get the JSON schema for a DSC type
  help       Print this message or the help of the given subcommand(s)

Plug-in commands:
  lint       Check configuration documents for problems without applying them

Options:
  ...
```

`dsc plugin list` shows which plug-in provides each command:

```text
Command  Type                Version  Description
-----------------------------------------------------------------------------------------------
lint     Microsoft.DSC/Lint  3.4.0    Check configuration documents for problems without applying them
```

A command that is neither built in nor provided by a plug-in fails as it does today, and the
suggestion covers plug-in commands:

```text
error: unrecognized subcommand 'lnt'

  tip: a similar subcommand exists: 'lint'

Usage: dsc [OPTIONS] <COMMAND>

For more information, try '--help'.
```

### Choose between plug-ins with the same command

When two plug-ins define the same command, the one DSC finds first provides it. DSC warns about
the other, and either plug-in can be run by its type name:

```sh
dsc lint --file ./demo.dsc.yaml
```

```text
WARN Command 'lint' is provided by plug-in 'Microsoft.DSC/Lint'. Plug-in 'OpenDSC/Lint' defines
the same command and is shadowed. To run it, use 'dsc OpenDSC/Lint'.
```

```sh
dsc OpenDSC/Lint --file ./demo.dsc.yaml
```

### Get help for a plug-in command

`dsc lint --help` and `dsc help lint` both show the help that the plug-in prints.

### Complete plug-in commands

The script from `dsc completer` completes plug-in commands and, when the plug-in ships a
completion spec, their options and values:

```powershell
dsc completer powershell | Out-String | Invoke-Expression

dsc l<Tab>                  # lint
dsc lint --f<Tab>           # --fail-on  --file
dsc lint --fail-on <Tab>    # error  warning  info
```

Users of [carapace][04] generate one spec for `dsc` and every installed plug-in:

```powershell
dsc completer carapace > "$env:AppData/carapace/specs/dsc.yaml"
```

### Author a plug-in

A plug-in is an executable or a script, plus a manifest in a directory that DSC searches. The
following manifest adds `dsc graph`, implemented as a PowerShell script that sits next to the
manifest:

```json
{
  "$schema": "https://aka.ms/dsc/schemas/v3/bundled/plugin/manifest.json",
  "type": "Contoso.DSC/Graph",
  "version": "0.1.0",
  "description": "Show the dependency graph of a configuration document",
  "condition": "[not(equals(tryWhich('pwsh'), null()))]",
  "command": {
    "name": "graph",
    "executable": "pwsh",
    "args": [
      "-NoLogo",
      "-NoProfile",
      "-NonInteractive",
      "-File",
      { "manifestRelativePath": "graph.ps1" }
    ]
  }
}
```

```sh
dsc config get --file ./app.dsc.yaml | dsc graph
```

## Specification

### Terms

- A **built-in command** is a command compiled into `dsc`: `action`, `completer`, `config`,
  `extension`, `function`, `help`, `resource`, `schema`, `server` with its alias `mcp`, and the
  `plugin` command this RFC adds.
- A **plug-in** is a plug-in manifest together with the executable it names.
- A **plug-in command** is the top-level command that a plug-in adds to `dsc`.

### Plug-in manifest

A plug-in manifest is a JSON or YAML file whose name ends with `.dsc.plugin.json`,
`.dsc.plugin.yaml`, or `.dsc.plugin.yml`. A manifest list (`.dsc.manifests.json` and its YAML
forms) gains a `plugins` array next to `resources`, `extensions`, and `actions`, so one file can
describe several plug-ins.

The manifest reuses the properties that resource and extension manifests already share:

| Property             | Required | Description                                                                                            |
| :------------------- | :------: | :----------------------------------------------------------------------------------------------------- |
| `$schema`            |   Yes    | The URI of the plug-in manifest schema, `https://aka.ms/dsc/schemas/v3/bundled/plugin/manifest.json`.  |
| `type`               |   Yes    | The fully qualified type name that identifies the plug-in, in the format resources and extensions use. |
| `version`            |   Yes    | The semantic version of the plug-in.                                                                   |
| `description`        |    No    | One line that `dsc --help` and `dsc plugin list` show for the command.                                 |
| `tags`               |    No    | Tags for the plug-in.                                                                                  |
| `condition`          |    No    | An expression. When it evaluates to `false`, DSC treats the plug-in as not installed.                  |
| `deprecationMessage` |    No    | A message DSC writes as a warning each time the command is invoked.                                    |
| `command`            |   Yes    | The command the plug-in adds and how DSC runs it.                                                      |
| `completion`         |    No    | Where to find the completion spec for the command. See [Shell completion](#shell-completion).          |
| `metadata`           |    No    | Arbitrary data, as in other manifests.                                                                 |

The `command` object has these properties:

| Property     | Required | Description                                                                                            |
| :----------- | :------: | :----------------------------------------------------------------------------------------------------- |
| `name`       |   Yes    | The command name the user types after `dsc`. See [Command names](#command-names).                      |
| `executable` |   Yes    | The executable to run, resolved the way DSC resolves executables for resource and extension manifests. |
| `args`       |    No    | Arguments DSC passes before the arguments the user typed.                                              |

An item of `args` is one of the following:

- A string, which DSC passes as written.
- An object with the single property `manifestRelativePath`. DSC replaces it with the absolute
  path of that file, resolved against the directory that contains the manifest. A plug-in runs in
  the caller's working directory, not in the manifest directory, so this is how a manifest names a
  script that ships next to it.

A manifest describes exactly one command. Commands nested under it, such as `dsc package build`,
are parsed by the plug-in.

The manifest for `dsclint`:

```json
{
  "$schema": "https://aka.ms/dsc/schemas/v3/bundled/plugin/manifest.json",
  "type": "Microsoft.DSC/Lint",
  "version": "3.4.0",
  "description": "Check configuration documents for problems without applying them",
  "command": {
    "name": "lint",
    "executable": "dsclint"
  },
  "completion": {
    "spec": "dsclint.completion.yaml"
  }
}
```

`dsc schema --type plugin-manifest` returns the schema.

### Command names

- A command name matches `^[a-z][a-z0-9]*(-[a-z0-9]+)*$`. DSC matches it case-sensitively, as it
  does for built-in commands.
- A plug-in can't use the name or the alias of a built-in command. Built-in commands are matched
  first, so such a plug-in could never run. DSC skips the manifest during discovery with a
  warning, and `dsc plugin list` doesn't list it.
- A built-in command added in a later release takes precedence over a plug-in that already uses
  the name. The changelog calls out the new command as a breaking change for that name, and the
  plug-in stays usable through its own executable.

### Discovery

DSC searches for plug-in manifests in the directories it already searches for resource and
extension manifests: the directories from the `resourcePath` settings and `DSC_RESOURCE_PATH`,
the directory that contains `dsc`, and the directories in `PATH` unless the resource path is
restricted. Plug-in discovery adds no search location and no setting of its own for locations.

Discovery reads manifests and evaluates `condition`. It never runs a plug-in. It only happens
when the result is needed:

| Invocation                                            | Plug-in discovery |
| :---------------------------------------------------- | :---------------: |
| A built-in command, such as `dsc config get`          |        No         |
| `dsc <name>` where `<name>` isn't a built-in command  |        Yes        |
| `dsc --help`, `dsc -h`, and `dsc help` without a name |        Yes        |
| `dsc plugin list`                                     |        Yes        |
| `dsc completer <SHELL>`                               |        Yes        |

Existing automation that calls built-in commands therefore pays no cost and sees no change in
behavior. Discovery extensions aren't asked for plug-in manifests.

DSC applies these rules to what it finds:

1. A manifest that fails to load or validate is skipped with a warning, as other manifests are.
1. When the same `type` is found more than once, DSC keeps the highest version, as it does for
   extensions.
1. When plug-ins of different types use the same command name, the one found first in the search
   order provides the command. Within one directory the type name that sorts first wins. The
   others are shadowed: `dsc plugin list` marks them, and they can only be run by their type
   name, as described in [Dispatch](#dispatch).

### Dispatch

1. DSC parses its own options and the first positional argument. Built-in commands and their
   aliases match first.
1. Any other first positional argument names a plug-in, in one of two ways. DSC keeps every
   argument after it as written, including `--help`, `--version`, and `--`, and doesn't interpret
   them.
   - A command name, like `lint`, runs the plug-in that provides the command. When other plug-ins
     are shadowed for that command, DSC writes a warning that names the plug-in it runs and the
     shadowed plug-ins, and shows how to run each one by its type name.
   - A fully qualified type name, like `OpenDSC/Lint`, runs that plug-in, whether it is shadowed
     or not, and writes no warning. A type name always contains a `/` and a command name never
     does, so the two forms can't be confused with each other or with a built-in command. Type
     names are compared case-insensitively, as everywhere in DSC.
1. When no plug-in matches the name, or plug-ins are disabled, DSC reports the unrecognized
   subcommand error and exits with code `2`, as it does today. The suggestion in the error covers
   built-in commands and plug-in commands.
1. DSC resolves `command.executable`. When it can't be found or started, DSC reports the error
   and exits with code `2`.
1. DSC starts the executable with these properties:
   - The arguments are the items of `command.args` followed by the arguments the user typed after
     the command name.
   - The working directory is the caller's working directory.
   - Standard input, output, and error are inherited. DSC doesn't read, buffer, or reformat what
     the plug-in writes.
   - The environment is the environment of `dsc` plus the variables in
     [Environment](#environment).
1. DSC waits for the plug-in and exits with the plug-in's exit code. When the plug-in is ended by
   a signal, DSC exits with `128` plus the signal number.

Pressing Ctrl+C ends the plug-in together with `dsc`, which exits with code `6` as it does for
every command.

### Environment

DSC sets these variables for the plug-in:

| Variable                   | Value                                                                               |
| :------------------------- | :---------------------------------------------------------------------------------- |
| `DSC_EXE`                  | The absolute path of the `dsc` executable that started the plug-in.                 |
| `DSC_VERSION`              | The semantic version of that `dsc`.                                                 |
| `DSC_PLUGIN_COMMAND`       | The name the plug-in was invoked as: its command name or its type name.             |
| `DSC_PLUGIN_DIR`           | The directory that contains the plug-in manifest.                                   |
| `DSC_TRACE_LEVEL`          | The resolved trace level. DSC already sets it for every child process.              |
| `DSC_TRACE_FORMAT`         | The resolved trace format.                                                          |
| `DSC_PROGRESS_FORMAT`      | The value of `--progress-format`, when the user specified it. This variable is new. |
| `DSC_IGNORE_SETTINGS_FILE` | Set as it is today when the user specified `--ignore-settings-file`.                |

These variables are how the options before the command reach the plug-in. DSC doesn't add them to
the plug-in's arguments, so a plug-in never has to accept options it didn't define. `DSC_EXE` lets
a plug-in call back into the same engine, for example `dsc resource list` or `dsc schema`, which
is how a plug-in that doesn't link `dsc-lib` uses DSC functionality. `DSC_VERSION` lets a plug-in
refuse to run with a `dsc` that is too old for it.

`PATH` is the value DSC already prepares for child processes, which includes the directory that
contains `dsc`. New variables can be added in later releases. A variable is never removed or
given a different meaning without a new RFC.

### Help

- `dsc --help` lists the plug-in commands with their descriptions under a separate **Plug-in
  commands** heading, sorted by name.
- Type names aren't listed in the help or in completion scripts. `dsc plugin list` shows the type
  name of every plug-in.
- `dsc <command> --help` reaches the plug-in unchanged, like every other argument. A type name
  works wherever a command name does, including after `dsc help`.
- `dsc help <command> [<subcommand>...]` runs the plug-in with the subcommands followed by
  `--help`.

A plug-in must print its help and exit with code `0` when it receives `--help`.

### The `dsc plugin list` command

```text
dsc plugin list [<PLUGIN_NAME>] [--output-format <FORMAT>]
```

The command follows `dsc extension list`. `PLUGIN_NAME` filters on the type name and accepts
wildcards. The output formats and their defaults are those of the other `list` commands: a table
on a terminal and one JSON object per line otherwise.

```json
{
  "type": "Microsoft.DSC/Lint",
  "version": "3.4.0",
  "command": "lint",
  "description": "Check configuration documents for problems without applying them",
  "deprecationMessage": null,
  "path": "/opt/dsc/dsclint.dsc.plugin.json",
  "directory": "/opt/dsc",
  "shadowed": false,
  "manifest": {}
}
```

`shadowed` is `true` when another plug-in found earlier provides the same command. A shadowed
plug-in still runs when it is invoked by its `type`, like `dsc OpenDSC/Lint`. `manifest`
holds the manifest as DSC read it. The object is published as a schema under
`outputs/plugin/list.json`.

### Settings

A `plugins` section is added to the DSC settings:

```json
{
  "plugins": {
    "enabled": true
  }
}
```

`plugins.enabled` defaults to `true`. When it is `false`, DSC doesn't discover plug-ins, an
unknown command fails with the unrecognized subcommand error, `dsc --help` has no **Plug-in
commands** section, and `dsc plugin list` returns nothing. The setting follows the existing
precedence of settings files, so a value in the policy file `dsc.policy.json` can't be
overridden. This gives an administrator one switch to keep a server to the built-in commands.

`resourcePath.restricted` and `resourcePath.directories` already limit where manifests are found,
and they limit plug-ins in the same way.

### Shell completion

`dsc completer <SHELL>` generates a static script from the command tree. This RFC extends it in
three steps.

#### Plug-in command names

Before it generates the script, DSC adds every discovered plug-in command with its description to
the command tree. Every shell `dsc completer` supports (Bash, Elvish, fish, PowerShell, and Zsh)
then completes plug-in command names. The script is static: after a plug-in is installed or
removed, it has to be generated again. Generating it when the shell starts, as the PowerShell
example in [Complete plug-in commands](#complete-plug-in-commands) does, keeps it current.

#### Completion spec

To get its options and values completed, a plug-in ships a completion spec and names it in the
manifest with `completion.spec`, a path relative to the manifest. The file uses the
[carapace command spec][05] format: YAML that declares a command's flags, its nested commands,
and the values to complete.

```yaml
# yaml-language-server: $schema=https://carapace.sh/schemas/command.json
name: lint
description: Check configuration documents for problems without applying them
flags:
  -f, --file=*: A document to lint
  -i, --input=: The document as a string
  --no-discovery: Skip resource discovery and the rules that need it
  --fail-on=: The lowest level that makes the command exit with code 5
  -o, --output-format=: The output format to use
exclusiveflags:
  - [file, input]
completion:
  flag:
    file: ["$files([.json, .yaml, .yml])"]
    fail-on: [error, warning, info]
    output-format: [json, pretty-json, yaml]
```

DSC adopts this format instead of defining one for the following reasons:

- It has a published [JSON schema][06] and covers flags, nested commands, and values, which is
  what [#461][01] tried to sketch for a manifest and left open.
- It can be generated from the argument definitions of clap, cobra, kong, urfave/cli, click, and
  oclif, so most authors don't write it by hand. The spec for `dsclint` is generated from its
  clap definition during the build, for example with [carapace-spec-clap][07].
- The same file serves carapace users directly.

DSC replaces the `name` at the root of the spec with the command name from the manifest. The spec
is read only by `dsc completer`. It plays no part in dispatch, and DSC never validates the
arguments of a plug-in against it.

#### Native scripts

For the shells it supports itself, DSC translates the static part of each spec into the command
tree before it generates the script:

| Spec element                                                      | Used for native scripts                                      |
| :---------------------------------------------------------------- | :----------------------------------------------------------- |
| `name`, `aliases`, `description`, `hidden`                        | Yes                                                          |
| `commands` (nested, recursive)                                    | Yes                                                          |
| `flags`, `persistentflags` with the modifiers `=` `*` `?` `&` `!` | Yes, for flags in the `-s`, `--long`, and `-s, --long` forms |
| Literal values in `completion.flag`, with descriptions            | Yes                                                          |
| `$files`, `$directories`, and `$executables` in `completion.flag` | Yes, as file, directory, and command completion              |
| Other macros, such as `$(...)` and `$carapace.*`                  | No. The shell falls back to its default completion.          |
| `completion.positional`, `positionalany`, `dash`, `dashany`       | Literal values only                                          |
| `exclusiveflags`, `run`, `parsing`, `documentation`, `examples`   | No                                                           |

Spec elements DSC doesn't recognize are ignored, so a spec that uses more of the format still
works.

#### `dsc completer carapace`

`carapace` is added as a value of the `SHELL` argument. The command writes one carapace spec for
`dsc`: the built-in command tree, plus one entry under `commands` per plug-in.

- For a plug-in with a completion spec, the entry is that spec, complete with its macros.
- For a plug-in without one, the entry delegates to a completer named `dsc-<command>` and falls
  back to file completion. This is [the convention carapace uses][08] for the plug-ins of `git`,
  `gh`, and `cargo`, so a user can still add a spec of their own for that plug-in.

The user saves the output as `dsc.yaml` in the carapace user spec directory. Carapace then
completes `dsc` in every shell it supports, which adds Nushell, Xonsh, Oil, and others to the
shells above, and it evaluates the dynamic macros that the native scripts skip.

DSC has no runtime dependency on carapace. A user who doesn't install it loses nothing they have
today.

### Security

- **Built-in commands can't be replaced.** They are matched before any plug-in, and a manifest
  that claims a built-in name is skipped. Automation that calls `dsc config` or `dsc resource`
  can't be redirected by installing a plug-in.
- **No new trust boundary.** A plug-in runs with the caller's privileges, like any executable.
  DSC runs only what a manifest in the resource path names, which is the boundary resources and
  extensions have today. The difference favors plug-ins: an extension with the `discover`
  capability runs during every resource operation, and a plug-in runs only when its command is
  typed.
- **No search in the working directory.** DSC searches the resource path. The working directory
  is only searched when the user put it on `PATH`.
- **Listing doesn't execute anything.** `dsc --help`, `dsc plugin list`, and `dsc completer` read
  manifests and spec files. They never start a plug-in.
- **Administrators keep control.** `plugins.enabled` and `resourcePath.restricted` in the policy
  file remove plug-ins, or limit them to chosen directories.
- **Trust for artifacts applies.** The trust RFC in [#1687][09] covers manifests and the
  executables they name. When it is implemented, plug-in manifests fall under the same policy,
  and `dsc plugin list` reports the trust property that RFC adds to `dsc resource list`.
- **Completion specs can name commands.** A carapace spec can contain macros that run a command
  when the user presses the Tab key. DSC's native scripts never run them. The output of
  `dsc completer carapace` carries them into the user's carapace configuration, so the existing
  advice to review the output of `dsc completer` before using it applies.

### `dsclint` as the first plug-in

- The DSC packages add `dsclint.dsc.plugin.json` and `dsclint.completion.yaml` next to the
  `dsclint` executable that [#1768][02] already ships.
- `dsc lint <arguments>` is equivalent to `dsclint <arguments>`. The `dsclint` executable stays
  directly usable.
- `dsclint` keeps its own `--trace-level`, `--trace-format`, `--progress-format`, and
  `--ignore-settings-file` options for standalone use. When it is started through `dsc`, the
  environment variables provide the defaults, and an option given after `lint` takes precedence.
- The reference documentation presents `dsc lint` as the command and `dsclint` as its executable.

### Guidelines for plug-in authors

These aren't enforced, but a plug-in that follows them feels like a part of `dsc`:

- When `DSC_PLUGIN_COMMAND` is set, show the usage in help as `dsc <command>`.
- Write results to standard output and messages to standard error. Honor `DSC_TRACE_LEVEL` and
  `DSC_TRACE_FORMAT` for the messages.
- Offer `--output-format` with the values `json`, `pretty-json`, and `yaml` when the result is
  structured.
- Use the DSC exit codes where their meaning fits: `0` for success, `1` for invalid arguments,
  `2` for a failure, and `4` for invalid input.
- Use `DSC_EXE` to call DSC instead of searching `PATH` for `dsc`.

### Implementation phases

1. The manifest and its schema, discovery, dispatch, the environment contract, help,
   `dsc plugin list`, the `plugins.enabled` setting, plug-in command names in `dsc completer`,
   and the manifest for `dsclint`.
1. Completion specs: `completion.spec`, the translation for native scripts, and
   `dsc completer carapace`.

The first phase stands on its own. The second can change in response to the open question about
the spec format without affecting plug-ins that already exist.

## Alternate proposals and considerations

### Prior art

Every command-line tool surveyed dispatches an unknown first argument to something outside its
own executable, and none lets a plug-in replace a built-in command. They differ in how they find
plug-ins, what they know about one before running it, and how the arguments of a plug-in get
completed:

| Tool             | Finds plug-ins by                                          | Metadata before running          | Completion of plug-in arguments                               |
| :--------------- | :--------------------------------------------------------- | :------------------------------- | :------------------------------------------------------------ |
| [git][10]        | `git-<name>` in its exec path, then `PATH`                 | None                             | A shell function `_git_<name>` that the plug-in installs      |
| [dotnet][11]     | `dotnet-<name>` through a local tool manifest, then `PATH` | None for a tool found on `PATH`  | A separate tool, `dotnet-suggest`                             |
| [cargo][12]      | `cargo-<name>` in `$CARGO_HOME/bin`, then `PATH`           | None                             | A Zsh function `_cargo-<name>`; names only in other shells    |
| [kubectl][13]    | `kubectl-<name>` on `PATH`                                 | None                             | A second executable, `kubectl_complete-<name>`                |
| [GitHub CLI][14] | `gh-<name>` in its own extensions directory                | None from the author             | None                                                          |
| [Docker CLI][15] | `docker-<name>` in fixed `cli-plugins` directories         | JSON the plug-in prints when run | The plug-in answers the host's completion request             |
| [Helm][16]       | A `plugin.yaml` manifest in its plug-in directories        | The manifest                     | A static `completion.yaml`, or a `plugin.complete` executable |

The [Azure CLI][17] is the exception to both statements: its extensions are Python packages
loaded in process, and they may replace a core command, with a warning.

This RFC takes the following from them:

- Built-in commands always win, as in every tool in the table.
- Arguments, the standard streams, and the exit code pass through unchanged, as in git, cargo,
  and dotnet.
- Options the host parsed reach the plug-in as environment variables, as in git and Helm.
- The plug-in learns the path of its host (`CARGO`, `GH_PATH`, `HELM_BIN`) and its own directory
  and name (`HELM_PLUGIN_DIR`, `HELM_PLUGIN_NAME`).
- Help has a separate group for plug-in commands, as in kubectl and the GitHub CLI, and
  `help <command>` runs the plug-in with `--help`, as in cargo and Docker.
- A second plug-in with the same name is reported as shadowed, as `kubectl plugin list` does,
  and stays reachable by an unambiguous name, as `gh extension exec` allows.
- A manifest carries the metadata and a static file next to it carries the completions, read only
  when completions are generated, as in Helm.

### Alternatives

- **A file name convention instead of a manifest.** Git, dotnet, cargo, and kubectl run any
  `<tool>-<name>` executable on `PATH`, which needs no file besides the executable. It was
  considered and not chosen as the mechanism for DSC for these reasons:
  - DSC already ships `dsc-bicep-ext`, a gRPC server, which would become the command
    `dsc bicep-ext`. Every future helper executable would need a name that avoids the prefix.
  - A name carries no description, so `dsc --help` could list names only, or would have to run
    every candidate to ask, as Docker does.
  - On Windows a script isn't directly executable. A manifest names the interpreter, which matters
    for a community that writes much of its tooling in PowerShell.
  - DSC already discovers by manifest, and the resource path settings, the policy file, and the
    planned trust model apply to manifests without new rules.
  - `dsclint` keeps its name.

  The convention can be added later as a fallback behind manifests if there is demand for it.
- **A command capability in the extension manifest.** An earlier comment in [#461][01] suggested
  another kind of extension, and the latest suggests a separate `.dsc.plugin.json`. This RFC
  follows the latest. DSC discovers extensions during resource operations and may call them
  without the user asking, while a plug-in is looked up only when its command is typed. Keeping
  the kinds apart keeps plug-in definitions out of every resource operation and keeps the two
  contracts simple.
- **Declaring arguments in the manifest.** [#461][01] sketched `args` and `argSets`, or JSON
  schemas for input and output, from which DSC would build the command line. That gives uniform
  help and validation, but it turns DSC into a command-line framework and limits a plug-in to what
  the manifest can express. Plug-ins parse their own arguments, and the completion spec covers
  discoverability.
- **Several commands per manifest.** A map of commands would let one manifest add `dsc package`
  and `dsc manifest`. One command per manifest keeps the identity, version, and shadowing rules
  simple, and a manifest list covers a bundle.
- **Hooks into existing commands.** [#461][01] also describes cache and resolver hooks that DSC
  calls during `dsc config`. Those are extension capabilities, in the same family as `discover`
  and `import`, and aren't part of this RFC.
- **An installer, such as `dsc plugin install`.** The GitHub CLI, Helm, dotnet, and the Azure CLI
  have one. DSC has no installer for resources or extensions either. Plug-ins are distributed the
  way those are, and a future package story should cover all three.
- **Dynamic completion.** kubectl, Docker, and Helm can ask a plug-in for completions while the
  user types. For `dsc` that needs clap's dynamic completion engine, which is still behind the
  `unstable-dynamic` feature ([clap#3166][18]), and it runs plug-ins on every press of the Tab
  key. The static approach can be replaced when that engine is stable. The manifest doesn't
  change.
- **Another completion format.** Other formats were only surveyed for this RFC. They weren't
  evaluated in the same depth as carapace:
  - The Fig completion spec, which Microsoft's `inshellisense` consumes. A spec is a TypeScript
    module. `inshellisense` is maintained and completes in Windows PowerShell as well as in
    PowerShell 7, which carapace doesn't. The tool the format was created for is no longer
    actively maintained.
  - The `usage` spec, a KDL document with an exporter for clap. It generates completions for
    Bash, Zsh, fish, PowerShell, and Nushell, and the generated scripts need the `usage` tool on
    `PATH`.
  - Helm's `completion.yaml`, which is specific to Helm.

  The carapace spec is proposed because it was raised in the working group, it is YAML with a
  published JSON schema, and generators exist for common command-line frameworks. Whether one of
  the other formats fits DSC better hasn't been established. That is part of the open question
  about the format.
- **Requiring carapace.** Carapace could replace `dsc completer` entirely. It needs a separate
  binary, [it requires PowerShell 7][19] while DSC users also run Windows PowerShell 5.1, and it
  has [no machine-wide directory for specs][20] that an installer could write to. It stays
  optional.
- **Moving built-in commands out.** `completer`, `server`, and `schema` could become plug-ins to
  shrink the engine further. This RFC doesn't propose it.
- **Further ideas left out of the first version.** A property that states the `dsc` versions a
  plug-in requires, like the
  minimum version of the Azure CLI extensions. An allow list of plug-in types in the settings.
  Plug-ins found through discovery extensions. Plug-in commands as tools of the MCP server.

### Risks of the carapace spec format

- The format has no version field and its schema URL isn't versioned. It has changed
  incompatibly before, for example when its macro prefix was renamed.
- The schema doesn't allow additional properties, so DSC can't add keys of its own to a spec.
- A spec describes flags and nested commands well. It doesn't describe named positional arguments
  or the types of values.
- The project appears to depend largely on a single maintainer.

DSC limits the exposure by reading a small, documented subset, by ignoring what it doesn't
recognize, and by keeping the spec out of dispatch. If the format has to be replaced, a `format`
property next to `completion.spec` can select another one, and a manifest without it keeps meaning
carapace.

### Open questions

1. Does the working group want the `dsc-<name>` file name convention as a fallback in the first
   version? This RFC leaves it out for the reasons under [Alternatives](#alternatives).
1. Is the carapace spec the format the working group wants in a DSC manifest, given the risks
   above? The alternatives for the second phase are a minimal format owned by DSC, or one of the
   formats under [Alternatives](#alternatives), which this RFC surveyed but didn't compare in
   depth.
1. Should DSC skip plug-in manifests in directories that non-administrators can write to when it
   runs elevated, using the path permission check from [#1760][21]? The same question exists for
   resource and extension manifests, so the answer should be the same for all three.
1. Is `DSC_PROGRESS_FORMAT` acceptable as a new environment variable, or should
   `--progress-format` before a plug-in command be an error?

## Related work items

- [#461][01] - Asks for plug-ins for `dsc`. This RFC specifies the command plug-ins from that
  discussion and leaves the cache and resolver hooks to extensions.
- [#1768][02] - Adds the `dsclint` executable, the first plug-in.
- [#1766][03] - The RFC for linting. Its command becomes the plug-in command `dsc lint`.
- [#591][22] - The original request for `dsc lint`, where the question of extension or core was
  first raised.
- [#1687][09] - The RFC for trust in DSC artifacts, which covers plug-in manifests and
  executables.
- [#1760][21] - The path permission check that the open question about elevated runs refers to.

[01]: https://github.com/PowerShell/DSC/issues/461
[02]: https://github.com/PowerShell/DSC/pull/1768
[03]: https://github.com/PowerShell/DSC/pull/1766
[04]: https://carapace.sh/
[05]: https://carapace-sh.github.io/carapace-spec/
[06]: https://carapace.sh/schemas/command.json
[07]: https://github.com/carapace-sh/carapace-spec-clap
[08]: https://github.com/carapace-sh/carapace-bin/blob/master/docs/src/spec/embed.md
[09]: https://github.com/PowerShell/DSC/pull/1687
[10]: https://git-scm.com/docs/git
[11]: https://learn.microsoft.com/dotnet/core/tools/global-tools
[12]: https://doc.rust-lang.org/cargo/reference/external-tools.html
[13]: https://kubernetes.io/docs/tasks/extend-kubectl/kubectl-plugins/
[14]: https://cli.github.com/manual/gh_extension
[15]: https://github.com/docker/cli/tree/master/cli-plugins
[16]: https://helm.sh/docs/topics/plugins/
[17]: https://learn.microsoft.com/cli/azure/azure-cli-extensions-overview
[18]: https://github.com/clap-rs/clap/issues/3166
[19]: https://github.com/carapace-sh/carapace-bin/issues/2112
[20]: https://github.com/carapace-sh/carapace-bin/issues/1336
[21]: https://github.com/PowerShell/DSC/issues/1760
[22]: https://github.com/PowerShell/DSC/issues/591
