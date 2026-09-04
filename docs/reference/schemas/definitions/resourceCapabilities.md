---
description: JSON schema reference for resource capabilities
ms.date:     09/01/2026
ms.topic:    reference
title:       DSC Resource capabilities schema reference
---

# DSC Resource capabilities schema reference

## Synopsis

Defines the operations you can invoke for a resource and how the resource behaves when invoked.

## Metadata

```yaml
SchemaDialect:     https://json-schema.org/draft/2020-12/schema
SchemaID:          https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/definitions/resourceCapabilities.json
Type:              array
Required:          true
ItemsMustBeUnique: true
ItemsType:         string
ItemsValidValues: [
                    get,
                    set,
                    setHandlesExist,
                    setWhatIf,
                    test,
                    delete,
                    deleteWhatIf,
                    export,
                    resolve
                  ]
```

## Description

DSC resources always have at least one capability. Resource capabilities define the operations you
can invoke for a resource and how the resource behaves when invoked. DSC reports the capabilities
of every discovered resource in the output of the `dsc resource list` command.

DSC resources may have the following capabilities:

- `get` - The resource supports retrieving the current state of an instance.
- `set` - The resource supports enforcing the desired state for an instance.
- `setHandlesExist` - The resource handles the `_exist` canonical property directly during a
  **Set** operation, including removing an instance when `_exist` is `false`.
- `setWhatIf` - The resource supports simulating the **Set** operation directly, reporting how it
  would change the state of an instance without changing it. This capability was added in DSC
  version 3.3.0. Through DSC version 3.2.x, this capability was reported as `whatIf`.
- `test` - The resource implements the **Test** operation and doesn't rely on synthetic testing.
- `delete` - The resource supports removing an instance.
- `deleteWhatIf` - The resource supports simulating the **Delete** operation directly, reporting
  how it would remove an instance without removing it. This capability was added in DSC version
  3.3.0.
- `export` - The resource supports enumerating every instance.
- `resolve` - The resource supports resolving nested instances from an external source.

### Capabilities for command resources

DSC infers the capabilities of a command resource from the properties defined in its resource
manifest:

| Capability        | Manifest properties                                                  |
|:------------------|:---------------------------------------------------------------------|
| `get`             | [get][01]                                                            |
| `set`             | [set][02]                                                            |
| `setHandlesExist` | [set][02] with `handlesExist` set to `true`                          |
| `setWhatIf`       | [set][02] with a `whatIfArg` item in `args`, or [whatIf][03]         |
| `test`            | [test][04]                                                           |
| `delete`          | [delete][05]                                                         |
| `deleteWhatIf`    | [delete][05] with a `whatIfArg` item in `args`                       |
| `export`          | [export][06]                                                         |
| `resolve`         | [resolve][07]                                                        |

For more information about resource capabilities, see [DSC resource capabilities][08]. For more
information about the operations you can invoke for a resource, see [DSC resource operations][09].

<!-- Link reference definitions -->
[01]: ../resource/manifest/get.md
[02]: ../resource/manifest/set.md
[03]: ../resource/manifest/whatif.md
[04]: ../resource/manifest/test.md
[05]: ../resource/manifest/delete.md
[06]: ../resource/manifest/export.md
[07]: ../resource/manifest/resolve.md
[08]: ../../../concepts/resources/capabilities.md
[09]: ../../../concepts/resources/operations.md
