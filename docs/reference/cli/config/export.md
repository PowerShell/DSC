---
description: Command line reference for the 'dsc config export' command
ms.date:     03/25/2025
ms.topic:    reference
title:       dsc config export
---

# dsc config export

## Synopsis

Generates a configuration document that defines the existing instances of a set of resources.

## Syntax

### Configuration document from file

```sh
dsc config export [Options] --file <FILE>
```

### Configuration document from option string

```sh
dsc config export [Options] --input <INPUT>
```

### Configuration document from stdin

```sh
cat <FILE> | dsc config export [Options] --file -
```

### Configuration document from file with parameters from stdin

```sh
cat <PARAMETERS_FILE> | dsc config --parameters-file - export [Options] --file <FILE>
```

### Configuration document from option string with parameters from stdin

```sh
cat <PARAMETERS_FILE> | dsc config --parameters-file - export [Options] --input <INPUT>
```

## Description

The `export` subcommand generates a configuration document that includes every instance of a set of
resources.

The configuration document must be passed to this command as JSON or YAML with the `--input` or
`--file` option.

The input document defines the resources to export. DSC ignores any properties specified for
the resources in the input configuration for the operation, but the input document and any
properties for resource instances must still validate against the configuration document and
resource instance schemas.

Only specify resources with a resource manifest that defines the [export][01] section in the input
configuration. Only define each resource type once. If the configuration document includes any
resource instance where the resource type isn't exportable or has already been declared in the
configuration, DSC raises an error.

## Examples

### Example 1 - Test whether a configuration's resource instances are in the desired state

<a id="example-1"></a>

The command inspects the system to return a configuration document containing every discovered
instance of the resources defined in the configuration document saved as `example.dsc.config.yaml`.
It passes the configuration document to the command from stdin using the `--file` option.

```yaml
# example.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Operating system information
  type: Microsoft/OSInfo
  properties: {}
- name: Processes
  type: Microsoft/Process
  properties: {}
```

```sh
cat ./example.dsc.config.yaml | dsc config export --file -
```

### Example 2 - Passing a file to read as the configuration document

<a id="example-2"></a>

The command uses the `--file` option to export resources from the configuration defined in the
`example.dsc.config.yaml` file.

```sh
dsc config export --file ./example.dsc.config.yaml
```

### Example 3 - Passing a configuration document as a variable

<a id="example-3"></a>

The command uses the `--input` option to exoirt resources from the configuration stored in the
`$desired` variable.

```sh
dsc config export --input $desired
```

## Options

### -i, --input

<a id="-i"></a>
<a id="--input"></a>

Specifies the configuration document to validate state for.

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

Defines the path to a configuration document to validate state for.

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

This command returns formatted data that defines a configuration document including every instance
of the resources declared in the input configuration. For more information, see
[DSC Configuration document schema reference][03].

For more information about the formatting of the output data, see the
[--output-format option](#--output-format).

<!-- Link reference definitions -->
[01]: ../../schemas/resource/manifest/export.md
[02]: https://jsonlines.org/
[03]: ../../schemas/config/document.md
