---
description: Command line reference for the 'dsc resource delete' command
ms.date:     05/08/2024
ms.topic:    reference
title:       dsc resource delete
---

# dsc resource delete

## Synopsis

Invokes the delete operation of a resource.

## Syntax

### Without instance properties

```sh
dsc resource delete [Options] --resource <RESOURCE>
```

### Instance properties from stdin

```sh
<instance-properties> | dsc resource delete [Options] --resource <RESOURCE>
```

### Instance properties from input option

```sh
dsc resource delete --input '<instance-properties>' --resource <RESOURCE>
```

### Instance properties from file

```sh
dsc resource delete --path <instance-properties-filepath> --resource <RESOURCE>
```

## Description

The `delete` subcommand removes a resource instance.

Any properties the resource requires for discerning which instance to delete must be passed to this
command as a JSON or YAML object. The object can be passed to this command from stdin or with the
`--input` option. You can also use the `--path` option to read the object from a JSON or YAML file.

This command returns no output when successful. If it encounters an error, it surfaces the error to
the caller on stderr and exits with a non-zero exit code.

## Examples

### Example 1 - delete resource instance with input option

If a resource requires one or more property values to return the actual state of the instance, the
instance properties can be passed with the **input** option as either JSON or YAML.

```sh
dsc resource delete --resource Microsoft.Windows/Registry --input '{
    "keyPath": "HKCU\\DSC\\Example"
}'
```

### Example 2 - delete resource instance with input from stdin

If a resource requires one or more property values to return the actual state of the instance, the
instance properties can be passed over stdin as either JSON or YAML.

```sh
'{
    "keyPath": "HKCU\\DSC\\Example"
}' | dsc resource delete --resource Microsoft.Windows/Registry
```

### Example 3 - delete resource instance with input from a YAML file

If a resource requires one or more property values to return the actual state of the instance, the
instance properties can be retrieved from a saved JSON or YAML file.

```sh
cat ./example.delete.yaml
```

```yaml
keyPath: HKCU\\DSC\\Example
```

```sh
dsc resource delete --resource Microsoft.Windows/Registry --path ./example.delete.yaml
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

### -h, --help

Displays the help for the current command or subcommand. When you specify this option, the
application ignores all options and arguments after this one.

```yaml
Type:      Boolean
Mandatory: false
```

## Output

This command returns no output when successful. When the resource errors, DSC surfaces the error on
stderr and exits with a non-zero exit code.
