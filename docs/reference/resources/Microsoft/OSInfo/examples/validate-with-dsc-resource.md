---
description: >
  Validate operating system information with the Microsoft/OSInfo DSC Resource
  and the dsc resource commands.
ms.date: 03/25/2025
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
```

---

## Testing the operating system information

DSC can use the resource to validate the operating system information. When you use the
[dsc resource test][02] command, input JSON representing the desired state of the instance is
required. The JSON must define at least one instance property to validate.

The resource doesn't implement the [test operation][03]. It relies on the synthetic testing feature
of DSC instead. The synthetic test uses a case-sensitive equivalency comparison between the actual
state of the instance properties and the desired state. If any property value isn't an exact match,
DSC considers the instance to be out of the desired state.

# [Linux](#tab/linux)

This test checks whether the `family` property for the instance is `Linux`. It passes the desired
state for the instance to the command from stdin with the `--file` (`-f`) option.

```bash
invalid_instance='{"family": "Linux"}'
echo $invalid_instance | dsc resource test -r "${resource}" -f -
```

```yaml
desiredState:
  family: linux
actualState:
  $id: https://developer.microsoft.com/json-schemas/dsc/os_info/20230303/Microsoft.Dsc.OS_Info.schema.json
  family: Linux
  version: '20.04'
  codename: focal
  bitness: '64'
  architecture: x86_64
inDesiredState: false
differingProperties:
- family
```

The result shows that the resource is out of the desired state because the actual state of the
`family` property wasn't case-sensitively equal to the desired state.

The next test validates that the operating system is a 64-bit Linux operating system. It passes
the desired state for the instance to the command with the `--input` (`-i`) option.

```bash
valid_instance='{ "family": "Linux", "bitness": "64" }'
echo $valid_instance | dsc resource test -r Microsoft/OSInfo -i $valid_instance
```

```yaml
desiredState:
  family: Linux
  bitness: '64'
actualState:
  $id: https://developer.microsoft.com/json-schemas/dsc/os_info/20230303/Microsoft.Dsc.OS_Info.schema.json
  family: Linux
  version: '20.04'
  codename: focal
  bitness: '64'
  architecture: x86_64
inDesiredState: true
differingProperties: []
```

# [macOS](#tab/macos)

This test checks whether the `family` property for the instance is `macOS`. It passes the desired
state for the instance to the command from stdin with the `--file` (`-f`) option.

```zsh
invalid_instance='{"family": "macOS"}'
echo $invalid_instance | dsc resource test -r Microsoft/OSInfo -f -
```

```yaml
desiredState:
  family: macOS
actualState:
  $id: https://developer.microsoft.com/json-schemas/dsc/os_info/20230303/Microsoft.Dsc.OS_Info.schema.json
  family: MacOS
  version: 13.5.0
  bitness: '64'
  architecture: arm64
inDesiredState: false
differingProperties:
- family
```

The result shows that the resource is out of the desired state because the actual state of the
`family` property wasn't case-sensitively equal to the desired state.

The next test validates that the operating system is a 64-bit macOS operating system. It passes the
desired state for the instance to the command with the `--input` (`-i`) option.

```zsh
valid_instance='{ "family": "MacOS", "bitness": "64" }'
dsc resource test -r Microsoft/OSInfo -i $valid_instance
```

```yaml
desiredState:
  family: MacOS
  bitness: '64'
actualState:
  $id: https://developer.microsoft.com/json-schemas/dsc/os_info/20230303/Microsoft.Dsc.OS_Info.schema.json
  family: MacOS
  version: 13.5.0
  bitness: '64'
  architecture: arm64
inDesiredState: true
differingProperties: []
```

# [Windows](#tab/windows)

This test checks whether the `family` property for the instance is `windows`. It passes the desired
state for the instance to the command from stdin with the `--file` (`-f`) option.

```powershell
$invalidInstance = @{ family = 'windows' } | ConvertTo-JSON
$invalidInstance | dsc resource test --resource Microsoft/OSInfo --file -
```

```yaml
desiredState:
  family: windows
actualState:
  $id: https://developer.microsoft.com/json-schemas/dsc/os_info/20230303/Microsoft.Dsc.OS_Info.schema.json
  family: Windows
  version: 10.0.22621
  edition: Windows 11 Enterprise
  bitness: "64"
inDesiredState: false
differingProperties:
- family
```

The result shows that the resource is out of the desired state because the actual state of the
`family` property wasn't case-sensitively equal to the desired state.

The next test validates that the operating system is a 64-bit Windows operating system. It passes
the desired state for the instance to the command with the `--input` (`-i`) option.

```powershell
$validInstance = @{
    family  = 'Windows'
    bitness = '64'
} | ConvertTo-JSON

dsc resource test --resource Microsoft/OSInfo --input $validInstance
```

```yaml
desiredState:
  family: Windows
  bitness: '64'
actualState:
  $id: https://developer.microsoft.com/json-schemas/dsc/os_info/20230303/Microsoft.Dsc.OS_Info.schema.json
  family: Windows
  version: 10.0.22621
  edition: Windows 11 Enterprise
  bitness: "64"
inDesiredState: true
differingProperties: []
```

---

<!-- Link reference -->
[01]: ../../../../cli/resource/get.md
[02]: ../../../../cli/resource/test.md
[03]: ../../../../../concepts/resources/overview.md#test-operations
