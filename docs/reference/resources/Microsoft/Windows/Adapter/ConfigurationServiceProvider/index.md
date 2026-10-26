---
description: Microsoft.Windows.Adapter/ConfigurationServiceProvider resource adapter reference documentation
ms.date:     10/25/2026
ms.topic:    reference
title:       Microsoft.Windows.Adapter/ConfigurationServiceProvider
---

# Microsoft.Windows.Adapter/ConfigurationServiceProvider

## Synopsis

Adapts installed Windows Configuration Service Provider (CSP) bridge classes into DSC resources.

## Metadata

```yaml
Version           : 0.1.0
Kind              : adapter
Tags              : [Windows, CSP, MDM, WMI]
Executable        : csp
MinimumDSCVersion : 3.2.0
```

## Adapted resource instance definition syntax

### Implicitly required adapter syntax

```yaml
- name: Disable Wi-Fi internet sharing
  type: Microsoft.Windows.CSP/MDM_Policy_Config01_Wifi02
  properties:
    # All WMI key properties are required for get, set, and test.
    ParentID: ./Vendor/MSFT/Policy/Config
    InstanceID: WiFi
    # Optional writable properties retain their WMI names.
    AllowInternetSharing: 0
```

### Explicitly required adapter syntax

```yaml
- name: Disable Wi-Fi internet sharing
  type: Microsoft.Windows.CSP/MDM_Policy_Config01_Wifi02
  properties:
    ParentID: ./Vendor/MSFT/Policy/Config
    InstanceID: WiFi
    AllowInternetSharing: 0
  directives:
    requireAdapter: Microsoft.Windows.Adapter/ConfigurationServiceProvider
```

## Description

Windows [CSPs][01] expose management settings through a hierarchical collection of nodes. A node's
documentation defines its supported Windows versions, editions, scopes, data types, and operations.
The [MDM Bridge WMI provider][02] projects a subset of these nodes as WMI classes in
`root\cimv2\mdm\dmmap`. The bridge is not a general-purpose interface to every CSP URI.

This adapter uses the Rust [`wmi` crate][03], not PowerShell, to discover and invoke the local
bridge. Discovery enumerates class metadata, accepts `MDM_*` classes with the `DMWmiBridgeProv`
provider and a `local-system` or `local-user` partition, and exposes each as
`Microsoft.Windows.CSP/<WMI class name>`. The adapter does not cache class metadata.

The adapter implements **Get**, **Set**, **Test**, and **Export**. **Test** compares only specified
properties after conversion to their CIM types, and returns the actual state, `_inDesiredState`,
and a property-name diff. It does not
rely on DSC's synthetic test implementation.

**Set** updates an existing instance without replacing unspecified properties. If no instance
exists, it creates an instance with the supplied properties. Setting `_exist: false` deletes an
existing instance and succeeds without a write if the instance is already absent. Creation,
modification, and deletion remain subject to the underlying CSP's supported operations.

**Export** enumerates a selected class, not every class or every device setting. With no input,
it exports every instance visible in the caller's context. An optional input object filters on
any subset of the class's keys. Export excludes WMI system properties and unreadable properties.
For writable classes, it also excludes read-only non-key properties so exported instances can
be used as desired configuration. Read-only classes export their readable observational state.
An empty enumeration produces no instances, not a fabricated default configuration.

> [!NOTE]
> This adapter is included in Windows DSC packages. Updating DSC also updates the adapter.

## Requirements

- Windows with the MDM Bridge provider and the required CSP class installed.
- DSC 3.2 or later, which supports single-instance resource adapters.
- Run device-scoped classes (`InPartition("local-system")`) as **LocalSystem**, SID `S-1-5-18`.
  An elevated administrator is not equivalent to LocalSystem.
- Run user-scoped classes (`InPartition("local-user")`) as the user whose settings are managed.
  Elevation may also be required by the CSP. The adapter does not impersonate users, elevate,
  schedule tasks, or target another user's SID.

The adapter checks the effective caller token before instance operations. Discovery can list both
partitions without changing identity. A partition mismatch produces an actionable error instead
of treating inaccessible state as absent.

The adapter does not invoke CSP `Exec` methods, accept remote namespaces or arbitrary WQL, or
translate CSP URI paths into WMI class names. It does not apply XML/HTML encoding automatically;
string payloads must follow the selected CSP's documented format. Provider errors are propagated.
Operations are not transactional across resources.

The `wmi` crate converts an empty array to native `null` when writing. The adapter explicitly
rejects empty-array writes instead of silently changing their meaning. A JSON `null` is a WMI
null value, not a request to delete a CSP node; use `_exist: false` for instance deletion.

> [!WARNING]
> Exported values can include sensitive device or user configuration. Review exports before
> storing or sharing them. Write-only values cannot be exported or verified by reading them.

## Capabilities

The resource adapter has the following capabilities:

- `list` - Discover installed bridge classes and generate their instance schemas.
- `get` - Retrieve an instance identified by all its key properties.
- `set` - Create or update an instance of a class with writable non-key properties.
- `setHandlesExist` - Delete an instance during **Set** when `_exist` is `false`.
- `test` - Compare supplied properties with the actual state and report differing properties.
- `export` - Enumerate instances of a selected class, optionally filtered by keys.

