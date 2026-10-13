---
description: Microsoft.Windows/Service resource reference documentation
ms.date:     05/08/2026
ms.topic:    reference
title:       Microsoft.Windows/Service
---

# Microsoft.Windows/Service

## Synopsis

Manage Windows services.

## Metadata

```yaml
Version    : 0.1.0
Kind       : resource
Tags       : [Windows]
Author     : Microsoft
```

## Instance definition syntax

```yaml
resources:
  - name: <instance name>
    type: Microsoft.Windows/Service
    properties:
      # Key properties
      name: string
      # Instance properties
      description:
      dependencies:
      displayName:
      errorControl:
      executablePath:
      logonAccount:
      startType:
      status:
```

## Description

The `Microsoft.Windows/Service` resource enables you to idempotently manage the configuration and
runtime state of Windows services registered with the Service Control Manager (SCM). The resource
can:

- Retrieve the full configuration and status of a service.
- Change the start type, status, description, display name, logon account, error control,
  executable path, and service dependencies.
- Export a list of all services registered on the system.

> [!NOTE]
> This resource is installed with DSC itself on Windows systems.
>
> You can update this resource by updating DSC. When you update DSC, the updated version of this
> resource is automatically available.

## Requirements

- The resource is only usable on a Windows system.
- **Set** operations require an elevated (administrator) process context. Running `dsc` without
  elevation when using the **Set** operation causes an access-denied error from the SCM.

## Capabilities

The resource has the following capabilities:

- `get` - You can use the resource to retrieve the actual state of a service instance.
- `set` - You can use the resource to enforce the desired configuration and status of a service.
- `export` - You can use the resource to export a list of all services registered on the system.

This resource uses the synthetic test functionality of DSC to determine whether an instance is in
the desired state. For more information about resource capabilities, see
[DSC resource capabilities][01].

## Examples

1. [Get service status][02] - Shows how to retrieve the current state of a Windows service with the
   `dsc resource` commands.
1. [Configure a Windows service][03] - Shows how to enforce the desired configuration of a Windows
   service using a DSC configuration document.

## Properties

The following list describes the properties for the resource.

