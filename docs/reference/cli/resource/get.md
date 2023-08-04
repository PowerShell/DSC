# dsc resource get

## Synopsis

Invokes the get operation of a resource.

## Syntax

```sh
dsc resource get [Options] --resource <RESOURCE>
```

## Description

The `get` subcommand returns the current state of a resource instance.

This subcommand returns one instance from a specific DSC Resource. To return multiple resources,
use a resource group or the [dsc config get][01] command.

Any properties the resource requires for retrieving the state of an instance must be passed to this
command as JSON. The JSON can be passed to this command from stdin or with the `--input` option.

## Examples

### Example 1 - Get resource instance without any input

For resources single-instance resources that don't require any property values to return the actual
state of the resource instance, the JSON input isn't required.

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
JSON object can be passed with the **input** option.

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
JSON object can be passed over stdin.

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

## Options

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

Specifies a JSON object with the properties needed for retrieving an instance of the DSC Resource.
DSC validates the JSON against the resource's instance schema. If the validation fails, DSC raises
an error.

This option can't be used with JSON over stdin. Choose whether to pass the instance JSON to the
command over stdin or with the `--input` flag.

```yaml
Type:      String
Mandatory: false
```

### -h, --help

Displays the help for the current command or subcommand. When you specify this option, the
application ignores all options and arguments after this one.

```yaml
Type:      Boolean
Mandatory: false
```

## Output

This command returns JSON output that includes the actual state of the instance. For more
information, see [dsc resource get result schema][02].

[01]: ../config/get.md
[02]: ../../schemas/outputs/resource/
