---
description: Command line reference for the 'dsc config get' command
ms.date:     3/05/2025
ms.topic:    reference
title:       dsc config get
---

# dsc config get

## Synopsis

Retrieves the current state of resource instances in a configuration document.

## Syntax

### Configuration document from file

```sh
dsc config get [Options] --file <FILE>
```

### Configuration document from option string

```sh
dsc config get [Options] --input <INPUT>
```

### Configuration document from stdin

```sh
cat <FILE> | dsc config get [Options] --file -
```

## Description

The `get` subcommand returns the actual state of the resource instances in a configuration
document. When this command runs, DSC validates the configuration document before invoking the get
operation for each resource instance defined in the document.

The configuration document must be passed to this command as JSON or YAML with the `--input` or
`--file` option.

## Examples

### Example 1 - Get the current state of a configuration's resource instances

<a id="example-1"></a>

The command returns the actual state for the resource instances defined in the configuration
document saved as `example.dsc.config.yaml`. It passes the configuration document to the command
from stdin using the `--file` option.

```yaml
# example.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Windows only
  type: Microsoft.DSC/Assertion
  properties:
    $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
    resources:
    - name: os
      type: Microsoft/OSInfo
      properties:
        family: Windows
- name: Current user registry example
  type: Microsoft.Windows/Registry
  properties:
    keyPath: HKCU\example
    _exist: true
  dependsOn:
    - "[resourceId('Microsoft.DSC/Assertion', 'Windows only')"
```

```sh
cat ./example.dsc.config.yaml | dsc config get --file -
```

### Example 2 - Passing a file to read as the configuration document

<a id="example-2"></a>

The command uses the `--file` option to retrieve the resource instances defined in the
`example.dsc.config.yaml` file.

```sh
dsc config get --path ./example.dsc.config.yaml
```

### Example 3 - Passing a configuration document as a variable

<a id="example-3"></a>

The command uses the `--input` option to retrieve the resource instances defined in a
configuration document stored in the `$desired` variable.

```sh
dsc config get --input $desired
```

## Options

### -i, --input

<a id="-i"></a>
<a id="--input"></a>

Specifies the configuration document to retrieve actual state for.

The document must be a string containing a JSON or YAML object. DSC validates the document against
the configuration document schema. If the validation fails, DSC raises an error.

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

Defines the path to a configuration document to retrieve actual state for.

The specified file must contain a configuration document as a JSON or YAML object. DSC validates
the document against the configuration document schema. If the validation fails, or if the
specified file doesn't exist, DSC raises an error.

You can also use this option to pass a configuration document from stdin, as shown in
[Example 1](#example-1).

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

- `json` to emit the data as a [JSON Line][01].
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

This command returns formatted data that includes whether the operation or any resources raised any
errors, the collection of messages emitted during the operation, and the get operation results for
every instance. For more information, see [dsc config get result schema][02].

For more information about the formatting of the output data, see the
[--output-format option](#--output-format).

<!-- Link reference definitions -->
[01]: https://jsonlines.org/
[02]: ../../schemas/outputs/config/get.md