Classes without writable non-key properties advertise only `get`, `test`, and `export`.
For writable classes, metadata indicates which properties are read-only; attempting to set such
a property fails before any provider write. A writable class does not guarantee that every
operation is supported by the CSP on the current Windows edition.

For more information about resource capabilities, see [DSC resource capabilities][04].

## Examples

1. [Discover and inspect CSP resources][05]
2. [Configure, test, and export a device CSP][06]

## Properties

Adapted resources use the exact WMI property names and CIM types reported by the installed class.
Names are case-sensitive in JSON. The adapter itself has no configurable instance properties.

**Required and key properties:** All properties with WMI's `Key` qualifier are required by
**Get**, **Set**, and **Test**. Most bridge classes use [ParentID](#parentid) and
[InstanceID](#instanceid). **Export** accepts no keys or a partial key filter.

**Instance properties:** Properties with the WMI `write` qualifier are optional. Only supplied
properties are modified. [_exist](#_exist) controls existence.

**Read-only properties:** Other WMI properties and [_inDesiredState](#_indesiredstate) cannot be
configured. The generated schema marks read-only properties with `readOnly: true`.

### ParentID

<details><summary>Expand for <code>ParentID</code> property metadata</summary>

```yaml
Type       : string
IsRequired : true
IsKey      : true
IsReadOnly : false
IsWriteOnly: false
```

</details>

The CSP parent path, for example `./Vendor/MSFT/Policy/Config`. Consult the selected bridge class
and CSP documentation for the correct value. The adapter does not invent a default parent.

### InstanceID

<details><summary>Expand for <code>InstanceID</code> property metadata</summary>

```yaml
Type       : string
IsRequired : true
IsKey      : true
IsReadOnly : false
IsWriteOnly: false
```

</details>

The instance's identifier under its parent, for example `WiFi`. The combination of key values
identifies one instance. Missing keys fail input validation rather than selecting a first instance.

### _exist

<details><summary>Expand for <code>_exist</code> property metadata</summary>

```yaml
Type        : boolean
IsRequired  : false
IsKey       : false
IsReadOnly  : false
IsWriteOnly : false
DefaultValue: true
```

</details>

Returned as `true` for an existing instance and `false` for an absent instance. An absent state
contains only the supplied keys and `_exist: false`. For writable classes, `_exist: false`
requests deletion during **Set**. **Test** compares existence only when absence is requested.
Omitting `_exist` during **Set** means that the instance should exist.

### _inDesiredState

<details><summary>Expand for <code>_inDesiredState</code> property metadata</summary>

```yaml
Type        : boolean
IsRequired  : false
IsKey       : false
IsReadOnly  : true
IsWriteOnly : false
```

</details>

Returned by **Test**. It is `true` when the specified desired properties match, including
existence, and `false` otherwise. Do not include this property in desired input.

## Instance validating schema

Each discovered resource includes a JSON Schema directly in its `schema` field, generated from
its installed class. CIM integer widths determine numeric ranges; Boolean values remain JSON booleans, string values
remain strings, and CIM arrays become typed JSON arrays. Unknown properties and incompatible
types are rejected. Non-key properties can be null.

To inspect an actual schema on the target machine:

```powershell
$resource = dsc resource list 'Microsoft.Windows.CSP/MDM_Policy_Config01_Wifi02' `
    --adapter Microsoft.Windows.Adapter/ConfigurationServiceProvider | ConvertFrom-Json
$resource.schema | ConvertTo-Json -Depth 20
```

Keys are checked per operation rather than marked globally required in the schema, because export
allows partial key filters. Schemas describe WMI data types, not every CSP-specific allowed value.

## Exit codes

The adapter writes a JSON error object to stderr for every unsuccessful operation.

- [0](#exit-code-0) - Success.
- [1](#exit-code-1) - Invalid command-line arguments.
- [2](#exit-code-2) - Invalid resource type, input, or unsupported property.
- [3](#exit-code-3) - WMI Bridge operation or execution-context check failed.

### Exit code 0

The operation completed without errors.

### Exit code 1

A required command-line argument is missing or an argument is unrecognized.

### Exit code 2

Correct the resource name, key properties, property names, data types, or writable-property
selection. This code also reports unsupported empty-array writes.

### Exit code 3

Check the caller's identity, CSP permissions, supported Windows version and edition, and provider
diagnostics. Access-denied and unsupported-operation failures are not interpreted as absent state.

## See also

- [Configuration service providers][01]
- [MDM Bridge WMI provider][02]
- [Using scripting with the WMI Bridge Provider][07]
- [Microsoft.Windows/WMI](../../WMI/index.md)

<!-- Link definitions -->
[01]: /windows/client-management/mdm/
[02]: /windows/win32/dmwmibridgeprov/mdm-bridge-wmi-provider-portal
[03]: https://docs.rs/wmi/0.18.4/wmi/
[04]: ../../../../../../concepts/resources/capabilities.md
[05]: ./examples/discover-csp-resources.md
[06]: ./examples/configure-csp.md
[07]: /windows/client-management/using-powershell-scripting-with-the-wmi-bridge-provider
