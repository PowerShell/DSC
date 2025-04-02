---
description: Command line reference for the 'registry query' command
ms.date:     03/25/2025
ms.topic:    reference
title:       registry query
---

# registry query

## Synopsis

Query a registry key or value.

> [!IMPORTANT]
> The `registry` command and `Microsoft.Windows/Registry` resource are a proof-of-concept example
> for use with DSCv3. Don't use it in production.

## Syntax

```sh
registry query [Options] --key-path <KEY_PATH>
```

## Description

The `query` command isn't implemented yet. It returns a string that echoes the specified options.

## Examples

### Example 1 - Echo the options

<a id="example-1"></a>

The options are returned as a string on a single line.

```powershell
registry query --key-path HKCU\SYSTEM --recurse
```

```Output
Get key_path: HKCU\SYSTEM, value_name: None, recurse: true
```

## Options

### -k, --key-path

<a id="-k"></a>
<a id="--key-path"></a>

Specifies the registry key path to query. The path must start with a valid hive identifier. Each
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

Defines the name of the value to query for in the specified registry key path.

```yaml
Type:      String
Mandatory: false
```

### -r, --recurse

<a id="-r"></a>
<a id="--recurse"></a>

Indicates whether the command should recursively query subkeys. By default, the command isn't
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
