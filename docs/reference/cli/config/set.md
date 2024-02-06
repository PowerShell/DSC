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

```sh
dsc config set [Options]
```

## Description

The `set` subcommand enforces the desired state of the resource instances in a configuration
document. When this command runs, DSC validates the configuration document before invoking the test
operation for each resource instance defined in the document. DSC then invokes the set operation
for each resource instance that isn't in the desired state.

The configuration document must be passed to this command as JSON or YAML over stdin.

## Examples

### Example 1 - Set a configuration's resource instances to the desired state

The command inspects the resource instances defined in the configuration document saved as
`example.dsc.config.yaml` and sets them to the desired state as needed.

```yaml
# example.dsc.config.yaml
$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/10/config/document.json
resources:
- name: Windows only
  type: DSC/AssertionGroup
  properties:
    $schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/10/config/document.json
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

The command uses the [--input-file][01] global option to enforce the configuration defined in
the `example.dsc.config.yaml` file.

```sh
dsc --input-file ./example.dsc.config.yaml config set
```

### Example 3 - Passing a configuration document as a variable

The command uses the [--input][02] global option to enforce the configuration stored in the
`$desired` variable.

```sh
dsc --input $desired config set
```

## Options

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
every instance. For more information, see [dsc config get result schema][03].

[01]: ../dsc.md#-p---input-file
[02]: ../dsc.md#-i---input
[03]: ../../schemas/outputs/config/set.md
