---
description: Command line reference for the 'dsc resource get' command
ms.date:     3/05/2025
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

### Instance properties from input option

```sh
dsc resource get --input <INPUT> --resource <RESOURCE>
```

### Instance properties from file

```sh
dsc resource get --file <FILE> --resource <RESOURCE>
```

### Instance properties from stdin

```sh
cat <FILE> | dsc resource get [Options] --resource <RESOURCE> --file -
```

## Description

The `get` subcommand returns the current state of a resource instance.

By default, this subcommand returns one instance from a specific DSC Resource. To return multiple
resources, use the `--all` parameter, a resource group, or the [dsc config get][01] command.

Any properties the resource requires for retrieving the state of an instance must be passed to this
command as a JSON or YAML object with the `--input` or `--file` option.

## Examples

### Example 1 - Get resource instance without any input

<a id="example-1"></a>

For single-instance resources that don't require any property values to return the actual state of
the resource instance, the instance properties aren't required.

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

<a id="example-2"></a>

If a resource requires one or more property values to return the actual state of the instance, the
instance properties can be passed with the `--input` option as either JSON or YAML.

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

<a id="example-3"></a>

If a resource requires one or more property values to return the actual state of the instance, the
instance properties can be passed over stdin as either JSON or YAML with the `--file` option.

```sh
'{
    "keyPath": "HKLM\\Software\\Microsoft\\Windows NT\\CurrentVersion",
    "valueName": "SystemRoot"
}' | dsc resource get --resource Microsoft.Windows/Registry --file -
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

<a id="example-4"></a>

If a resource requires one or more property values to return the actual state of the instance, the
instance properties can be retrieved from a saved JSON or YAML file.

```yaml
# ./example.yaml
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

<a id="-r"></a>
<a id="--resource"></a>

Specifies that the command should return every instance of the specified DSC Resource instead of a
specific instance.

This option is only valid when the Resource is an exportable resource that defines the
[export][02] section in the input configuration. If the resource type isn't exportable, DSC raises
an error.

When this option is specified, DSC ignores the `--input` and `--path` options.

```yaml
Type:      Boolean
Mandatory: false
```

### -r, --resource

<a id="-r"></a>
<a id="--resource"></a>

Specifies the fully qualified type name of the DSC Resource to use, like
`Microsoft.Windows/Registry`.

The fully qualified type name syntax is: `<owner>[.<group>][.<area>]/<name>`, where:

- The `owner` is the maintaining author or organization for the resource.
- The `group` and `area` are optional name components that enable namespacing for a resource.
- The `name` identifies the component the resource manages.

```yaml
Type        : string
Mandatory   : true
LongSyntax  : --resource <RESOURCE>
ShortSyntax : -r <RESOURCE>
```

### -i, --input

<a id="-i"></a>
<a id="--input"></a>

Specifies the resource instance to retrieve.

The instance must be a string containing a JSON or YAML object. DSC validates the object against
the resource's instance schema. If the validation fails, DSC raises an error.

This option is mutually exclusive with the `--file` option.

```yaml
Type        : string
Mandatory   : false
LongSyntax  : --input <INPUT>
ShortSyntax : -i <INPUT>
```

### -f, --file

<a id="-f"></a>
<a id="--file"></a>

Defines the path to a file defining  the resource instance to retrieve.

The specified file must contain a JSON or YAML object that represents valid properties for the
resource. DSC validates the object against the resource's instance schema. If the validation fails,
or if the specified file doesn't exist, DSC raises an error.

You can also use this option to pass an instance from stdin, as shown in [Example 3](#example-3).

This option is mutually exclusive with the `--input` option.

```yaml
Type        : string
Mandatory   : false
LongSyntax  : --file <FILE>
ShortSyntax : -f <FILE>
```

### -o, --output-format

<a id="-o"></a>
<a id="--output-format"></a>

The `--output-format` option controls which format DSC uses for the data the command returns. The
available formats are:

- `json` to emit the data as a [JSON Line][aa]. When you use the [--all option](#--all), each instance is returned as a JSON Line.
- `pretty-json` to emit the data as JSON with newlines, indentation, and spaces for readability.
- `yaml` to emit the data as YAML. When you use the `--all` option, each instance is returned as a
  YAML document with the `---` document separator between each returned instance.

The default output format depends on whether DSC detects that the output is being redirected or
captured as a variable:

- If the command isn't being redirected or captured, DSC displays the output as the `yaml` format
  in the console.
- If the command output is redirected or captured, DSC emits the data as the `json` format to
  stdout.

When you use this option, DSC uses the specified format regardless of whether the command is being
redirected or captured.

When the command isn't redirected or captured, the output in the console is formatted for improved
readability. When the command isn't redirected or captured, the output include terminal sequences
for formatting.

```yaml
Type        : string
Mandatory   : false
ValidValues : [json, pretty-json, yaml]
LongSyntax  : --output-format <OUTPUT_FORMAT>
ShortSyntax : -o <OUTPUT_FORMAT>
```

[aa]: https://jsonlines.org/

### -h, --help

<a id="-h"></a>
<a id="--help"></a>

Displays the help for the current command or subcommand. When you specify this option, the
application ignores all other options and arguments.

```yaml
Type        : boolean
Mandatory   : false
LongSyntax  : --help
ShortSyntax : -h
```

## Output

By default, this command returns a formatted data object that includes the actual state of the instance. When
the `--all` option is specified, the command returns the formatted data for each instance.

For more information about the structure of the output JSON, see
[dsc resource get result schema][04].

For more information about the formatting of the output data, see the
[--output-format option](#--output-format).

[01]: ../config/get.md
[02]: ../../schemas/resource/manifest/export.md
[03]: https://jsonlines.org/
[04]: ../../schemas/outputs/resource/get.md
