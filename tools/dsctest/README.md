# DSCTest Resource

## JSON-RPC

`dsctest jsonrpc` serves the JSON-RPC 2.0 lifecycle (`initialize`, `dsc_invoke`, `shutdown`, and
`exit`) over newline-delimited stdin/stdout. The test manifests include:

- `Test/JsonRpc`: native get, set, test, delete, export, and command-based schema, including native
  set/delete what-if and state-and-diff results.
- `Test/JsonRpcValidate`: custom validation.
- `Test/JsonRpcFallback`: get through JSON-RPC or an executable fallback.
- `Test/JsonRpcSynthetic`: no-return set/test and synthetic export/what-if.
- `Test/JsonRpcAdapter`: a single-input adapter for `Adapted/JsonRpcOne` and
  `Adapted/JsonRpcTwo`, with executable enumeration and command-based export-input schema.

Returned state includes `processId`, `requestCount`, and `transport` for lifecycle assertions.
Set `DSC_TEST_JSONRPC_LOG` to a file path to append lifecycle events as JSON Lines.
`dsctest jsonrpc --operation get --input '{}'` exercises the executable fallback.

Test properties can request `delayMS`, valid `notifications`, custom `progress` notifications, or
`notificationDelayMS` for deadline tests. `fault` accepts `error` (with optional `errorCode`),
`exit`, `malformed`, `wrongId`, `nullId`, `missingId`, `both`, `batch`, `duplicate`, or `invalidResult`.
Set `DSC_TEST_JSONRPC_FAULT` to `initializeError`, `initializeVersion`, `initializeTimeout`,
`shutdownError`, `shutdownTimeout`, `exitTimeout`, or `exitOutput` to exercise lifecycle failures.

The engine's Pester coverage is in `dsc/tests/config/dsc_jsonrpc.tests.ps1`.

## Sleep

Example config:

```yaml
$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/08/config/document.json
resources:
- name: Sleep1
  type: Test/Sleep
  properties:
    seconds: 30
```
