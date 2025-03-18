---
description: Command line reference for the 'dsc config set' command
ms.date:     03/25/2025
ms.topic:    reference
title:       dsc config set
---

# dsc config set

## Synopsis

Enforces the desired state of resource instances in a configuration document.

## Syntax

### Configuration document from file

```sh
dsc config set [Options] --file <FILE>
```

### Configuration document from option string

```sh
dsc config set [Options] --input <INPUT>
```

### Configuration document from stdin

```sh
cat <FILE> | dsc config set [Options] --file -
```

## Description

The `set` subcommand enforces the desired state of the resource instances in a configuration
document. When this command runs, DSC validates the configuration document before invoking the test
operation for each resource instance defined in the document. DSC then invokes the set operation
for each resource instance that isn't in the desired state.

The configuration document must be passed to this command as JSON or YAML with the `--input` or
`--file` option.

## Examples

### Example 1 - Set a configuration's resource instances to the desired state

<a id="example-1"></a>

The command inspects the resource instances defined in the configuration document saved as
`example.dsc.config.yaml` and sets them to the desired state as needed.

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
cat ./example.dsc.config.yaml | dsc config set --file -
```

### Example 2 - Passing a file to read as the configuration document

<a id="example-2"></a>

The command uses the **path** option to enforce the configuration defined in the
`example.dsc.config.yaml` file.

```sh
dsc config set --path ./example.dsc.config.yaml
```

### Example 3 - Passing a configuration document as a variable

<a id="example-3"></a>

The command uses the **document** option to enforce the configuration stored in the `$desired`
variable.

```sh
dsc config set --document $desired
```

## Options

### -i, --input

<a id="-i"></a>
<a id="--input"></a>

Specifies the configuration document to enforce state for.

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

Defines the path to a configuration document to enforce state for.

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

### -w, --what-if

<a id="-w"></a>
<a id="--what-if"></a>

When you specify this flag option, DSC doesn't actually change the system state with the `set`
operation. Instead, it returns output indicating _how_ the operation will change system state when
called without this option. Use this option to preview the changes DSC will make to a system.

The output for the command when you use this option is the same as without, except that the
`ExecutionType` metadata field is set to `WhatIf` instead of `Actual`.

```yaml
Type        : boolean
Mandatory   : false
LongSyntax  : --what-if
ShortSyntax : -w
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
errors, the collection of messages emitted during the operation, and the set operation results for
every instance. For more information, see [dsc config get result schema][02].

For more information about the formatting of the output data, see the
[--output-format option](#--output-format).

<!-- Link reference definitions -->
[01]: https://jsonlines.org/
[02]: ../../schemas/outputs/config/set.md
