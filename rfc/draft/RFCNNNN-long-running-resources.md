---
RFC:          RFCNNNN # WG will set the number after submission
Author:       @SteveL-MSFT
Sponsor:      null    # <@GitHubUserName>
Status:       Draft   # <Draft | Experimental | Accepted | Final>
SupercededBy: null    # <Superceding RFC Number>
Version:      1.1     # <Major>.<Minor>
Area:         Resources
CommentsDue:  December 10, 2026
---

# Long Running DSC Resources

Allow a DSC resource to run as a single long lived process that handles many operations over a
JSON-RPC 2.0 connection on stdio, instead of being spawned once per operation.

## Motivation

> As a DSC configuration author,
> I want to deploy a configuration that uses the same resource many times,
> so that the deployment time is reduced.

> As a DSC resource author,
> I want to pay the startup cost of my runtime (for example loading PowerShell modules or a
> .NET/Python runtime) once per configuration,
> so that each additional instance of my resource is cheap to process.

Currently, if a large configuration document uses the same resource multiple times, each
invocation of the resource spawns a new process. For resources hosted in a runtime with a
noticeable startup cost this dominates the deployment time and increases resource consumption.

Although we introduced the pattern of a list resource accepting an array of items to process in a
single resource invocation, it changes the shape of the configuration and can make it more complex
to understand and maintain. The author has to know that a list variant exists, and references such
as `[reference(...)]` to a single item become awkward.

## Proposed experience

### Configuration authors

Nothing changes by default: a resource that supports long running operation is reused
automatically. Authors who need process isolation for a specific instance opt out with a resource
directive:

```yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Key1
  type: Microsoft.Windows/Registry
  properties:
    keyPath: HKCU\Example\One
- name: Key2
  type: Microsoft.Windows/Registry
  directives:
    reuseResource: false   # this instance gets its own process
  properties:
    keyPath: HKCU\Example\Two
```

