---
description: Command line reference for the 'dsc config test' command
ms.date:     06/24/2024
ms.topic:    reference
title:       dsc config test
---

# dsc config test

## Synopsis

Verifies whether the resource instances in a configuration document are in the desired state.

## Syntax

### Configuration document from stdin

```sh
<document-string> | dsc config test [Options]
```

### Configuration document from option string

```sh
dsc config test [Options] --document <document-string>
```

### Configuration document from file

```sh
dsc config test [Options] --path <document-filepath>
```

## Description

The `test` subcommand verifies whether the resource instances in a configuration document are in
the desired state. When this command runs, DSC validates the configuration document before invoking
the test operation for each resource instance defined in the document.

The configuration document must be passed to this command as JSON or YAML over stdin, as a string
with the **document** option, or from a file with the **path** option.

## Examples

### Example 1 - Test whether a configuration's resource instances are in the desired state

The command returns the status, desired state, actual state, and differing properties for the
resource instances defined in the configuration document saved as `example.dsc.config.yaml`.

```yaml
# example.dsc.config.yaml
$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/config/document.json
resources:
- name: Windows only
  type: Microsoft.DSC/Assertion
  properties:
    $schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/config/document.json
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
cat ./example.dsc.config.yaml | dsc config test
```

### Example 2 - Passing a file to read as the configuration document

The command uses the **path** option to validate the configuration defined in the
`example.dsc.config.yaml` file.

```sh
dsc config test --path ./example.dsc.config.yaml
```

### Example 3 - Passing a configuration document as a variable

The command uses the **document** option to validate the configuration stored in the `$desired`
variable.

```sh
dsc config test --document $desired
```

## Options

### -d, --document

Specifies the configuration document to validate state for. The document must be a string
containing a JSON or YAML object. DSC validates the document against the configuration document
schema. If the validation fails, DSC raises an error.

This option can't be used with configuration document over stdin or the `--path` option. Choose
whether to pass the configuration document to the command over stdin, from a file with the `--path`
option, or with the `--document` option.

```yaml
Type:      String
Mandatory: false
```

### -p, --path

Defines the path to a configuration document to validate state for instead of piping the document
from stdin or passing it as a string with the `--document` option. The specified file must contain
a configuration document as a JSON or YAML object. DSC validates the document against the
configuration document schema. If the validation fails, or if the specified file doesn't exist, DSC
raises an error.

This option is mutually exclusive with the `--document` option. When you use this option, DSC
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

This command returns JSON output that includes whether the operation or any resources raised any
errors, the collection of messages emitted during the operation, and the test operation results for
every instance. For more information, see [dsc config test result schema][01].

[01]: ../../schemas/outputs/config/test.md
