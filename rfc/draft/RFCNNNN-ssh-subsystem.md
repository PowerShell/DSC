---
RFC:          RFCNNNN # WG will set the number after submission
Author:       "@Gijsreyn"
Sponsor:      null    # <@GitHubUserName>
Status:       Draft   # <Draft | Experimental | Accepted | Final>
SupercededBy: null    # <Superceding RFC Number>
Version:      0.1     # <Major>.<Minor>
Area:         Remoting
CommentsDue:  2026-12-15
---

# DSC as an SSH Subsystem

Allow `dsc` to apply a configuration to another machine by talking to a `dsc server` process that
the target's SSH daemon starts as a subsystem. The existing MCP server is the remote endpoint,
OpenSSH is the transport, and the existing `dsc config` commands gain a `--target` option.

## Motivation

> As an operator running a deployment pipeline,
> I want to push a configuration document from a runner to a machine with one `dsc` command,
> so that I can apply and validate changes immediately without logging on to each machine.

> As an administrator of a managed machine,
> I want remote DSC to use the SSH service I already secure and audit,
> so that I do not have to open another port, issue another certificate or maintain another
> access list.

DSC v3 has no remote story today. `dsc` runs against the local machine only, and users who want to
manage another machine copy the document over and run `dsc` in a remote shell. That works, but
every team solves quoting, file transfer, secret handling and output parsing again.

The pieces for a better experience already ship with DSC:

- `dsc server` is a server mode that accepts a configuration document and parameters as inline
  strings and returns the same result objects as the CLI.
- `Microsoft.OpenSSH.SSHD/Subsystem` can register a subsystem in `sshd_config`.
- OpenSSH is available on every platform DSC supports and already handles authentication,
  authorization, encryption, host verification, jump hosts and enterprise PKI.