- **Key properties:** <a id="key-properties"></a> The following properties uniquely identify an
  instance. If two instances of a resource have the same values for their key properties, the
  instances are conflicting. For more information about key properties, see the "Key resource
  properties" section in [DSC resource properties][04].

  - [name](#name) - The name of the service in the Service Control Manager.

- **Instance properties:** <a id="instance-properties"></a> The following properties are optional.
  They define the desired state for an instance of the resource.

  - [dependencies](#dependencies) - A list of service names that this service depends on.
  - [description](#description) - A description of the service.
  - [displayName](#displayname) - The display name of the service shown in the Services console.
  - [errorControl](#errorcontrol) - The error control level for the service.
  - [executablePath](#executablepath) - The fully qualified path to the service binary.
  - [logonAccount](#logonaccount) - The account under which the service runs.
  - [startType](#starttype) - The start type of the service.
  - [status](#status) - The current or desired status of the service.

- **Read-only properties:** <a id="read-only-properties"></a> The resource returns the following
  properties, but they aren't configurable. For more information about read-only properties, see
  the "Read-only resource properties" section in [DSC resource properties][05].

  - [_exist](#_exist) - Indicates whether the service exists in the Service Control Manager.

### name

<details><summary>Expand for <code>name</code> property metadata</summary>

```yaml
Type        : string
IsRequired  : false
IsKey       : true
IsReadOnly  : false
IsWriteOnly : false
```

</details>

The service key name as registered in the Service Control Manager. This is the short internal name
used to identify the service. For example, `wuauserv` for Windows Update. This value is
case-insensitive.

When performing a **Get** operation you may supply either `name` or `displayName` (or both) to
identify the service. For **Set** operations you must supply `name`.

### displayName

<details><summary>Expand for <code>displayName</code> property metadata</summary>

```yaml
Type        : string
IsRequired  : false
IsKey       : false
IsReadOnly  : false
IsWriteOnly : false
```

</details>

The friendly display name of the service shown in the Windows Services console — for example,
`Windows Update`. You can define `displayName` instead of (or alongside) `name` in a **Get**
operation to locate a service when you don't know its key name. If both are provided, DSC verifies
that they refer to the same service and returns an error if they don't match.

### description

<details><summary>Expand for <code>description</code> property metadata</summary>

```yaml
Type        : string
IsRequired  : false
IsKey       : false
IsReadOnly  : false
IsWriteOnly : false
```

</details>

A human-readable description of the service. Setting this property updates the description shown
in the Services console and the **Description** field in the SCM database.

### _exist

<details><summary>Expand for <code>_exist</code> property metadata</summary>

```yaml
Type        : boolean
IsRequired  : false
IsKey       : false
IsReadOnly  : true
IsWriteOnly : false
```

</details>

Indicates whether the service exists in the Service Control Manager. This property is returned by
the resource and cannot be set. A value of `true` means the service is registered; `false` means
it is not found.

### status

<details><summary>Expand for <code>status</code> property metadata</summary>

```yaml
Type        : string
IsRequired  : false
IsKey       : false
IsReadOnly  : false
IsWriteOnly : false
Enum        : [Running, Stopped, Paused, StartPending, StopPending, PausePending, ContinuePending]
```

</details>

The runtime status of the service. When used as desired state in a **Set** operation, only the
following values are valid:

| Value     | Effect                                       |
|:----------|:---------------------------------------------|
| `Running` | DSC starts the service if it is not running. |
| `Stopped` | DSC stops the service if it is not stopped.  |
| `Paused`  | DSC pauses the service if it is not paused.  |

The following additional values may be returned by a **Get** or **Export** operation to describe a
transient state, but they must not be used as desired-state values:

- `StartPending`
- `StopPending`
- `PausePending`
- `ContinuePending`

### startType

<details><summary>Expand for <code>startType</code> property metadata</summary>

```yaml
Type        : string
IsRequired  : false
IsKey       : false
IsReadOnly  : false
IsWriteOnly : false
Enum        : [Automatic, AutomaticDelayedStart, Manual, Disabled]
```

</details>

Defines how the service is started. The following values are valid:

| Value                   | Description                                                                          |
|:------------------------|:-------------------------------------------------------------------------------------|
| `Automatic`             | The service is started automatically by the SCM at system startup.                   |
| `AutomaticDelayedStart` | The service starts automatically after other auto-start services have initialized.   |
| `Manual`                | The service is started only when explicitly requested (e.g., via `sc start`).        |
| `Disabled`              | The service cannot be started.                                                       |

### executablePath

<details><summary>Expand for <code>executablePath</code> property metadata</summary>

```yaml
Type        : string
IsRequired  : false
IsKey       : false
IsReadOnly  : false
IsWriteOnly : false
```

</details>

The fully qualified path to the service binary, including any command-line arguments registered
with the SCM. For example, `C:\Windows\System32\svchost.exe -k netsvcs`.

### logonAccount

<details><summary>Expand for <code>logonAccount</code> property metadata</summary>

```yaml
Type        : string
IsRequired  : false
IsKey       : false
IsReadOnly  : false
IsWriteOnly : false
```

</details>

The account under which the service process runs. Only the following built-in service accounts are
supported by the **Set** operation:

- `LocalSystem`
- `NT AUTHORITY\LocalService`
- `NT AUTHORITY\NetworkService`

Specifying a regular user account causes the **Set** operation to return an error.

### errorControl

<details><summary>Expand for <code>errorControl</code> property metadata</summary>

```yaml
Type        : string
IsRequired  : false
IsKey       : false
IsReadOnly  : false
IsWriteOnly : false
Enum        : [Ignore, Normal, Severe, Critical]
```

</details>

Controls the action taken if the service fails to start during system boot. The following values
are valid:

| Value      | Description                                                                                                                                                   |
|:-----------|:--------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `Ignore`   | The error is logged and startup continues.                                                                                                                    |
| `Normal`   | The error is logged, a message box is displayed, and startup continues.                                                                                       |
| `Severe`   | The error is logged. If the last-known-good configuration is in use, startup continues; otherwise the system restarts with the last-known-good configuration. |
| `Critical` | The error is logged. If the last-known-good configuration is in use, startup fails; otherwise the system restarts with the last-known-good configuration.     |

### dependencies

<details><summary>Expand for <code>dependencies</code> property metadata</summary>

```yaml
Type              : array
ItemsType         : string
ItemsMustBeUnique : false
IsRequired        : false
IsKey             : false
IsReadOnly        : false
IsWriteOnly       : false
```

</details>

A list of service key names that this service depends on. The SCM will not start the service until
all listed dependencies are running. Setting this property _replaces_ the existing dependency list
for the service.

## Instance validating schema

The following snippet contains the JSON Schema that validates an instance of the resource.

```json
{
  "type": "object",
  "additionalProperties": false,
  "properties": {
    "name": {
      "type": "string"
    },
    "displayName": {
      "type": "string"
    },
    "description": {
      "type": "string"
    },
    "_exist": {
      "type": "boolean",
      "readOnly": true
    },
    "status": {
      "type": "string",
      "enum": [
        "Running", "Stopped", "Paused",
        "StartPending", "StopPending", "PausePending", "ContinuePending"
      ]
    },
    "startType": {
      "type": "string",
      "enum": ["Automatic", "AutomaticDelayedStart", "Manual", "Disabled"]
    },
    "executablePath": {
      "type": "string"
    },
    "logonAccount": {
      "type": "string"
    },
    "errorControl": {
      "type": "string",
      "enum": ["Ignore", "Normal", "Severe", "Critical"]
    },
    "dependencies": {
      "type": "array",
      "items": {
        "type": "string"
      }
    }
  }
}
```

## Exit codes

The resource returns the following exit codes from operations:

- [0](#exit-code-0) - Success
- [1](#exit-code-1) - Invalid arguments
- [2](#exit-code-2) - Invalid input
- [3](#exit-code-3) - Service error

### Exit code 0

Indicates the resource operation completed without errors.

### Exit code 1

Indicates the resource operation failed because required arguments were missing or the operation
name was not recognized.

### Exit code 2

Indicates the resource operation failed because the JSON input could not be deserialized into a
valid `WindowsService` instance.

### Exit code 3

Indicates the resource operation failed due to an error raised by the Windows Service Control
Manager API, or the result could not be serialized.

## See also

- [Microsoft.Windows/Registry resource][06]
- [DSC resource capabilities][01]
- [DSC resource properties][07]

<!-- Link definitions -->
[01]: ../../../../../concepts/resources/capabilities.md
[02]: ./examples/get-service-status.md
[03]: ./examples/configure-windows-service.md
[04]: ../../../../../concepts/resources/properties.md#key-resource-properties
[05]: ../../../../../concepts/resources/properties.md#read-only-resource-properties
[06]: ../Registry/index.md
[07]: ../../../../../concepts/resources/properties.md
