---
description: Command line reference for the 'registry find' command
ms.date:     03/18/2025
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

The `find` command isn't implemented yet. It returns a string that echoes the specified options.

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
