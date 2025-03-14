---
description: Command line reference for the 'dsc config test' command
ms.date:     3/05/2025
ms.topic:    reference
title:       dsc config test
---

# dsc config test

## Synopsis

Verifies whether the resource instances in a configuration document are in the desired state.

## Syntax

### Configuration document from file

```sh
dsc config test [Options] --file <FILE>
```

### Configuration document from option string

```sh
dsc config test [Options] --document <INPUT>
```

### Configuration document from stdin

```sh
cat <FILE> | dsc config test [Options] --file -
```

## Description

The `test` subcommand verifies whether the resource instances in a configuration document are in
the desired state. When this command runs, DSC validates the configuration document before invoking
the test operation for each resource instance defined in the document.

The configuration document must be passed to this command as JSON or YAML with the `--input` or
`--file` option.

## Examples

### Example 1 - Test whether a configuration's resource instances are in the desired state

<a id="example-1"></a>

The command returns the status, desired state, actual state, and differing properties for the
resource instances defined in the configuration document saved as `example.dsc.config.yaml`. It
passes the configuration document to the command from stdin using the `--file` option.

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
cat ./example.dsc.config.yaml | dsc config test --file -
```

### Example 2 - Passing a file to read as the configuration document

<a id="example-2"></a>

The command uses the `--file` option to validate the configuration defined in the
`example.dsc.config.yaml` file.

```sh
dsc config test --file ./example.dsc.config.yaml
```

### Example 3 - Passing a configuration document as a variable

<a id="example-3"></a>

The command uses the `--input` option to validate the configuration stored in the `$desired`
variable.

```sh
dsc config test --input $desired
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
errors, the collection of messages emitted during the operation, and the test operation results for
every instance. For more information, see [dsc config test result schema][02].

For more information about the formatting of the output data, see the
[--output-format option](#--output-format).

<!-- Link reference definitions -->
[01]: https://jsonlines.org/
[02]: ../../schemas/outputs/config/test.md
