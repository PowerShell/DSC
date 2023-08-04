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
cat ./example.dsc.config.yaml | dsc config test
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
every instance. For more information, see [dsc config test result schema][01].

[01]: ../../schemas/outputs/config/test.md
