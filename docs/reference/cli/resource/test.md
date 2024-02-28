---
description: Command line reference for the 'dsc resource test' command
ms.date:     08/04/2023
ms.topic:    reference
title:       dsc resource test
---

# dsc resource test

## Synopsis

Invokes the test operation of a resource.

## Syntax

### Instance properties from stdin

```sh
<instance-properties> | dsc resource set [Options] --resource <RESOURCE>
```

### Instance properties from input option

```sh
dsc resource set --input '<instance-properties>' --resource <RESOURCE>
```

### Instance properties from file

```sh
dsc resource set --path <instance-properties-filepath> --resource <RESOURCE>
```

## Description

The `test` subcommand validates the actual state of a resource instance against a desired state.

This subcommand tests one instance of a specific DSC Resource. To test multiple resources, use a
resource group or the [dsc config test][01] command.

The desired state of the instance to test must be passed to this command as a JSON or YAML object.
The object properties must be valid properties for the resource. The instance properties can be
passed to this command from stdin, as a string with the `--input` option, or from a saved file with
the `--path` option.

If this command is invoked for a command-based DSC Resource that doesn't define its own test
operation, DSC performs a synthetic test. The synthetic test compares each property for the desired
state of an instance against the actual state. The synthetic test uses strict, case-sensitive
equivalence. If the desired state for a property and the actual state aren't the same, DSC marks
the property as out of the desired state.

This command only validates instance properties under two conditions:

1. When the property is explicitly included in the desired state input.
1. When the property has a default value and isn't explicitly included in the desired state input.

## Examples

### Example 1 - Testing a resource with properties from stdin

The command tests whether the `Example` key exists in the current user hive. It specifies the
resource instance properties as JSON and passes them from stdin.

```sh
'{
    "keyPath": "HKCU\\Example",
    "_exist": true
}' | dsc resource test --resource Microsoft.Windows/Registry
```

### Example 2 - Testing a resource with the input option

The command tests whether the `Example` key exists in the current user hive. It specifies the
resource instance properties as JSON and passes them with the **input** option.

```sh
dsc resource test --resource Microsoft.Windows/Registry --input '{
    "keyPath": "HKCU\\Example",
    "_exist": true
}'
```

### Example 3 - Testing a resource with properties from a YAML file

The command tests whether the `Example` key exists in the current user hive. It specifies the
path to a YAML file defining the resource instance properties with the **path** option.

```sh
```

```yaml
keyPath: HKCU\\Example
_exist:  true
```

```sh
dsc resource test --resource Microsoft.Windows/Registry --path ./example.yaml
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

Specifies a JSON or YAML object with the properties defining the desired state of a DSC Resource
instance. DSC validates the object against the resource's instance schema. If the validation fails,
DSC raises an error.

This option can't be used with instance properties over stdin or the `--path` option. Choose
whether to pass the instance properties to the command over stdin, from a file with the `--path`
option, or with the `--input` option.

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

This command returns JSON output that includes the desired state of the instance, the actual state,
the list of properties that are out of the desired state, and a boolean value indicating whether
the instance is in the desired state. For more information, see
[dsc resource test result schema][02].

[01]: ../config/test.md
[02]: ../../schemas/outputs/resource/test.md
