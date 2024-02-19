---
description: Command line reference for the 'dsc config test' command
ms.date:     01/17/2024
ms.topic:    reference
title:       dsc config test
---

# dsc config test

## Synopsis

Verifies whether the resource instances in a configuration document are in the desired state.

## Syntax

```sh
dsc config test [Options]
```

## Description

The `test` subcommand verifies whether the resource instances in a configuration document are in
the desired state. When this command runs, DSC validates the configuration document before invoking
the test operation for each resource instance defined in the document.

The configuration document must be passed to this command as JSON or YAML over stdin.

## Examples

### Example 1 - Test whether a configuration's resource instances are in the desired state

The command returns the status, desired state, actual state, and differing properties for the
resource instances defined in the configuration document saved as `example.dsc.config.yaml`.

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
cat ./example.dsc.config.yaml | dsc config test
```

### Example 2 - Passing a file to read as the configuration document

The command uses the [--input-file][01] global option to validate the configuration defined in
the `example.dsc.config.yaml` file.

```sh
dsc --input-file ./example.dsc.config.yaml config test
```

### Example 3 - Passing a configuration document as a variable

The command uses the [--input][02] global option to validate the configuration stored in the
`$desired` variable.

```sh
dsc --input $desired config test
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
errors, the collection of messages emitted during the operation, and the test operation results for
every instance. For more information, see [dsc config test result schema][03].

[01]: ../dsc.md#-p---input-file
[02]: ../dsc.md#-i---input
[03]: ../../schemas/outputs/config/test.md
