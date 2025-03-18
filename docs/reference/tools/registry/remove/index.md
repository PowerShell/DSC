---
description: Command line reference for the 'registry remove' command
ms.date:     03/18/2025
ms.topic:    reference
title:       registry remove
---

# registry remove

## Synopsis

Remove a registry key or value.

> [!IMPORTANT]
> The `registry` command and `Microsoft.Windows/Registry` resource are a proof-of-concept example
> for use with DSCv3. Don't use it in production.

## Syntax

```sh
registry remove [Options] --key-path <KEY_PATH>
```

## Description

The `remove` command deletes a registry key or value. The [Microsoft.Windows/Registry] resource
uses this command for the **Delete** resource operation.

## Examples

### Example 1 - Remove a registry value

<a id="example-1"></a>

This example deletes the `ExampleValue` value on the `HKCU\Example\Key` registry key.

```powershell
registry remove --key-path HKCU\Example\Key --value-name ExampleValue
```

```Output
{"timestamp":"2025-03-17T20:43:48.472328Z","level":"DEBUG","fields":{"message":"Remove key_path: HKCU\\Example\\Key, value_name: Some(\"ExampleValue\"), recurse: false"},"target":"registry","line_number":47}
```

### Example 2 - Remove a registry key recursively

<a id="example-2"></a>

This example deletes the `HKCU\ExampleKey` registry key recursively. The command also deletes any
subkeys or values of the `HKCU\ExampleKey` key.

```powershell
registry remove --key-path HKCU\Example\Key --recurse
```

```Output
{"timestamp":"2025-03-17T20:44:13.597157Z","level":"DEBUG","fields":{"message":"Remove key_path: HKCU\\Example\\Key, value_name: None, recurse: true"},"target":"registry","line_number":47}
```

## Options

### -k, --key-path

<a id="-k"></a>
<a id="--key-path"></a>

Specifies the registry key path to remove. The path must start with a valid hive identifier. Each
segment of the path must be separated by a backslash (`\`).

The following table describes the valid hive identifiers for the key path.

| Short Name |       Long Name       |                                 NT Path                                 |
| :--------: | :-------------------: | :---------------------------------------------------------------------- |
|   `HKCR`   |  `HKEY_CLASSES_ROOT`  | `\Registry\Machine\Software\Classes\`                                   |
|   `HKCU`   |  `HKEY_CURRENT_USER`  | `\Registry\User\<User SID>\`                                            |
|   `HKLM`   | `HKEY_LOCAL_MACHINE`  | `\Registry\Machine\`                                                    |
|   `HKU`    |     `HKEY_USERS`      | `\Registry\User\`                                                       |
|   `HKCC`   | `HKEY_CURRENT_CONFIG` | `\Registry\Machine\System\CurrentControlSet\Hardware Profiles\Current\` |

```yaml
Type:      String
Mandatory: true
```

### -v, --value-name

<a id="-v"></a>
<a id="--value-name"></a>

Defines the name of the value to remove for in the specified registry key path.

```yaml
Type:      String
Mandatory: false
```

### -r, --recurse

<a id="-r"></a>
<a id="--recurse"></a>

Indicates whether the command should recursively remove subkeys. By default, the command isn't
recursive.

```yaml
Type:      boolean
Mandatory: false
```

### -h, --help

<a id="-h"></a>
<a id="--help"></a>

Displays the help for the current command or subcommand. When you specify this option, the
application ignores all options and arguments after this one.

```yaml
Type:      boolean
Mandatory: false
```
