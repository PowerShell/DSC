---
description: Command line reference for the 'dsc resource delete' command
ms.date:     03/25/2025
ms.topic:    reference
title:       dsc resource delete
---

# dsc resource delete

## Synopsis

Removes a resource instance from the system.

## Syntax

### Without instance properties

```sh
dsc resource delete [Options] --resource <RESOURCE>
```

### Instance properties from input option

```sh
dsc resource delete --input <INPUT> --resource <RESOURCE>
```

### Instance properties from file

```sh
dsc resource delete --file <FILE> --resource <RESOURCE>
```

### Instance properties from stdin

```sh
cat <FILE> | dsc resource delete [Options] --resource <RESOURCE> --file -
```

## Description

The `delete` subcommand removes a resource instance.

Any properties the resource requires for discerning which instance to delete must be passed to this
command as a JSON or YAML object with the `--input` or `--file` opion.

This command returns no output when successful. If it encounters an error, it surfaces the error to
the caller on stderr and exits with a non-zero exit code.

## Examples

### Example 1 - delete resource instance with input option

<a id="example-1"></a>

If a resource requires one or more property values to identify the instance, the instance
properties can be passed with the `--input` option as either JSON or YAML.

```sh
dsc resource delete --resource Microsoft.Windows/Registry --input '{
    "keyPath": "HKCU\\DSC\\Example"
}'
```

### Example 2 - delete resource instance with input from stdin

<a id="example-2"></a>

If a resource requires one or more property values to identify the instance, the instance
properties can be passed over stdin as either JSON or YAML with the `--file` option.

```sh
'{
    "keyPath": "HKCU\\DSC\\Example"
}' | dsc resource delete --resource Microsoft.Windows/Registry --file -
```

### Example 3 - delete resource instance with input from a YAML file

<a id="example-3"></a>

If a resource requires one or more property values to identify the instance, the instance
properties can be retrieved from a saved JSON or YAML file with the `--file` option.

```yaml
# ./example.delete.yaml
keyPath: HKCU\\DSC\\Example
```

```sh
dsc resource delete --resource Microsoft.Windows/Registry --file ./example.delete.yaml
```

## Options

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

Specifies the resource instance to delete.

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

Defines the path to a file defining the resource instance to delete.

The specified file must contain a JSON or YAML object that represents valid properties for the
resource. DSC validates the object against the resource's instance schema. If the validation fails,
or if the specified file doesn't exist, DSC raises an error.

You can also use this option to pass an instance from stdin, as shown in [Example 2](#example-2).

This option is mutually exclusive with the `--input` option.

```yaml
Type        : string
Mandatory   : false
LongSyntax  : --file <FILE>
ShortSyntax : -f <FILE>
```

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

This command returns no output when successful. When the resource errors, DSC surfaces the error on
stderr and exits with a non-zero exit code.
