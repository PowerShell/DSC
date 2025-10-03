---
description: Command line reference for the 'registry find' command
ms.date:     03/25/2025
ms.topic:    reference
title:       registry find
---

# registry find

## Synopsis

Find a registry key or value.

> [!IMPORTANT]
> The `registry` command and `Microsoft.Windows/Registry` resource are a proof-of-concept example
> for use with DSCv3. Don't use it in production.

## Syntax

```sh
registry find [Options] --key-path <KEY_PATH>
```

## Description

The `find` command searches for registry keys and values that match the specified search string.
You can use this command to locate specific registry entries by name, optionally searching
recursively through subkeys or limiting results to keys or values only.

## Examples

### Example 1 - Find keys and values

<a id="example-1"></a>

Search for registry keys and values containing "System" in a specific path.

```powershell
registry find --key-path "HKLM\SOFTWARE\Microsoft" --find "System"
```

```output
Find key_path: HKLM\\SOFTWARE\\Microsoft, find: System, recurse: false, keys_only: false, values_only: false
```

### Example 2 - Recursive search for keys only

<a id="example-2"></a>

Recursively search for registry keys containing "Windows".

```powershell
registry find --key-path "HKLM\SOFTWARE" --find "Windows" --recurse
```

```output
Find key_path: HKLM\\SOFTWARE, find: Windows, recurse: true, keys_only: false, values_only: false
```

### Example 3 - Find values only

<a id="example-3"></a>

Search for registry values containing "Desktop".

```powershell
registry find --key-path "HKCU\Environment" --find "Desktop" --values-only
```

```output
Find key_path: HKCU\\Environment, find: Desktop, recurse: false, keys_only: false, values_only: true
```

## Options

### -k, --key-path

<a id="-k"></a>
<a id="--key-path"></a>

Specifies the registry key path to use as the base for the search. The path must start with a valid
hive identifier. Each segment of the path must be separated by a backslash (`\`).

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

### -f, --find

<a id="-f"></a>
<a id="--find"></a>

The string to search for in the registry key and value names.

```yaml
Type:      String
Mandatory: true
```

### -r, --recurse

<a id="-r"></a>
<a id="--recurse"></a>

Indicates whether the command should recursively find subkeys and values. By default, the command
isn't recursive.

```yaml
Type:      boolean
Mandatory: false
```

### --keys_only

<a id="--keys_only"></a>

Indicates whether the command should limit results to registry keys. By default, the command
returns matching registry keys and values.

```yaml
Type:      boolean
Mandatory: false
```

### --values_only

<a id="--values_only"></a>

Indicates whether the command should limit results to registry values. By default, the command
returns matching registry keys and values.

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
