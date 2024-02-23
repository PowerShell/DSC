---
description: Command line reference for the 'dsc resource get' command
ms.date:     09/06/2023
ms.topic:    reference
title:       dsc resource get
---

# dsc resource get

## Synopsis

Invokes the get operation of a resource.

## Syntax

### Without instance properties

```sh
dsc resource get [Options] --resource <RESOURCE>
```

### Instance properties from stdin

```sh
<instance-properties> | dsc resource get [Options] --resource <RESOURCE>
```

### Instance properties from input option

```sh
dsc resource get --input '<instance-properties>' --resource <RESOURCE>
```

### Instance properties from file

```sh
dsc resource get --path <instance-properties-filepath> --resource <RESOURCE>
```

## Description

The `get` subcommand returns the current state of a resource instance.

By default, this subcommand returns one instance from a specific DSC Resource. To return multiple
resources, use the `--all` parameter, a resource group, or the [dsc config get][01] command.

Any properties the resource requires for retrieving the state of an instance must be passed to this
command as a JSON or YAML object. The object can be passed to this command from stdin or with the
`--input` option. You can also use the `--path` option to read the object from a JSON or YAML file.

## Examples

### Example 1 - Get resource instance without any input

For single-instance resources that don't require any property values to return the actual
state of the resource instance, the instance properties aren't required.

```sh
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

### Example 2 - Get resource instance with input option

If a resource requires one or more property values to return the actual state of the instance, the
instance properties can be passed with the **input** option as either JSON or YAML.

```sh
dsc resource get --resource Microsoft.Windows/Registry --input '{
    "keyPath": "HKLM\\Software\\Microsoft\\Windows NT\\CurrentVersion",
    "valueName": "SystemRoot"
}'
```

```yaml
actualState:
  $id: https://developer.microsoft.com/json-schemas/windows/registry/20230303/Microsoft.Windows.Registry.schema.json
  keyPath: HKLM\Software\Microsoft\Windows NT\CurrentVersion
  valueName: SystemRoot
  valueData:
    String: C:\WINDOWS
```

### Example 3 - Get resource instance with input from stdin

If a resource requires one or more property values to return the actual state of the instance, the
instance properties can be passed over stdin as either JSON or YAML.

```sh
'{
    "keyPath": "HKLM\\Software\\Microsoft\\Windows NT\\CurrentVersion",
    "valueName": "SystemRoot"
}' | dsc resource get --resource Microsoft.Windows/Registry
```

```yaml
actualState:
  $id: https://developer.microsoft.com/json-schemas/windows/registry/20230303/Microsoft.Windows.Registry.schema.json
  keyPath: HKLM\Software\Microsoft\Windows NT\CurrentVersion
  valueName: SystemRoot
  valueData:
    String: C:\WINDOWS
```

### Example 4 - Get resource instance with input from a YAML file

If a resource requires one or more property values to return the actual state of the instance, the
instance properties can be retrieved from a saved JSON or YAML file.

```sh
cat ./example.yaml
```

```yaml
keyPath:   HKLM\\Software\\Microsoft\\Windows NT\\CurrentVersion
valueName: SystemRoot
```

```sh
dsc resource get --resource Microsoft.Windows/Registry --path ./example.yaml
```

```yaml
actualState:
  $id: https://developer.microsoft.com/json-schemas/windows/registry/20230303/Microsoft.Windows.Registry.schema.json
  keyPath: HKLM\Software\Microsoft\Windows NT\CurrentVersion
  valueName: SystemRoot
  valueData:
    String: C:\WINDOWS
```

## Options

### -a, --all

Specifies that the command should return every instance of the specified DSC Resource instead of a
specific instance.

This option is only valid when the Resource is an exportable resource that defines the
[export][02] section in the input configuration. If the resource type isn't exportable, DSC raises
an error.

When this option is specified, DSC ignores the `--input` and `--path` options and any JSON or YAML
sent to the command from stdin.

```yaml
Type:      Boolean
Mandatory: false
```

### -r, --resource

Specifies the fully qualified type name of the DSC Resource to use, like
`Microsoft.Windows/Registry`.

The fully qualified type name syntax is: `<owner>[.<group>][.<area>]/<name>`, where:

- The `owner` is the maintaining author or organization for the resource.
- The `group` and `area` are optional name components that enable namespacing for a resource.
- The `name` identifies the component the resource manages.

```yaml
Type:      String
Mandatory: true
```

### -i, --input

Specifies a JSON or YAML object with the properties needed for retrieving an instance of the DSC
Resource. DSC validates the object against the resource's instance schema. If the validation fails,
DSC raises an error.

This option can't be used with instance properties over stdin or the `--path` option. Choose
whether to pass the instance properties to the command over stdin, from a file with the `--path`
option, or with the `--input` option.

DSC ignores this option when the `--all` option is specified.

```yaml
Type:      String
Mandatory: false
```

### -p, --path

Defines the path to a text file to read as input for the command instead of piping input from stdin
or passing it as a string with the `--input` option. The specified file must contain JSON or YAML
that represents valid properties for the resource. DSC validates the object against the resource's
instance schema. If the validation fails, or if the specified file doesn't exist, DSC raises an
error.

This option is mutually exclusive with the `--input` option. When you use this option, DSC
ignores any input from stdin.

DSC ignores this option when the `--all` option is specified.

```yaml
Type:      String
Mandatory: false
```

### -f, --format

The `--format` option controls the console output format for the command. If the command output is
redirected or captured as a variable, the output is always JSON.

```yaml
Type:         String
Mandatory:    false
DefaultValue: yaml
ValidValues:  [json, pretty-json, yaml]
```

### -h, --help

Displays the help for the current command or subcommand. When you specify this option, the
application ignores all options and arguments after this one.

```yaml
Type:      Boolean
Mandatory: false
```

## Output

By default, this command returns JSON output that includes the actual state of the instance. When
the `--all` option is specified, the command returns the JSON output for each instance as
[JSON Lines][03].

For more information about the structure of the output JSON, see
[dsc resource get result schema][04].

[01]: ../config/get.md
[02]: ../../schemas/resource/manifest/export.md
[03]: https://jsonlines.org/
[04]: ../../schemas/outputs/resource/get.md
