---
description: Command line reference for the 'dsc config set' command
ms.date:     01/17/2024
ms.topic:    reference
title:       dsc config set
---

# dsc config set

## Synopsis

Enforces the desired state of resource instances in a configuration document.

## Syntax

### Configuration document from stdin

```sh
<document-string> | dsc config set [Options]
```

### Configuration document from option string

```sh
dsc config set [Options] --document <document-string>
```

### Configuration document from file

```sh
dsc config set [Options] --path <document-filepath>
```

## Description

The `set` subcommand enforces the desired state of the resource instances in a configuration
document. When this command runs, DSC validates the configuration document before invoking the test
operation for each resource instance defined in the document. DSC then invokes the set operation
for each resource instance that isn't in the desired state.

The configuration document must be passed to this command as JSON or YAML over stdin, as a string
with the **document** option, or from a file with the **path** option.

## Examples

### Example 1 - Set a configuration's resource instances to the desired state

The command inspects the resource instances defined in the configuration document saved as
`example.dsc.config.yaml` and sets them to the desired state as needed.

```yaml
# example.dsc.config.yaml
$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/config/document.json
resources:
- name: Windows only
  type: DSC/AssertionGroup
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
    _ensure: Present
  dependsOn:
    - '[DSC/Assertion]Windows only'
```

```sh
cat ./example.dsc.config.yaml | dsc config set
```

### Example 2 - Passing a file to read as the configuration document

The command uses the **path** option to enforce the configuration defined in the
`example.dsc.config.yaml` file.

```sh
dsc config set --path ./example.dsc.config.yaml
```

### Example 3 - Passing a configuration document as a variable

The command uses the **document** option to enforce the configuration stored in the `$desired`
variable.

```sh
dsc config set --document $desired
```

## Options

### -d, --document

Specifies the configuration document to enforce state for. The document must be a string
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

Defines the path to a configuration document to enforce state for instead of piping the document
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

### -w, --what-if

When you specify this flag option, DSC doesn't actually change the system state with the `set`
operation. Instead, it returns output indicating _how_ the operation will change system state when
called without this option. Use this option to preview the changes DSC will make to a system.

The output for the command when you use this option is the same as without, except that the
`ExecutionType` metadata field is set to `WhatIf` instead of `Actual`.

```yaml
Type:      Boolean
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
errors, the collection of messages emitted during the operation, and the set operation results for
every instance. For more information, see [dsc config get result schema][01].

[01]: ../../schemas/outputs/config/set.md
