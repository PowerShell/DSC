---
description: Command line reference for the 'dsc resource set' command
ms.date:     09/27/2023
ms.topic:    reference
title:       dsc resource set
---

# dsc resource set

## Synopsis

Invokes the set operation of a resource.

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

The `set` subcommand enforces the desired state of a resource instance and returns the final state.

This subcommand sets one instance of a specific DSC Resource. To set multiple resources,
use a resource group or the [dsc config set][01] command.

The desired state of the instance to set must be passed to this command as a JSON or YAML object.
The object properties must be valid properties for the resource. The instance properties can be
passed to this command from stdin, as a string with the `--input` option, or from a saved file with
the `--path` option.

This subcommand can only be invoked for command-based DSC Resources that define the `set` section
of their resource manifest. If this subcommand is called for a resource that doesn't define a set
operation, DSC raises an error.

> [!IMPORTANT]
> The `dsc resource set` command always invokes the `set` operation for the resource. Resources
> may, but aren't required to, implement logic that pretests an instance for the `set` operation.
>
> This is different from how [dsc config set][02] works, where DSC always tests an instance, either
> synthetically or by invoking the `test` operation for the resource, and only invokes `set` for an
> instance if it's not in the desired state.
>
> Command-based resources indicate whether they implement pretest for the `set` operation by
> defining the [set.implementsPretest][03] property in their resource manifest. If that property is
> defined as `true`, it indicates that the resource implements pretest. If `set.implementsPretest`
> is set to `false` or is undefined, the manifest indicates that the resource doesn't implement
> pretest.
>
> If a resource indicates that it implements pretest, users should expect that the resource only
> modifies an instance during a `set` operation if the pretest shows that the instance isn't in the
> desired state.
>
> If a resource doesn't implement pretest, users should expect that the resource always modifies an
> instance during a `set` operation.
>
> For resources that don't implement pretest for the `set` operation, Microsoft recommends always
> calling `dsc resource test` against an instance to see whether it's in the desired state _before_
> invoking `dsc resource set`. This can help avoid accidental errors caused by resources that don't
> implement a fully idempotent `set` command.

## Examples

### Example 1 - Setting a resource with properties from stdin

The command ensures that the `Example` key exists in the current user hive. It specifies the
resource instance properties as JSON and passes them from stdin.

```sh
'{
    "keyPath": "HKCU\\Example",
    "_exist": true
}' | dsc resource set --resource Microsoft.Windows/Registry
```

### Example 2 - Setting a resource with the input option

The command ensures that the `Example` key exists in the current user hive. It specifies the
resource instance properties as JSON and passes them with the **input** option.

```sh
dsc resource set --resource Microsoft.Windows/Registry --input '{
    "keyPath": "HKCU\\Example",
    "_exist": true
}'
```

### Example 3 - Setting a resource with properties from a YAML file

The command ensures that the `Example` key exists in the current user hive. It specifies the
path to a yaml file defining the resource instance properties with the **path** option.

```sh
cat ./example.yaml
```

```yaml
keyPath: HKCU\\Example
_exist:  true
```

```sh
dsc resource set --resource Microsoft.Windows/Registry --path ./example.yaml
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

This command returns JSON output that includes the actual state of the instance before and after
the set operation, and the list of properties that the set operation modified. For more
information, see [dsc resource set result schema][04].

[01]: ../config/set.md
[02]: ../config/set.md
[03]: ../../schemas/resource/manifest/set.md#implementspretest
[04]: ../../schemas/outputs/resource/set.md
