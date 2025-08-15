---
description: Command line reference for the 'dsc resource test' command
ms.date:     03/25/2025
ms.topic:    reference
title:       dsc resource test
---

# dsc resource test

## Synopsis

Validates the actual state of a resource instance against a desired state.

## Syntax

### Instance properties from input option

```sh
dsc resource test --input <INPUT> --resource <RESOURCE>
```

### Instance properties from file

```sh
dsc resource test --file <FILE> --resource <RESOURCE>
```

### Instance properties from stdin

```sh
cat <FILE> | dsc resource test [Options] --resource <RESOURCE> --file -
```

## Description

The `test` subcommand validates the actual state of a resource instance against a desired state.

This subcommand tests one instance of a specific DSC Resource. To test multiple resources, use a
group resource, adapter resource, or the [dsc config test][01] command.

The desired state of the instance to test must be passed to this command as a JSON or YAML object
with the `--input` or `--file` option.

If this command is invoked for a command resource that doesn't define its own test operation, DSC
performs a synthetic test. The synthetic test compares each property for the desired state of an
instance against the actual state. The synthetic test uses strict, case-sensitive equivalence. If
the desired state for a property and the actual state aren't the same, DSC marks the property as
out of the desired state.

This command only validates instance properties under two conditions:

1. When the property is explicitly included in the desired state input.
1. When the property has a default value and isn't explicitly included in the desired state input.

## Examples

### Example 1 - Testing a resource with properties from stdin

<a id="example-1"></a>

The command tests whether the `Example` key exists in the current user hive. It specifies the
desired state for the resource instance as JSON and passes it from stdin.

```sh
'{
    "keyPath": "HKCU\\Example",
    "_exist": true
}' | dsc resource test --resource Microsoft.Windows/Registry --file -
```

### Example 2 - Testing a resource with the input option

<a id="example-2"></a>

The command tests whether the `Example` key exists in the current user hive. It specifies the
desired state for the resource instance as JSON and passes it with the `--input` option.

```sh
dsc resource test --resource Microsoft.Windows/Registry --input '{
    "keyPath": "HKCU\\Example",
    "_exist": true
}'
```

### Example 3 - Testing a resource with properties from a YAML file

<a id="example-3"></a>

The command tests whether the `Example` key exists in the current user hive. It specifies the path
to a YAML file defining the desired state for the resource instance with the `--file` option.

```yaml
# example.yaml
keyPath: HKCU\\Example
_exist:  true
```

```sh
dsc resource test --resource Microsoft.Windows/Registry --path ./example.yaml
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

Specifies the desired state of the resource instance to validate against the actual state.

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

Defines the path to a file defining the desired state of the resource instance to validate against
the actual state.

The specified file must contain a JSON or YAML object that represents valid properties for the
resource. DSC validates the object against the resource's instance schema. If the validation fails,
or if the specified file doesn't exist, DSC raises an error.

You can also use this option to pass an instance from stdin, as shown in [Example 1](#example-1).

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

- `json` to emit the data as a [JSON Line][02].
- `pretty-json` to emit the data as JSON with newlines, indentation, and spaces for readability.
- `yaml` to emit the data as YAML.

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

This command returns a formatted data object that includes the desired state of the instance, the
actual state, the list of properties that are out of the desired state, and a boolean value
indicating whether the instance is in the desired state. For more information, see
[dsc resource test result schema][03].

For more information about the formatting of the output data, see the
[--output-format option](#--output-format).

<!-- Link reference definitions -->
[01]: ../config/test.md
[02]: https://jsonlines.org/
[03]: ../../schemas/outputs/resource/test.md