Operators can disable reuse for an entire configuration run through a configuration level
directive or through DSC settings (see [Engine behavior](#engine-behavior)).

### Resource authors

A resource author declares a single entry point in the manifest and implements the JSON-RPC
contract described below. Existing per operation manifest fields (`get`, `set`, `test`, ...) remain
and keep describing the *semantics* of each operation (`returns`, `preTest`, `handlesExist`,
`requireSecurityContext`, `whatIfReturns`). Only the *transport* changes.

```yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.json
type: Microsoft.Windows/Registry
version: 0.2.0
jsonrpc:
  executable: registry
  args: ["--serve"]
  initializeTimeoutSeconds: 10
get:
  executable: registry          # still used when process isolation is requested
  args: ["config", "get", { jsonInputArg: "--input", mandatory: true }]
set:
  executable: registry
  args: ["config", "set", { jsonInputArg: "--input", mandatory: true }]
  returns: state
```

## Specification

### Configuration

#### Resource directive

A new field `reuseResource` (boolean) is added to the resource `directives` object
(`ResourceDirective`), alongside the existing `requireAdapter` and `securityContext` directives.

- Default: `true`. The engine reuses a long running process for this instance if one is available
  or can be started.
- `false`: this instance is executed in its own process. For a resource that declares both
  `jsonrpc` and executable based operations, the executable based operation is used. For a
  resource that declares only `jsonrpc`, the engine starts a dedicated JSON-RPC process for this
  instance, sends exactly one `dsc_invoke`, and shuts it down.

#### Configuration directive

A new field `reuseResources` (boolean, default `true`) is added to the document level
`directives` object (`ConfigDirective`). When `false`, every resource in the document behaves as if
`reuseResource: false` were set. A resource level `reuseResource: true` cannot override a
document level `false`.

#### Settings

A new setting `resourceReuse` (boolean, default `true`) is added to DSC settings with the same
effect as the document level directive, so an operator can enforce isolation machine wide without
editing configuration documents. The most restrictive of setting, document directive, and resource
directive wins.

#### Behavior matrix

| Manifest declares          | Effective reuse | Behavior                                                     |
| -------------------------- | --------------- | ------------------------------------------------------------ |
| executable ops only        | any             | Today's behavior: one process per operation.                 |
| `jsonrpc` only             | `true`          | One process per pool key for the whole run.                  |
| `jsonrpc` only             | `false`         | One JSON-RPC process per instance, shut down after the call. |
| `jsonrpc` + executable ops | `true`          | `jsonrpc` is used and executable ops are ignored.            |
| `jsonrpc` + executable ops | `false`         | Executable ops are used and `jsonrpc` is ignored.            |

### Resource manifest

A new optional top-level field `jsonrpc` is added to the resource manifest. Because the manifest
schema uses `deny_unknown_fields`, this requires a new manifest schema version. Manifests using
`jsonrpc` are rejected by older versions of DSC with the existing unknown field error.

```yaml
jsonrpc:
  executable: <string>                 # required, resolved like other executables (PATH or manifest directory)
  args: [<string>, ...]                # optional, static arguments only, no input argument kinds
  initializeTimeoutMS: <integer>       # optional, default 250ms
  requestTimeoutMS: <integer>          # optional, default 0 (no timeout)
  shutdownTimeoutMS: <integer>         # optional, default 250ms
```

Rationale for making the timeouts configurable in the manifest: the resource author knows the minimum
startup cost of their runtime (for example `pwsh -NoProfile` plus module import is already well
above 500ms), while the operator knows the machine. Operators can override the defaults in DSC
settings (`resourceTimeouts.initializeSeconds`, `.requestSeconds`, `.shutdownSeconds`). The larger
of the manifest value and the settings value is used.

JSON Schema fragment:

```json
{
  "jsonrpc": {
    "type": "object",
    "additionalProperties": false,
    "required": ["executable"],
    "properties": {
      "executable": { "type": "string" },
      "args": { "type": "array", "items": { "type": "string" } },
      "initializeTimeoutMS": { "type": "integer", "minimum": 1, "default": 250 },
      "requestTimeoutMS": { "type": "integer", "minimum": 0, "default": 0 },
      "shutdownTimeoutMS": { "type": "integer", "minimum": 1, "default": 250 }
    }
  }
}
```

The per operation manifest fields keep their meaning:

- The presence of `get`, `set`, `test`, `delete`, `export`, `resolve`, `validate`, `whatIf`,
  `schema` continues to declare which operations the resource supports. When `jsonrpc` is present
  the `executable`/`args`/`input` of these fields are only used for the isolation fallback.
- `requireSecurityContext` is validated by the engine before `dsc_invoke` is sent, exactly as today.
- `returns`, `whatIfReturns`, `preTest`, `handlesExist` govern how the engine interprets the
  `dsc_invoke` result for `set`/`whatIf`.
- `exitCodes` is reused as the error-code table for JSON-RPC errors (see [Errors](#errors)).

### Protocol

#### Transport and framing

- The engine starts the process with `jsonrpc.executable` and `jsonrpc.args`, with the working
  directory set to the manifest directory, and the same environment it passes to resources today
  (`DSC_CONFIG_ROOT`, `DSC_TRACE_LEVEL`, `DSC_TRACE_FORMAT`, ...).
- Messages flow over stdin (engine to resource) and stdout (resource to engine).
- Framing is newline delimited JSON: each message is a single JSON object serialized on one line
  and terminated by `\n`. Messages must not contain unescaped newlines.
- stdout is reserved for protocol messages. Any line on stdout that is not a valid JSON-RPC 2.0
  message is a protocol error: the engine logs the line at `error`, terminates the process, and
  fails the current resource instance.
- stderr keeps today's contract: structured trace lines (`{"level": "...", "fields": {"message":
  ...}}`) or the simple `{"error"|"warn"|"info"|"debug"|"trace": "..."}` objects are logged by the
  engine as they arrive, attributed to the process id. Resources must not use stderr for results.
- Only JSON-RPC 2.0 is supported, so every message carries `"jsonrpc": "2.0"`.
- The engine sends requests strictly sequentially for a given process: it does not send a new
  request until the previous response has been received. Resources may therefore be implemented
  as a simple read, handle, write loop. (If the engine later executes independent resources in
  parallel, it will use one process per concurrent lane unless the resource advertises
  `concurrentRequests: true` in its `initialize` result.)

#### Lifecycle

```text
engine                                resource
  │  spawn ───────────────────────────►  │
  │  initialize (request) ─────────────► │
  │  ◄───────────────────── initialize result
  │  dsc_invoke (request) ─────────────► │   repeated 0..n times
  │  ◄───────────────────── result / error
  │  shutdown (request) ───────────────► │
  │  ◄───────────────────── shutdown result
  │  exit (notification) ──────────────► │
  │  ◄───────────────────────── process exits
```

#### `initialize`

Sent once, immediately after spawn. It is **required**: the engine does not send `dsc_invoke`
until it has received the response. The result may be an empty object.

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {
    "dscVersion": "3.4.0",
    "protocolVersion": 1,
    "traceLevel": "info",
    "traceFormat": "json",
    "securityContext": "elevated",
    "configRoot": "/path/to/config"
  }
}
```

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "protocolVersion": 1,
    "operations": ["get", "set", "test", "delete", "export"],
    "concurrentRequests": false
  }
}
```

- `protocolVersion` is the version of *this* DSC JSON-RPC contract (not the JSON-RPC version). The
  engine sends the highest version it supports and the resource answers with the version it will
  speak, which must be less than or equal to the engine's. A higher value is a protocol error.
- `operations` is informational. The manifest remains the source of truth for which operations a
  resource supports. A mismatch is logged at `warn`.
- If no response arrives within `initializeTimeoutSeconds`, the engine terminates the process and
  fails the resource instance with an error naming the timeout.

#### `dsc_invoke`

Performs one operation. Parameters:

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "dsc_invoke",
  "params": {
    "operation": "set",
    "executionType": "actual",
    "resourceType": "Microsoft.Windows/Registry",
    "properties": { "keyPath": "HKCU\\Example\\One" },
    "context": {
      "targetResource": null
    }
  }
}
```

