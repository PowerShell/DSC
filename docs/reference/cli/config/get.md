# dsc config get

## Synopsis

Retrieves the current state of resource instances in a configuration document.

## Syntax

```sh
dsc config get [Options]
```

## Description

The `get` subcommand returns the current state of the resource instances in a configuration
document. When this command runs, DSC validates the configuration document before invoking the get
operation for each resource instance defined in the document.

The configuration document must be passed to this command as JSON or YAML over stdin.

## Examples

### Example 1 - Get the current state of a configuration's resource instances

The command returns the actual state for the resource instances defined in the configuration
document saved as `example.dsc.config.yaml`.

```yaml
# example.dsc.config.yaml
$schema: https://schemas.microsoft.com/dsc/2023/03/configuration.schema.json
resources:
- name: Windows only
  type: DSC/AssertionGroup
  properties:
    $schema: https://schemas.microsoft.com/dsc/2023/03/configuration.schema.json
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
cat ./example.dsc.config.yaml | dsc config get
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
errors, the collection of messages emitted during the operation, and the get operation results for
every instance. For more information, see [dsc config get result schema][01].

[01]: ../../schemas/outputs/config/get.md