This RFC connects them. It covers pushing a document directly to one target. Pull models, and a
push that only tells the target to pull, are out of scope and discussed under
[Alternate Proposals and Considerations](#alternate-proposals-and-considerations).

## Proposed experience

### Preparing a target

DSC ships a configuration document that registers the subsystem. An administrator applies it once
on the target from an elevated session and then restarts the SSH service:

```powershell
dsc config set --file "$(Split-Path (Get-Command dsc).Source)/ssh_subsystem.dsc.yaml"
Restart-Service sshd      # Linux: systemctl restart sshd
```

The result is one line in `sshd_config`:

```text
Subsystem dsc "/usr/bin/dsc" --trace-format json --trace-level warn server
```

### Pushing a configuration

The operator adds `--target` to the command they already use locally. The document and parameter
files are read on the operator's machine. Nothing needs to exist on the target except `dsc`.

```powershell
dsc --target ssh://admin@web01:2222 config set --file web.dsc.yaml --parameters-file prod.yaml
```

```yaml
metadata:
  Microsoft.DSC:
    version: 3.5.0
    operation: set
    executionType: actual
    securityContext: elevated
    # ... same fields as a local run, reported by the target
results:
- name: Web tier marker
  type: Microsoft.Windows/Registry
  result:
    beforeState:
      keyPath: HKLM\Software\Contoso\Web
      _exist: false
    afterState:
      keyPath: HKLM\Software\Contoso\Web
    changedProperties:
    - _exist
messages: []
hadErrors: false
```

The output is what the target produced, printed through the local `--output-format`. The other
configuration operations work the same way:

```powershell
dsc --target ssh://admin@web01 config test --file web.dsc.yaml --output-format json
dsc --target ssh://admin@web01 config set --file web.dsc.yaml --what-if
dsc --target ssh://admin@web01 config export --file inventory.dsc.yaml
Get-Content web.dsc.yaml | dsc --target ssh://web01 config get --file -
```

Authentication is whatever `ssh admin@web01` would use for that user: keys, an agent, certificates,
`~/.ssh/config` host aliases and jump hosts all apply unchanged.

Traces from the target appear on the local stderr, attributed to the target:

```text
2026-11-12T10:15:02.118Z  WARN web01: Adapted resource manifest 'C:\...\WinGetPackage.dsc.adaptedResource.json' uses the deprecated schema URI ...
```

Connection problems are reported once and use their own exit code:

```text
2026-11-12T10:15:00.431Z ERROR Could not connect to 'ssh://admin@web01:2222': subsystem request failed on channel 0
```

## Specification

### Components

| Component                                 | Status                          | Role                                                                  |
|-------------------------------------------|---------------------------------|-----------------------------------------------------------------------|
| `sshd` on the target, `ssh` on the client | Exists (OpenSSH)                | Transport, authentication, authorization, encryption.                 |
| `dsc server`                              | Exists (`dsc/src/server`)       | Remote endpoint. Speaks MCP (JSON-RPC 2.0) on stdin/stdout.           |
| `invoke_dsc_config` tool                  | Exists                          | Runs get, set, test or export for an inline document and parameters.  |
| `Microsoft.OpenSSH.SSHD/Subsystem`        | Exists (`resources/sshdconfig`) | Writes the `Subsystem` line and validates the file with `sshd -T`.    |
| `ssh_subsystem.dsc.yaml`                  | New, shipped with DSC           | Registers the subsystem using the resource above.                     |
| `--target` and the remote client          | New, in `dsc`                   | Reads input locally, starts `ssh`, calls the tool, prints the result. |
| Server identity and error data            | Changed, in `dsc server`        | Lets a client check the remote version and reproduce exit codes.      |

The client uses the client side of `rmcp`, the crate that already implements the server, over the
stdin and stdout of an `ssh` child process. No SSH or cryptography library is added to DSC.

```text
operator machine                               target
┌──────────────────────────┐                   ┌────────────────────────────────────┐
│ dsc --target ssh://...   │                   │ sshd                               │
│   reads file, parameters │                   │   authenticates the user           │
│   spawns: ssh -s ... dsc │ ═══ SSH channel ══│   starts subsystem "dsc":          │
│   MCP client on its      │   stdin / stdout  │     dsc ... server                 │
│   stdin and stdout       │   stderr (traces) │       invoke_dsc_config            │
│   prints the result      │                   │         Configurator, resources    │
└──────────────────────────┘                   └────────────────────────────────────┘
```

### The `--target` option

A new global option is added next to `--trace-level`, `--trace-format` and `--progress-format`:

```text
dsc [--target <URI>] <subcommand> ...
```

The value is a URI with the following form:

```text
ssh://[<user>@]<host>[:<port>]
```

- `host` is a DNS name, an IPv4 address, a bracketed IPv6 address, or a `Host` alias from the
  user's SSH configuration.
- `user` and `port` are optional. When omitted, the SSH client's own defaults and configuration
  apply.
- A path, query, fragment or password in the URI is an error.
- `ssh` is the only scheme in this version. Any other scheme fails with exit code `1` and a message
  that names the supported schemes. The scheme is what a later transport extension keys on.

`--target` is valid with these commands and options:

| Command         | Options honored                                                             |
|-----------------|-----------------------------------------------------------------------------|
| `config get`    | `--input`, `--file`, `--parameters`, `--parameters-file`, `--output-format` |
| `config set`    | The same, plus `--what-if`                                                  |
| `config test`   | The same as `config get`                                                    |
| `config export` | The same as `config get`                                                    |

Every other use of `--target` is rejected with exit code `1` before any connection is made. This
includes `config --system-root`, `resource`, `extension`, `function`, `schema`, `server`,
`completer` and the hidden `--as-group`, `--as-assert` and `--as-include` options. The commands are
limited on purpose: the server's resource tools do not behave exactly like the `dsc resource`
commands today (see [Open Questions](#open-questions)).

`--progress-format` has no effect on a remote operation in this version.

### Client behavior

For a supported command with `--target`, `dsc` performs these steps:

1. Read the document and parameters exactly as a local run does: `--input`, `--file` (including
   `-` for stdin and any installed import extension), `--parameters` and `--parameters-file`, and
   merge the two parameter sources. Input errors keep their current exit codes.
2. Start the system SSH client:

   ```text
   ssh -s [-p <port>] [-l <user>] -- <host> dsc
   ```

   `-s` requests the subsystem named `dsc`. The executable is `ssh` resolved from `PATH`.
3. Send the MCP `initialize` request and check the response (see
   [Version check](#version-check)).
4. Send one `tools/call` request for `invoke_dsc_config`.
5. Print the result through the existing output writer, close the child's stdin, wait for `ssh` to
   exit and return the exit code defined under [Errors and exit codes](#errors-and-exit-codes).

One command is one SSH session and one tool call. The client does not keep connections open
between commands.

The tool call maps from the command line as follows:

| Argument        | Value                                                                                                       |
|-----------------|-------------------------------------------------------------------------------------------------------------|
| `operation`     | `get`, `set`, `test` or `export`.                                                                           |
| `configuration` | The document as JSON text, after local import and normalization.                                            |
| `parameters`    | The merged parameters as JSON text with the top-level `parameters` wrapper, or omitted when there are none. |
| `what_if`       | `true` when `--what-if` is given, otherwise omitted.                                                        |

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "invoke_dsc_config",
    "arguments": {
      "operation": "set",
      "configuration": "{\"$schema\":\"https://aka.ms/dsc/schemas/v3/bundled/config/document.json\",\"resources\":[]}",
      "parameters": "{\"parameters\":{\"environment\":\"prod\"}}",
      "what_if": true
    }
  }
}
```

The client prints `structuredContent.result` from the tool response. For `export` it prints the
`result` member of that object, which is what `dsc config export` prints locally. The client
passes the JSON through without converting it into its own result types, so a newer target can
return fields an older client does not know.

#### Connection rules

- **No option injection.** The client rejects a `user` or `host` that starts with `-` or contains
  whitespace or control characters, and always passes `--` before the host. A target value is
  never able to add an option to the `ssh` command line.
- **IPv6.** Brackets are removed before the address is passed to `ssh`.
- **Interactive prompts.** `ssh` prompts for passwords and host key confirmation on the terminal,
  not on stdin, so they keep working. Unattended callers must use non-interactive authentication.
  `dsc` does not add or override any SSH option.
- **Handshake timeout.** If no valid `initialize` response arrives within 60 seconds of starting
  `ssh`, the client terminates the child and fails with exit code `11`.
- **Noise on stdout.** Lines on the channel that are not JSON-RPC messages, for example output
  from a shell profile on the target, are ignored before the `initialize` response. After it, they
  are logged at `debug` and ignored.
- **Message size.** `dsc server` drops an incoming message larger than 16 MiB without answering.
  The client therefore refuses to send a larger request and fails with exit code `4`. The client
  accepts responses up to 64 MiB.

### Target registration

DSC ships `ssh_subsystem.dsc.yaml` in the same directory as `dsc` on all platforms:

```yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
metadata:
  Microsoft.DSC:
    securityContext: elevated
parameters:
  dscPath:
    type: string
    defaultValue: "[tryWhich('dsc')]"
    metadata:
      description: Full path of the dsc executable that sshd starts.
  traceLevel:
    type: string
    defaultValue: warn
    allowedValues:
    - error
    - warn
    - info
    metadata:
      description: Highest trace level the target sends to remote clients.
resources:
- name: DSC subsystem
  type: Microsoft.OpenSSH.SSHD/Subsystem
  properties:
    subsystem:
      name: dsc
      value: >-
        [concat('"', parameters('dscPath'), '" --trace-format json --trace-level ', parameters('traceLevel'), ' server')]
```

- The subsystem name is `dsc`. Clients always request that name.
- `dscPath` defaults to the `dsc` found on `PATH` of the session that applies the document. An
  administrator passes it explicitly when `dsc` is not on `PATH`.
- The path is written in double quotes. With OpenSSH for Windows 9.5, a subsystem whose unquoted
  path contains a space (`C:\Program Files\...`) fails to start, and the quoted form works.
- `--trace-format json` is required so the client can parse remote traces.
- The document does not restart the SSH service. The subsystem is available after the
  administrator restarts or reloads it.
- Removing the subsystem is the same resource instance with `_exist: false`.

No DSC setting, port, certificate or service is created. Whether a user may reach the subsystem is
decided entirely by `sshd` configuration.

### Server changes

`dsc server` keeps its command line, transport and tools. Two additions are required. Both are
visible to existing MCP clients and are backward compatible.

#### Server identity

The `initialize` result reports DSC instead of the MCP library. Today it reports
`{"name": "rmcp", "version": "3.8.0"}`, which tells a client nothing about the target.

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "serverInfo": { "name": "dsc", "version": "3.5.0" },
    "capabilities": { "tools": {} }
  }
}
```

`version` is the semantic version that `dsc --version` prints. Other members of the result are
unchanged and omitted from the example.

#### Error data

When a tool call fails, the JSON-RPC error carries the exit code that the local CLI returns for
the same failure:

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "error": {
    "code": -32600,
    "message": "Failed to set parameters: Parameter 'environment' is not defined",
    "data": { "exitCode": 4 }
  }
}
```

```json
{
  "type": "object",
  "required": ["exitCode"],
  "properties": {
    "exitCode": {
      "type": "integer",
      "minimum": 1,
      "maximum": 255,
      "description": "The exit code the dsc CLI returns for this failure when run locally."
    }
  }
}
```

Without this, a client can only tell "invalid request" from "internal error" and cannot reproduce
codes such as `5` (validation failed) or `7` (resource not found).

### Version check

After `initialize`, the client requires:

- `serverInfo.name` equal to `dsc`, and
- `serverInfo.version` equal to or higher than the first DSC release that implements this RFC.

If either check fails, the client closes the connection without calling a tool and fails with
exit code `11` and a message that states the version found and the version required.

The check is mandatory because of how older servers treat unknown input. A server that predates
the `what_if` argument ignores it, so `config set --what-if` against such a target would apply the
changes for real. Refusing is the only safe behavior.

A target newer than the client is accepted.

### Tracing

- The target writes trace lines as JSON on stderr, at the level fixed in the subsystem line.
  `sshd` forwards them on the channel and the local `ssh` writes them on its stderr.
- The client reads the child's stderr while the operation runs. A line that is a DSC trace is
  re-emitted through the client's own tracing at the same level, prefixed with the target host.
  The client's `--trace-level` and `--trace-format` then apply as usual. This is the mechanism DSC
  already uses when it starts `dsc` for a group resource.
- The client's `--trace-level` cannot raise the target above the level in its subsystem line.
- Any other stderr line comes from `ssh` itself, for example a host key warning or an
  authentication failure. These lines are logged at `debug` as they arrive and included in the
  error message if the connection fails.
- The default level in the shipped document is `warn` and the allowed values stop at `info`. At
  `debug` and `trace`, DSC and the MCP library log complete requests and resource input, which
  includes secure parameter values.

Remote progress reporting is not part of this version.

### Errors and exit codes

One new exit code is added:

| Code | Constant | Meaning |
| ---- | -------- | ------- |
| `11` | `EXIT_REMOTE_FAILED` | The remote endpoint could not be reached, did not complete the handshake, failed the version check, or closed the connection before answering. |

The client derives its exit code in this order:

| Condition                                                                                    | Exit code                                   |
|----------------------------------------------------------------------------------------------|---------------------------------------------|
| Local argument or input error before connecting                                              | Unchanged (`1`, `3` or `4`).                |
| `ssh` not found, `ssh` exits before a tool response, handshake timeout, version check failed | `11`.                                       |
| Tool error with `data.exitCode`                                                              | That code.                                  |
| Tool error without `data.exitCode`                                                           | `2`.                                        |
| Tool result with `hadErrors: true`                                                           | `2`, after printing the result, as locally. |
| Tool result otherwise                                                                        | `0`.                                        |
| Ctrl+C on the client                                                                         | `6`.                                        |

A tool error's `message` is logged at `error`. `config test` returns `0` when the target is not in
the desired state, as it does locally; the state is in the result.

### Security

- **Authentication and authorization** are performed by `sshd`. DSC adds no credential store and
  no access list. Anyone who may open an SSH session as a user can already run `dsc` as that user,
  so the subsystem grants nothing new. Administrators restrict it with the usual `sshd` controls.
- **Identity.** Operations run as the authenticated SSH user. DSC does not elevate. A resource or
  document that requires an elevated security context fails on the target with the same error as
  a local run by that user.
- **Where expressions run.** The whole document is evaluated on the target. `secret()`,
  `envvar()`, `systemRoot()` and every other function read the target's environment, extensions
  and vaults, not the client's.
- **Parameters** are sent inline over the encrypted channel, including values for `secureString`
  and `secureObject` parameters. They are never placed on a command line.
- **Traces.** The shipped trace level keeps request bodies out of the trace stream (see
  [Tracing](#tracing)). An administrator who edits the subsystem line to `debug` or `trace`
  exposes parameter values to every remote caller's stderr.
- **Host verification** is the SSH client's. `dsc` never disables or relaxes it.

### Behavior that differs from a local run

- **Included files.** A `Microsoft.DSC/Include` instance with a `configurationFile` or
  `parametersFile` path is resolved on the target, relative to the working directory `sshd` gives
  the subsystem. `DSC_CONFIG_ROOT` is not set. When the document sent contains a
  `Microsoft.DSC/Include` instance, the client logs a warning that says so. Documents that use
  `configurationContent` are unaffected.
- **Cancellation.** Ctrl+C, a client timeout or a dropped connection ends the client. It does not
  stop an operation the target has already started; the target runs it to completion and discards
  the result. After an interrupted `set`, run `config test` to learn the target's state.
- **Startup cost.** Every command starts a new `dsc` process on the target, which performs its own
  resource discovery.

### Packaging

- `ssh_subsystem.dsc.yaml` is added as a configuration project that is copied on all platforms and
  listed in the package file lists for every package type.
- The `rmcp` `client` feature is enabled for the `dsc` crate. No new third-party crate is
  introduced; URI parsing uses the `url` crate that is already a workspace dependency.

## Open Questions

- **Includes.** Should the client replace literal `configurationFile` and `parametersFile` paths
  with `configurationContent` and `parametersContent` before sending, so that a repository
  checked out on a runner works unchanged? Paths written as expressions cannot be resolved on the
  client.
- **Cancellation.** Should `dsc server` honor MCP cancellation and stop between resource
  instances? This needs the configurator to observe a cancellation signal.
- **Restart signal.** Should `Microsoft.OpenSSH.SSHD/Subsystem` return `_restartRequired` for the
  `sshd` service when it changes the file, so the registration document can report that a restart
  is pending?
- **Elevation on Linux and macOS.** A non-root SSH user cannot apply resources that need root.
  Options are documenting root login with keys, or a second subsystem line that starts `dsc`
  through `sudo` for users with a matching sudoers rule.
- **Remote trace level.** Should a client be able to ask for more detail, through `sshd`
  `AcceptEnv` for `DSC_TRACE_LEVEL` or through MCP logging notifications, given the secret
  exposure described under [Security](#security)?
- **Quoted subsystem line.** The registration document quotes the `dsc` path, which the prototype
  showed is needed on Windows. Still to confirm: that `Microsoft.OpenSSH.SSHD/Subsystem` reads a
  quoted value back unchanged, so the document stays idempotent, and that the line works with
  PowerShell as the OpenSSH default shell.
- **Resource commands.** `dsc resource set` skips the test and routes `_exist: false` to delete,
  while the `invoke_dsc_resource` tool does neither, and `dsc resource list` returns more than
  `list_dsc_resources`. Should the tools be aligned so `resource` commands can accept `--target`?
- **Handshake timeout.** Is a fixed 60 seconds right, or should it be a DSC setting?
- **Experimental marking.** DSC has no experimental-feature switch. Should this ship as
  experimental under the RFC process, and how would users see that?

## Alternate Proposals and Considerations

### Embedding an SSH library

A Rust SSH implementation would remove the dependency on an installed `ssh` client. It would also
move key handling, host verification, agent and configuration parsing, and cryptography into DSC,
and users would lose their existing SSH configuration. PowerShell remoting over SSH made the same
choice as this RFC and starts the system client. Rejected.

### An HTTPS listener in `dsc server`

`dsc server` could listen on a port. DSC would then own certificate provisioning and rotation,
authentication, authorization and a long-running service on every target, which is the work SSH
already does. It remains possible later as another scheme for `--target` without changing the
operation contract.

### Running `dsc` as a remote command

`ssh host dsc config set --input '<document>'` needs no subsystem. The document then passes through
the target's default shell, which differs between `cmd`, PowerShell and POSIX shells, is limited by
command-line length, and appears in process listings together with any parameters. A subsystem
gives the same stdin and stdout on every platform. Rejected as the supported path.

### A DSC-specific protocol instead of MCP

A small protocol such as the JSON-RPC profile proposed for long running resources would be simpler
to implement in other languages. The server contract for configuration operations already exists
in MCP, is documented and has clients. A second protocol for the same operations would have to be
kept in step with it. Rejected.

### Other client surfaces

- `dsc config set --server <host> --port <port>`, as first suggested in the issue, repeats the
  options on every subcommand, cannot express another transport, and `--server` reads like the
  existing `dsc server` subcommand.
- `dsc remote <target> config set ...` keeps global options untouched but duplicates the command
  tree.

A single global `--target` URI keeps commands identical for local and remote use.

### Transport extensions

Other transports were requested in the issue, WinRM in particular. The natural extension is a new
extension capability whose command connects stdin and stdout to a `dsc server` on a target, keyed
by URI scheme. PowerShell remoting, `docker exec`, `kubectl exec` and `wsl` all fit that shape, and
the client described here would not change. It needs its own RFC because every existing extension
capability runs once and exits, while a transport stays alive for the whole operation.

### Notify-to-pull

A target can also be told to fetch its configuration from a trusted registry instead of receiving
it inline. That depends on OCI artifact support and trusted-registry settings that are not designed
yet. Once they exist, the push described here extends to it by adding one server tool; the
transport, registration and client stay the same.

### Pull servers and agents

A scheduled pull with a local agent addresses drift correction rather than on-demand deployment.
It complements this proposal and is not replaced by it.

## Related work items

- [DSC as a SSH subsystem #1495](https://github.com/PowerShell/DSC/issues/1495) - The request and
  the discussion this RFC answers.
- [RFC for long running resources #1792](https://github.com/PowerShell/DSC/pull/1792) - Defines a
  JSON-RPC profile for resources over stdio; this RFC uses the existing MCP contract for remote
  operations instead.