| Field           | Type   | Description                                                                                                                                                                                                                                                         |
| --------------- | ------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `operation`     | string | One of `get`, `set`, `test`, `delete`, `export`, `validate`, `resolve`, `schema`.                                                                                                                                                                                   |
| `executionType` | string | `actual` (default) or `whatIf`. Only meaningful for `set`. Replaces the `whatIf` argument kind.                                                                                                                                                                     |
| `resourceType`  | string | The fully qualified type being operated on. Equal to the manifest type except for adapters, where it is the adapted resource type.                                                                                                                                  |
| `properties`    | object | The instance properties, including `_exist`, `_metadata` and any other engine provided `_` fields, exactly as they would be written to stdin today. Omitted for `export` with no filter and for `schema`.                                                           |
| `context`       | object | Engine provided context. `targetResource` is `null` for ordinary resources. For adapters it is the full adapted resource object (`type`, `version`, `directory`, `manifest`, `properties`) that the adapter receives today, so adapters need no separate code path. |

The `result` of `dsc_invoke` is **exactly the JSON the operation would have written to stdout
today**. That means the actual state for `get`, the state or `{ "actualState": ..., "diff": [...] }`
for `set`/`whatIf` according to `returns`/`whatIfReturns`, the test result for `test`, an array of
instances for `export`, a configuration document for `resolve`, and a JSON Schema for `schema`. This
keeps `_inDesiredState`, `_exist`, `_metadata`, `_restartRequired` and the group/array detection
in `invoke_get`/`invoke_set`/`invoke_test` unchanged.

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": { "keyPath": "HKCU\\Example\\One", "_exist": true }
}
```

If `requestTimeoutSeconds` is nonzero and no response arrives in time, the engine terminates the
process and fails the resource instance.

#### Errors

A failed operation is reported as a JSON-RPC error object:

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "error": {
    "code": 3,
    "message": "Access to the registry key was denied",
    "data": { "keyPath": "HKCU\\Example\\One" }
  }
}
```

- `code` is interpreted with the manifest's `exitCodes` map, so resource authors keep a single
  error table for both the executable and JSON-RPC transports. A code not present in the map is
  reported with `message` alone, mirroring today's handling of unmapped exit codes.
- Codes in the JSON-RPC reserved range (`-32768` to `-32000`) are protocol errors: the engine logs
  them, terminates the process, and fails the instance.
- An application error (any other code) does **not** terminate the process. The process remains
  in the pool and the next instance is sent to it. This mirrors today's behavior where a failing
  instance does not affect subsequent instances.
- The engine surfaces the error with the same `DscError` variants used for exit code failures so
  CLI output and exit codes are unchanged.

#### `shutdown` and `exit`

At the end of the run the engine sends a `shutdown` request, waits up to `shutdownTimeoutSeconds`
for the response, then sends an `exit` notification and waits the same period for the process to
terminate. If either wait expires the process is killed. Both messages are sent even if the run is
failing, so resources can release locks or flush caches.

```json
{ "jsonrpc": "2.0", "id": 3, "method": "shutdown" }
{ "jsonrpc": "2.0", "id": 3, "result": null }
{ "jsonrpc": "2.0", "method": "exit" }
```

Resources must treat `shutdown`/`exit` as optional to implement in the sense that they may simply
respond with `null` and exit. The engine always sends them.

### Engine behavior

#### Process pool

