---
description: >
  Validate operating system information with the Microsoft/OSInfo DSC Resource
  and the dsc resource commands.
ms.date: 07/12/2026
ms.topic: reference
title: Validate operating system information with dsc resource
---

# Validate operating system information with dsc resource

## Description

This example shows how you can use the `Microsoft/OSInfo` DSC Resource to retrieve and validate
information about an operating system with the `dsc resource` commands.

> [!IMPORTANT]
> The `osinfo` command and `Microsoft/OSInfo` resource are a proof-of-concept example for use with
> DSC. Don't use it in production.

## Getting the operating system information

The [dsc resource get][01] command returns an instance of the resource. The `Microsoft/OSInfo`
resource doesn't require any instance properties to return the instance. The resource returns the
available information for the operating system.

# [Linux](#tab/linux)

```bash
dsc resource get -r Microsoft/OSInfo
```

```yaml
actualState:
  $id: https://developer.microsoft.com/json-schemas/dsc/os_info/20230303/Microsoft.Dsc.OS_Info.schema.json
  family: Linux
  version: '20.04'
  codename: focal
  bitness: '64'
  architecture: x86_64
```

# [macOS](#tab/macos)

```zsh
resource=$(dsc resource list Microsoft/OSInfo)
dsc resource get -r Microsoft/OSInfo
```

```yaml
actualState:
  $id: https://developer.microsoft.com/json-schemas/dsc/os_info/20230303/Microsoft.Dsc.OS_Info.schema.json
  family: MacOS
  version: 13.5.0
  bitness: '64'
  architecture: arm64
```

# [Windows](#tab/windows)

```powershell
dsc resource get --resource Microsoft/OSInfo
```

```yaml
actualState:
  $id: https://developer.microsoft.com/json-schemas/dsc/os_info/20230303/Microsoft.Dsc.OS_Info.schema.json
  family: Windows
  version: 10.0.22621
  edition: Windows 11 Enterprise
  bitness: '64'
  architecture: x86_64
```

---

## Testing the operating system information

DSC can use the resource to validate the operating system information. When you use the
[dsc resource test][02] command, input JSON representing the desired state of the instance is
required. The JSON must define at least one instance property to validate.

The resource implements the [test operation][03]. The command passes the desired state to the
resource over stdin and the resource returns the actual operating system information with an
`_inDesiredState` value. DSC returns that value as `inDesiredState` in the test result.

All properties except `version` use case-sensitive equality comparison. For `version`, you can use
an exact version or a constraint with `>`, `<`, `=`, `>=`, or `<=`. For more information, see the
[version property][04] reference.

# [Linux](#tab/linux)

This test checks whether the `family` property for the instance is `Linux`. It passes the desired
state for the instance to the command from stdin with the `--file` (`-f`) option.

```bash
valid_instance='{"family": "Linux", "version": ">= 20.04"}'
echo $valid_instance | dsc resource test -r Microsoft/OSInfo -f -
```

```yaml
desiredState:
  family: Linux
  version: '>= 20.04'
actualState:
  family: Linux
  version: '20.04'
  codename: focal
  bitness: 64
  architecture: x86_64
  _inDesiredState: true
inDesiredState: true
differingProperties: []
```

The result shows that the resource evaluated both the family and version constraint successfully.
The next test demonstrates a case-sensitive mismatch.

```bash
invalid_instance='{ "family": "linux" }'
dsc resource test -r Microsoft/OSInfo -i $invalid_instance
```

```yaml
desiredState:
  family: linux
actualState:
  family: Linux
  version: '20.04'
  codename: focal
  bitness: 64
  architecture: x86_64
  _inDesiredState: false
inDesiredState: false
differingProperties:
- family
```

# [macOS](#tab/macos)

This test checks whether the `family` property for the instance is `macOS`. It passes the desired
state for the instance to the command from stdin with the `--file` (`-f`) option.

```zsh
valid_instance='{"family": "macOS", "version": ">= 13.0"}'
echo $valid_instance | dsc resource test -r Microsoft/OSInfo -f -
```

```yaml
desiredState:
  family: macOS
  version: '>= 13.0'
actualState:
  family: macOS
  version: 13.5.0
  bitness: 64
  architecture: arm64
  _inDesiredState: true
inDesiredState: true
differingProperties: []
```

The result shows that the resource evaluates the version constraint in addition to the family.
The next test demonstrates a case-sensitive mismatch.

```zsh
invalid_instance='{ "family": "MacOS" }'
dsc resource test -r Microsoft/OSInfo -i $invalid_instance
```

```yaml
desiredState:
  family: MacOS
actualState:
  family: macOS
  version: 13.5.0
  bitness: 64
  architecture: arm64
  _inDesiredState: false
inDesiredState: false
differingProperties:
- family
```

# [Windows](#tab/windows)

This test checks whether the `family` property for the instance is `Windows` and whether the
operating system version is at least `10.0`. It passes the desired state for the instance to the
command from stdin with the `--file` (`-f`) option.

```powershell
$validInstance = @{ family = 'Windows'; version = '>= 10.0' } | ConvertTo-JSON
$validInstance | dsc resource test --resource Microsoft/OSInfo --file -
```

```yaml
desiredState:
  family: Windows
  version: '>= 10.0'
actualState:
  family: Windows
  version: 10.0.22621
  edition: Windows 11 Enterprise
  bitness: 64
  architecture: x86_64
  _inDesiredState: true
inDesiredState: true
differingProperties: []
```

The result shows that the resource evaluated both the family and version constraint successfully.
The next test demonstrates a case-sensitive mismatch.

```powershell
$invalidInstance = @{ family = 'windows' } | ConvertTo-JSON

dsc resource test --resource Microsoft/OSInfo --input $invalidInstance
```

```yaml
desiredState:
  family: windows
actualState:
  family: Windows
  version: 10.0.22621
  edition: Windows 11 Enterprise
  bitness: 64
  architecture: x86_64
  _inDesiredState: false
inDesiredState: false
differingProperties:
- family
```

---

<!-- Link reference -->
[01]: ../../../../cli/resource/get.md
[02]: ../../../../cli/resource/test.md
[03]: ../../../../../concepts/resources/overview.md#test-operations
[04]: ../index.md#version
