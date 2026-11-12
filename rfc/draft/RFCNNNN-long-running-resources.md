---
RFC:          RFCNNNN # WG will set the number after submission
Author:       @SteveL-MSFT
Sponsor:      null    # <@GitHubUserName>
Status:       Draft   # <Draft | Experimental | Accepted | Final>
SupercededBy: null    # <Superceding RFC Number>
Version:      1.3     # <Major>.<Minor>
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

Existing resources keep today's behavior. A resource that opts into long running operation is
reused automatically without changing the configuration's instance properties or results. Authors
who need fresh-process isolation for every operation on a specific instance opt out with a resource
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
    reuseResource: false   # each underlying operation gets a fresh process
  properties:
    keyPath: HKCU\Example\Two
```

Operators can disable reuse for an entire configuration run through a configuration level
directive or through DSC settings (see [Engine behavior](#engine-behavior)).

### Resource authors

A resource author declares a single entry point in the manifest and implements the JSON-RPC
contract described below. Existing per operation manifest fields (`get`, `set`, `test`, ...) remain
and keep describing the *semantics* of each operation (`return`, `implementsPretest`, `handlesExist`,
`requireSecurityContext`, `whatIfReturns`). The engine keeps its orchestration, validation, and
semantic result contracts; a transport normalization layer maps JSON-RPC results to those contracts.

```yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.json
type: Microsoft.Windows/Registry
version: 0.2.0
jsonrpc:
  executable: registry
  args: ["--serve"]
  initializeTimeoutMS: 10000
get:
  executable: registry          # still used when process isolation is requested
  args: ["config", "get", { jsonInputArg: "--input", mandatory: true }]
set:
  executable: registry
  args: ["config", "set", { jsonInputArg: "--input", mandatory: true }]
  return: state