The engine keeps a process pool for the duration of one configuration operation (`dsc config
get|set|test|export`). The pool is owned by the `Configurator` so that its lifetime matches the
run, and is drained (all processes shut down) when the operation completes or fails. In `dsc
server` (MCP) mode the pool may be kept alive across requests. This is left to a later RFC.

Processes are keyed by:

- fully qualified resource type,
- the resolved resource version (two `requireVersion` constraints can resolve to different
  manifests),
- the manifest directory,
- the required security context.

Two instances that differ in any of these fields use different processes.

#### Per instance flow

1. Evaluate directives and settings to decide the effective reuse value.
2. If reuse is `false`, follow the behavior matrix above.
3. Otherwise look up the pool key. If no live process exists, spawn one and run `initialize`.
4. Validate `requireSecurityContext`, build `properties` exactly as today (`add_metadata`), and
   send `dsc_invoke`.
5. Interpret the result through the existing `invoke_*` parsing.

#### Failure handling

- If the process exits unexpectedly while a request is outstanding, the current instance fails
  with the process exit code and captured stderr, exactly as a nonzero exit does today. The pool
  entry is removed and the **next** instance for the same key spawns a fresh process. One restart
  per key is permitted. A second unexpected exit fails the run to avoid tight restart loops.
- A protocol error (malformed stdout, reserved JSON-RPC error code, initialize timeout) fails the
  instance and removes the pool entry without a restart.

#### Nested invocations

`Microsoft.DSC/Group`, `Microsoft.DSC/Include`, `Microsoft.DSC/Assertion` and other group
resources run a nested `dsc config --as-group ...` process which owns its own pool. A resource used
both inside and outside a group therefore runs as (at least) two processes. Sharing a pool across
nested engines is out of scope for this RFC.

#### State isolation

Because a process is reused, state can leak between instances: environment variables, working
directory, caches, module-scope variables. Resource authors MUST treat each `dsc_invoke` as
independent and MUST NOT rely on ordering between instances. The engine makes no guarantee about
which instances share a process beyond the pool key. This is the primary reason the opt-out exists
and documentation for resource authors must call it out.

## Open Questions

- Should `dsc resource get|set|test` (single-instance CLI) start a JSON-RPC process and shut it
  down, or always use the executable fallback when one is declared? The former is simpler to
  reason about (one code path per manifest). The latter is faster for single calls.
- Should the pool survive across requests in `dsc server` mode, and if so, with what idle timeout?
- Should the engine pass the original resource instance `name` in `context` so resources can
  improve their logging? It is not needed for correctness.
- Should `initialize` carry the resource's own manifest so a generic host (for example a PowerShell
  or Python adapter running as a long lived process) can serve multiple adapted resources from one
  process?

## Alternate Proposals and Considerations

### Directive naming

`reuseResource: false` is a boolean that negates the default. An alternative is a positively named
directive such as `processIsolation: true` / `isolated: true`, which reads more naturally in a
document and avoids `reuseResource: true` meaning "the default". The WG should pick one. The
semantics are identical.

### Batch input instead of a long running process

The engine could instead collect all instances of a resource type and invoke the existing
executable once with an array of instances. This needs no new transport, but it breaks the
`dependsOn` ordering model (instances of one type can be interleaved with others), makes
`[reference()]` to earlier instances impossible to honor, and is what the list-resource pattern
already provides at the configuration level. Rejected.

### In-process library loading

Loading resources as shared libraries into the engine process removes process startup cost
entirely, but ties resources to the engine's ABI and platform, removes crash isolation, and is not
viable for PowerShell, Python or .NET resources. Rejected.

### Named pipes / sockets instead of stdio

A socket transport would allow a resource process to outlive the engine and be shared across runs,
but adds discovery, authentication and lifetime problems that stdio avoids. stdio is also what the
LSP and MCP ecosystems default to, so there is tooling and precedent. Could be added later as an
alternative `jsonrpc.transport` without changing the message contract.

### gRPC instead of JSON-RPC

`tonic` is already a workspace dependency for `dsc-bicep-ext`. gRPC would give typed schemas and
streaming, but requires every resource language to have a gRPC stack and code generation, which is
a large barrier for script based resources. JSON-RPC over stdio can be implemented with a JSON
parser and a read loop in any language.

### Reusing the MCP server contract

DSC already speaks JSON-RPC via MCP in `dsc server` mode. Reusing MCP for resources would conflate
tool discovery with resource invocation and bring in a much larger spec than needed. The message
shape here is deliberately a small subset so a resource can be written without any SDK.

## Related work items

- [Define JSONRPC contract for reusable resources #1782](https://github.com/PowerShell/DSC/issues/1782)