```

## Specification

### Configuration

#### Resource directive

A new field `reuseResource` (boolean) is added to the resource `directives` object
(`ResourceDirective`), alongside the existing `requireAdapter` and `securityContext` directives.

- Default: `true`. The engine reuses a long running process for this instance if one is available
  or can be started.
- `false`: each underlying operation for this instance runs in a fresh process, not in the pool.
  If that operation declares an executable fallback, the engine uses it. Otherwise, the engine
  starts a dedicated JSON-RPC process, initializes it, sends one `dsc_invoke`, and shuts it down.
  One instance can require several underlying calls, such as test, get-before, set, and get-after.
  Each of those calls gets a fresh process. This preserves today's fresh-process isolation rather
  than introducing shared state between the calls for an isolated instance.

#### Configuration directive

A new field `reuseResources` (boolean, default `true`) is added to the document level
`directives` object (`ConfigDirective`). When `false`, every resource in the document behaves as if
`reuseResource: false` were set. A resource level `reuseResource: true` cannot override a
document level `false`. Both new directive fields require updated configuration schemas; the
configuration schema URI in the example must resolve to a version that includes them.

#### Settings

A new setting `resourceReuse` (boolean, default `true`) is added to DSC settings with the same
effect as the document level directive, so an operator can enforce isolation machine wide without
editing configuration documents. The most restrictive of setting, document directive, and resource
directive wins. The setting is resolved through DSC's existing settings scopes and policy
precedence; it is not a separate machine-only settings mechanism.

#### Behavior matrix

| Transport declaration for the operation | Effective reuse | Behavior |
| --------------------------------------- | --------------- | -------- |
| executable only                         | any             | Today's behavior: one process per underlying operation. |
| `jsonrpc`, no executable fallback        | `true`          | One process per pool key for the whole run. |
| `jsonrpc`, no executable fallback        | `false`         | One fresh JSON-RPC process per underlying operation. |
| `jsonrpc` + executable fallback          | `true`          | JSON-RPC is used; executable fallback is not invoked. |
| `jsonrpc` + executable fallback          | `false`         | Executable fallback is used for this operation. |

Transport selection is per underlying operation, so a manifest may provide fallback commands for
some operations but not others. Synthetic operations and embedded schemas do not acquire a
transport of their own; their underlying calls follow this matrix.

### Resource manifest

A new optional top-level field `jsonrpc` is added to the resource manifest. The Rust manifest
deserializer uses `deny_unknown_fields`, and the JSON Schema rejects unknown properties. The new
field and operation declarations below require an updated schema version and deserializer.
Manifests using `jsonrpc` are rejected by older versions of DSC; they cannot use the executable
fallback to make the new manifest backward compatible. The schema URIs in the examples illustrate the
existing URI family and must be updated to the schema version that introduces this feature.

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
above 500ms), while the operator knows the machine. Authors must specify an initialization timeout
appropriate for their runtime rather than relying on the 250ms default.

All timeout values are milliseconds, in both the manifest and DSC settings
(`resourceTimeouts.initializeTimeoutMS`, `.requestTimeoutMS`, `.shutdownTimeoutMS`). An explicitly
configured setting overrides the corresponding manifest value; otherwise the manifest value or its
default applies. Settings use the existing DSC scope and policy precedence. Initialization and
shutdown timeouts must be positive. A request timeout of `0` means unlimited, not a duration of zero.
In particular, an operator's positive request timeout overrides a manifest's unlimited default;
timeouts are not combined using a numeric maximum.

JSON Schema fragment:

```json
{
  "properties": {
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
}
```

#### Operation declarations and fallbacks

The presence of `get`, `set`, `test`, `delete`, `export`, `resolve`, `validate`, or `whatIf`
continues to declare the corresponding native operation. With `jsonrpc`, these objects may omit
`executable`. An empty object, such as `get: {}`, declares a JSON-RPC operation without an
executable fallback. Without `jsonrpc`, the existing required `executable` fields remain required.
If `args` or `input` is present, `executable` must also be present; these fields configure only
the executable fallback.

The existing semantic fields apply to both transports:

- `requireSecurityContext` is checked before spawning or invoking a process. It validates the
  engine's current privileges; neither it nor `initialize.securityContext` requests elevation
  or impersonation.
- `return`, `whatIfReturns`, `implementsPretest`, and `handlesExist` retain their current meaning.
  The `test.return` field also controls test result interpretation.
- `export.supportsFiltering` and export input schema declarations retain their existing meaning
  and validation. A command-based export input schema may omit its executable when `jsonrpc`
  is present; its JSON-RPC retrieval is described under [Operation results](#operation-results).
- `schema.embedded` remains engine-side and does not cause a JSON-RPC call. `schema.command`
  declares dynamic schema retrieval and may omit its executable when `jsonrpc` is present.
- Missing `test` or `export` continues to permit synthetic test or export using `get`, exactly as
  today. Missing native operations are not implicitly advertised by `initialize`.
- `exitCodes` can supply descriptions for nonzero application error codes
  (see [Errors](#errors)).

For example, this resource has no executable operation fallbacks:

```yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.json
type: Contoso/Example
version: 1.0.0
jsonrpc:
  executable: example
  args: ["--serve"]
  initializeTimeoutMS: 10000
get: {}
set:
  return: state
schema:
  embedded:
    type: object
    properties:
      value:
        type: string
    additionalProperties: false
```

#### What-if capabilities

A new optional `supportsWhatIf` boolean on `set` and `delete` declares native JSON-RPC what-if
support when `jsonrpc` is present. Its default is `false`. Existing `whatIf` argument declarations
on executable commands also continue to declare native what-if support for dual-transport
resources, and a separate top-level `whatIf` object continues to declare a set what-if operation.
A resource using either declaration must implement the advertised behavior over JSON-RPC.

For set what-if, the engine preserves today's selection order: native set what-if first, then the
separate `whatIf` operation, then synthetic test-based what-if. Native set what-if uses
`operation: "set"` and `executionType: "whatIf"`; the separate operation uses
`operation: "whatIf"` and `executionType: "whatIf"`. Its own `return` and `whatIfReturns` apply.
Native delete what-if uses `operation: "delete"` and `executionType: "whatIf"`; otherwise the engine
uses today's synthetic delete what-if. Merely supporting `set` or `delete` does not advertise
what-if support. No actual mutation is dispatched during synthetic what-if.

If `supportsWhatIf: true` is declared and an executable fallback is provided for the same operation,
that fallback must also declare a `whatIf` argument. This ensures disabling reuse cannot remove
the advertised native what-if behavior.

### Protocol

This is a DSC-specific profile of [JSON-RPC 2.0](https://www.jsonrpc.org/specification), particularly
sections 4 (requests and notifications), 5 (responses and errors), and 6 (batches). Framing and
lifecycle rules below are DSC requirements, not requirements of the transport-agnostic JSON-RPC
specification.

#### Transport and framing

- The engine starts the process with `jsonrpc.executable` and `jsonrpc.args`, with the working
  directory set to the manifest directory, and the same environment it passes to resources today
  (`DSC_CONFIG_ROOT`, `DSC_TRACE_LEVEL`, `DSC_TRACE_FORMAT`, ...).
- Messages flow over stdin (engine to resource) and stdout (resource to engine). Resource stdout
  carries responses and the tracing/progress notifications defined below.
- Framing is newline delimited JSON: each message is a single JSON object serialized on one line
  and terminated by `\n`. Receivers also accept `\r\n`. Messages must not contain unescaped
  newlines. Both peers must flush each message after writing it; they cannot wait for EOF.
  Resource writers must serialize access to stdout so notifications from background work cannot
  interleave bytes with another notification or the final response.
- stdout is reserved for protocol messages. Any line on stdout that is not a valid JSON-RPC 2.0
  response or notification permitted by this profile is a protocol error: the engine logs a
  diagnostic, terminates the process, and fails the current resource instance. Diagnostics must
  not expose unredacted resource properties or raw stdout that may contain secrets.
- stderr keeps today's contract: structured trace lines (`{"level": "...", "fields": {"message":
  ...}}`) or the simple `{"error"|"warn"|"info"|"debug"|"trace": "..."}` objects are logged by the
  engine as they arrive, attributed to the process id. Resources must not use stderr for results.
- Only JSON-RPC 2.0 is supported, so every message carries `"jsonrpc": "2.0"`.
- The engine assigns a unique positive integer `id` to each request for the lifetime of the
  process. The resource's response must return that same `id`, with exactly one of `result` or `error`.
  A missing, mismatched, or duplicate response id is a protocol error. JSON-RPC parse/invalid-request
  error responses may use `id: null`; receiving one fails the outstanding call.
- The engine sends only named parameters (an object), not positional parameters. Batches and
  resource-originated requests requiring a response are outside this profile. The engine never
  sends batches. Resources may send only the `dsc_trace` and `dsc_progress` notifications defined
  below, and only during an outstanding `dsc_invoke`. Other unsolicited stdout messages fail the
  call; the engine must not respond to notifications, including unsupported ones.
  The only engine notification in this version is `exit`, to which the resource must not reply.
- The engine sends requests strictly sequentially for a given process: it does not send a new
  request until the previous response has been received. Resources may therefore be implemented
  as a simple read, handle, write loop. Concurrent requests are outside protocol version 1;
  additional processes would be required for concurrent execution. While waiting for a response,
  the engine continuously reads stdout and stderr and dispatches notifications without completing
  the outstanding request. Notifications do not require concurrent invocation support.

#### Lifecycle

```text
engine                                resource
  │  spawn ───────────────────────────►  │
  │  initialize (request) ─────────────► │
  │  ◄───────────────────── initialize result
  │  dsc_invoke (request) ─────────────► │   repeated 0..n times
  │  ◄───────────────────── dsc_trace / dsc_progress (notifications, optional 0..n)
  │  ◄───────────────────── result / error
  │  shutdown (request) ───────────────► │
  │  ◄───────────────────── shutdown result
  │  exit (notification) ──────────────► │
  │  ◄───────────────────────── process exits
```

#### `initialize`

Sent once, immediately after spawn. It is **required**: the engine does not send `dsc_invoke`
until it has received a successful response with a supported protocol version.

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {
    "dscVersion": "3.4.0",
    "protocolVersions": [1],
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
    "operations": ["get", "set"]
  }
}
```

- `protocolVersions` lists the DSC contract versions the engine actually supports, not JSON-RPC
  versions, as a nonempty array of positive integers. The resource must return `protocolVersion`
  as an integer with a version from that list which it
  also supports. Version 1 is the only version defined here. An empty result, missing version, or
  unsupported version fails initialization. With no common version, the resource returns an
  application error explaining the incompatibility.
- `operations` is optional and informational. If provided, it lists native operations, not
  synthesized operations or embedded schema retrieval. The manifest remains the source of truth.
  A mismatch is logged at `warn` and never adds a capability.
- `securityContext` describes the engine's actual elevated or restricted context after validation.
  `configRoot` describes the run's configuration root; it may be omitted when none is available.
- If no complete response arrives within the effective `initializeTimeoutMS`, the engine
  terminates the process and fails the resource instance with an error naming the timeout.

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

| Field           | Type   | Description |
| --------------- | ------ | ----------- |
| `operation`     | string | One of `get`, `set`, `whatIf`, `test`, `delete`, `export`, `validate`, `resolve`, `schema`. Each call invokes one native operation, not the complete engine-level workflow. |
| `executionType` | string | `actual` (default) or `whatIf`. Meaningful for `set`, `whatIf`, and `delete`; other operations use `actual`. Native what-if requires a manifest capability declaration. |
| `resourceType`  | string | The fully qualified type being operated on. Equal to the manifest type except for adapters, where it is the adapted resource type. |
| `properties`    | object | The evaluated input properties or export filter, with `_exist` and supported `_metadata` added according to today's rules. The same logical input is used regardless of whether the executable transport would use stdin, args, or environment variables. Omitted when there is no input, including unfiltered `export` and `schema`. |
| `context`       | object | Engine-provided context. `targetResource` is `null` for ordinary resources, or the adapted target descriptor defined under [Adapters](#adapters). For `schema`, `schemaKind` identifies the schema being requested. |

#### Operation results

JSON-RPC requires one response value, whereas existing command resources can emit zero, one, or
many JSON lines. The following mapping defines that value. It preserves the logical contents of
existing operation output, not its raw stdout framing and not the final CLI result envelope.

| Operation / declaration | JSON-RPC `result` | Existing executable output |
| ----------------------- | ---------------- | -------------------------- |
| `get` | Actual state, including any canonical resource properties; existing group result arrays remain arrays. | The existing JSON state or group result. |
| `set` or `whatIf`, `return: state` | State object. | One JSON state line. |
| `set` or `whatIf`, `return: stateAndDiff` | `{ "actualState": <state>, "diff": [<property names>] }`. | State line followed by a diff array line. |
| `set` or `whatIf`, no `return` | `null`; the engine performs the existing follow-up get and synthesizes the result. | No result output is required. |
| `test`, `return: state` | State with `_inDesiredState` interpreted according to today's rules. | One JSON state line. |
| `test`, `return: stateAndDiff` | `{ "actualState": <state>, "diff": [<property names>] }`. | State line followed by a diff array line. |
| `test`, no `return` | `null`; the engine performs the existing follow-up get and synthesizes the result. | Any result output is ignored. |
| Actual `delete` | `null`. | No result output is required. |
| Native delete what-if | The existing delete what-if object, including supported `_metadata.whatIf`. | One delete what-if JSON object. |
| `export` | Array of instances, or `[]` for no instances. Exporter-kind resources return an array of resource definitions to recursively export. | One JSON line per instance or resource definition. |
| `validate` | `{ "valid": <boolean>, "reason": <string or null> }`. | The existing validation result object. |
| `resolve` | `{ "configuration": <document>, "parameters": <object or null> }`. | The existing resolve result wrapper. |
| `schema` | JSON Schema value. | One JSON Schema value from a schema command. |

For what-if on `set` or the separate `whatIf` operation, `whatIfReturns` overrides that selected
operation's `return` when present. Existing group/adapter result shapes and kind-specific
interpretation take precedence over the ordinary resource rows in the table; group results
are not wrapped as ordinary resource state.

For `operation: "schema"`, `context.schemaKind` is `resource` (the default) for `schema.command`,
or `exportInput` for the command-based export filter schema. Embedded schemas remain engine-side.
Schema caching and input/output validation keep their current behavior for both transports.

The engine normalizes these values before applying existing semantic processing. In particular,
`stateAndDiff` objects and export arrays require new transport-aware parsing; they cannot be passed
unchanged to parsers expecting multiple stdout lines. `_inDesiredState`, `_exist`, `_metadata`,
`_restartRequired`, before/after-state calculation, and group detection retain their current
meaning. A shape or schema validation failure fails the operation.

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": { "keyPath": "HKCU\\Example\\One", "_exist": true }
}
```

If the effective `requestTimeoutMS` is nonzero and no complete response arrives in time, the engine
terminates the process and fails the resource instance. Neither stderr activity nor tracing/progress
notifications reset this timeout.

#### Tracing and progress notifications

While a `dsc_invoke` request is executing, a resource may emit `dsc_trace` and `dsc_progress`
notifications on stdout. These are DSC-defined methods using the JSON-RPC 2.0 notification
mechanism, not partial responses or additional invocations. They have no top-level `id`, `result`,
or `error`, and the engine must not acknowledge them.

Both methods require an object `params` with `requestId` equal to the outstanding `dsc_invoke`
request's integer id. This is application-level correlation data, not a notification request id.
The engine attributes the event to that invocation's instance and process. Notifications must be
written before the final response; a missing or mismatched `requestId`, an invalid payload, or a
notification outside an outstanding invocation is a protocol error under this DSC profile.
Tracing during initialization, shutdown, or idle time continues to use stderr.

| Method | Additional parameters | Meaning |
| ------ | --------------------- | ------- |
| `dsc_trace` | Required `level`: one of `error`, `warn`, `info`, `debug`, `trace`; required `message`: string. | A trace event, subject to the engine's configured trace-level filtering. |
| `dsc_progress` | Required `message`: string; optional `completed`: finite nonnegative number; optional `total`: finite positive number. | Progress for the outstanding invocation, not the overall configuration. |

If `total` is present, `completed` must also be present and satisfy `0 <= completed <= total`.
The units are resource-defined but must be consistent within an invocation. Omit `total` when the
amount of work is unknown; `completed` may also be omitted for an indeterminate activity message.
When reporting numeric progress, keep `completed` nondecreasing and a reported `total` stable for
that invocation. A final response ends the activity; reaching `total` does not complete the request.

For example, after DSC sends this invocation:

```json
{"jsonrpc":"2.0","id":2,"method":"dsc_invoke","params":{"operation":"set","executionType":"actual","resourceType":"Contoso/Package","properties":{"name":"Example","installed":true},"context":{"targetResource":null}}}
```

The resource can write these separate, newline-terminated stdout messages over time, assuming its
set operation declares `return: state`:

```json
{"jsonrpc":"2.0","method":"dsc_trace","params":{"requestId":2,"level":"info","message":"Downloading package"}}
{"jsonrpc":"2.0","method":"dsc_progress","params":{"requestId":2,"message":"Downloading package","completed":40,"total":100}}
{"jsonrpc":"2.0","id":2,"result":{"name":"Example","installed":true}}
```

The engine forwards progress through its progress-reporting path and traces through its logging
path, applying existing secret-redaction rules. Resources must not include secrets in either
payload. Notifications are optional and informational: they do not alter operation results or
enter the resource state/CLI result envelope. In particular, an `error`-level trace is not a failed
operation, and progress is not a heartbeat that extends the request deadline. Exactly one final
response, containing `result` or `error`, determines the invocation outcome.

Existing stderr tracing remains supported and is drained while the invocation runs. Resource
authors should use either stderr or `dsc_trace` for a given trace event, not both, to avoid duplicate
logs. Resources should rate-limit notifications so progress reporting does not overwhelm the engine
or delay the final response.

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

- `code` must be an integer. DSC restricts it to a signed 32-bit value to match the existing
  `exitCodes` map. Every error response is a failure regardless of its code. Resources must not
  use application code `0`, because executable exit code `0` means success; receiving an error
  with code `0` still fails the call.
- Nonzero application codes outside `-32768..-32000` may use the manifest's `exitCodes`
  descriptions so authors can keep a single error table. A mapped description supplements, rather
  than discards, the response's `message`. Unmapped codes retain both code and message.
- JSON-RPC reserves `-32768..-32000`, including the implementation-defined server-error range
  `-32099..-32000`. Defined JSON-RPC errors and server errors are valid error responses, not
  malformed messages. They fail the call without requiring immediate process termination solely
  because of their numeric range. Resources must not repurpose other reserved codes as application
  codes, and DSC does not interpret reserved codes through `exitCodes`.
- `data` is optional diagnostic information, not a resource result. The engine preserves it for
  diagnostics subject to existing redaction rules; it must not log secrets from error payloads.
- An error response does not by itself require killing a responsive process. However, current
  configuration processing is fail-fast: the first failed operation ends the run and triggers
  pool shutdown. No subsequent instance is sent to the process in that run.
- The engine uses the existing resource-operation error reporting path. JSON-RPC error codes are
  resource diagnostics, not DSC CLI exit codes. Existing CLI success/failure exit-code behavior
  remains unchanged; preserving structured RPC diagnostics may require an additional error variant.

#### `shutdown` and `exit`

At the end of the run the engine sends a `shutdown` request to each live, initialized process,
waits up to the effective `shutdownTimeoutMS` for its response, then sends an `exit` notification
and waits the same period for process termination. A resource responds to `shutdown` with
`result: null`, stops accepting operation requests, and exits when it receives `exit`. It must not
respond to `exit`, because notifications have no responses. If either wait expires, the process
is killed.

```json
{ "jsonrpc": "2.0", "id": 3, "method": "shutdown" }
{ "jsonrpc": "2.0", "id": 3, "result": null }
{ "jsonrpc": "2.0", "method": "exit" }
```

Both methods are required, but resource-specific cleanup work is optional. Shutdown after a valid
operation error uses the same sequence, allowing resources to release locks or flush caches.
Failed initialization, malformed protocol output, and an outstanding request timeout terminate the
process directly because a responsive, initialized peer is not available. Already-dead processes
receive no further messages.

Cancellation fails the run and prevents new invocations. An idle initialized process can be shut
down gracefully; a process with an outstanding operation is terminated, since this version defines
no request-cancellation method. Cleanup errors are reported without replacing the original failure.
A shutdown failure after otherwise successful work is surfaced as a run error, not silently ignored.
The engine closes pipes, drains available stderr, and waits for terminated child processes before
returning. Resource authors must not rely on graceful shutdown being possible on every failure path.

### Engine behavior

#### Process pool

The engine keeps a process pool for the duration of one configuration operation (`dsc config
get|set|test|export`). The pool is owned by the `Configurator` so that its lifetime matches the
run, and is drained (all processes shut down) when the operation completes or fails. Resource
operation calls made during discovery, validation, or schema retrieval for the run must use the
same invocation context and pool, rather than creating hidden per-call pools. Adapter list commands
remain on their existing discovery path. The pool does not survive across requests in `dsc server`
(MCP) mode in this RFC; cross-request reuse is left to a later RFC.

Processes are keyed by:

- the host's fully qualified resource type,
- the host's resolved resource version,
- the resolved host manifest identity and directory, which determine the executable and arguments,
- the actual process security context.

Two instances that differ in any of these fields use different processes. For ordinary resources,
the host is the resource itself. For adapters, the host is the resolved adapter, not the adapted
target; target identity is sent on every invocation. The engine does not launch a different
privilege context merely because an operation declares a different requirement.

#### Per instance flow

1. Evaluate directives, expressions, resource conditions, and settings using today's ordering
   and validation rules; determine the effective reuse value.
2. Preserve the engine-level workflow: synthetic test/export/what-if, pretest, before-state get,
   `_exist` routing to delete when `handlesExist` is not set, and follow-up get when needed.
3. For each underlying native call, validate resource and operation security requirements before
   spawning or initializing a process. Preserve adapted-target requirement checks.
4. Build the same logical input as today, including `add_metadata` rules and schema validation.
   Embedded schemas are resolved without invoking a process.
5. Select the transport according to the behavior matrix. For pooled JSON-RPC, look up the host
   key and initialize a process if needed; for isolated JSON-RPC, start a fresh process.
6. Invoke the executable fallback or send one `dsc_invoke` for the native operation. Normalize
   JSON-RPC responses according to [Operation results](#operation-results), apply existing
   semantic result processing and validation, and shut down any dedicated isolated process.

The engine preserves `dependsOn` ordering and updates reference values after each instance's
workflow completes. Process reuse does not batch or reorder instances.

#### Failure handling

- The first failed operation ends the configuration run, consistent with today's behavior.
  The engine stops dispatching instances and drains the pool.
- Unexpected process exit while a request is outstanding fails the call, even if the process
  exits with code `0`, because the required response is missing. A nonzero exit code and available
  stderr use existing process-error diagnostics; signal termination is reported explicitly.
- Unexpected exit of an idle pooled process is also a run failure when observed. The engine
  does not silently restart it and hide the failure.
- Malformed responses, failed initialization, and request timeouts remove and terminate the
  affected process. Valid JSON-RPC errors fail the operation and allow graceful cleanup.
- The engine never automatically retries a failed invocation or restarts a process within the
  failed run. A request may have changed system state before its response was lost; replaying it
  would not be safe. A subsequent, separately initiated run starts with a fresh pool.

#### Adapters

This version supports JSON-RPC invocation for adapters with `adapter.inputKind: single`.
Existing `full` and `sequence` adapter paths remain unchanged; JSON-RPC hosting for those modes
is outside this RFC. A manifest combining `jsonrpc` with either mode is rejected, rather than
silently ignoring the transport declaration. Adapter enumeration
(`adapter.list`) remains on the existing executable discovery path and is not a `dsc_invoke`
operation.

For a single-input adapter, `resourceType` is the adapted target's type. `context.targetResource`
is a new explicit wire descriptor, not a claim that today's adapters already receive this object:

| Field | Type | Meaning |
| ----- | ---- | ------- |
| `type` | string | Adapted target's fully qualified type. |
| `version` | string | Resolved adapted target version. |
| `path` | string or null | Target manifest/definition path, corresponding to `resourcePathArg`. |
| `directory` | string or null | Target definition directory. |
| `manifest` | object or null | Target command resource manifest, when present. |
| `adaptedManifest` | object or null | Target adapted-resource manifest, when present. |
| `adaptedContent` | object or null | Adapted definition content, corresponding to `adaptedContentArg`. |
| `schema` | object or null | Target's embedded instance schema, when present. |

Instance values are provided only in `params.properties`; they are not duplicated in this
descriptor. The current `DscResource.properties` list describes property names, not instance
values, and is not the invocation payload. Optional descriptor fields use `null` when unavailable.
The adapter must use the target descriptor for each call rather than caching one target as
process-wide state. Targets hosted by different resolved adapter versions or manifests never share
a process; different targets may share the same adapter host.

#### Nested invocations

`Microsoft.DSC/Group`, `Microsoft.DSC/Include`, `Microsoft.DSC/Assertion` and other group
resources run a nested `dsc config --as-group ...` process which owns its own pool. A resource used
both inside and outside a group therefore runs as (at least) two processes. Sharing a pool across
nested engines is out of scope for this RFC.

A document-level `reuseResources: false` also constrains nested DSC runs created for that document.
The engine propagates an inherited reuse prohibition in the internal run context, including across
nested DSC process boundaries. A nested document or instance cannot override it with `true`.
Settings and any additional nested restrictions are still evaluated. An instance-level
`reuseResource: false` isolates calls to that instance, including a group host, but does not by
itself disable reuse among the group's child instances.

#### State isolation

Because a process is reused, state can leak between instances: environment variables, working
directory, caches, module-scope variables. Resource authors MUST treat each `dsc_invoke` as
independent and MUST NOT rely on incidental process-local state or ordering to identify an instance.
This does not remove DSC's `dependsOn` and reference ordering guarantees. The engine makes no
guarantee about which instances share a process beyond the pool key. This is the primary reason the
opt-out exists and documentation for resource authors must call it out.

## Open Questions

- Should `dsc resource get|set|test` (single-instance CLI) start a JSON-RPC process and shut it
  down, or always use the executable fallback when one is declared? The former is simpler to
  reason about (one code path per manifest). The latter is faster for single calls.
- Should the pool survive across requests in `dsc server` mode, and if so, with what idle timeout?
- Should the engine pass the original resource instance `name` in `context` so resources can
  improve their logging? It is not needed for correctness.
- Should adapter `list` and `full`/`sequence` input modes use JSON-RPC in a future version?

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
contract here is deliberately a small DSC-specific profile so a resource can be written without
any SDK. It is not an MCP-compatible lifecycle or an LSP transport implementation.

## Related work items

- [Define JSONRPC contract for reusable resources #1782](https://github.com/PowerShell/DSC/issues/1782)
