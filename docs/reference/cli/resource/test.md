---
description: Command line reference for the 'dsc resource test' command
ms.date:     08/04/2023
ms.topic:    reference
title:       dsc resource test
---

# dsc resource test

## Synopsis

Invokes the test operation of a resource.

## Syntax

```sh
dsc resource test [Options] --resource <RESOURCE>
```

## Description

The `test` subcommand validates the actual state of a resource instance against a desired state.

This subcommand tests one instance of a specific DSC Resource. To test multiple resources, use a
resource group or the [dsc config test][01] command.

The desired state of the instance to test must be passed to this command as JSON. The JSON can be
passed to this command from stdin or with the `--input` option.

If this command is invoked for a command-based DSC Resource that doesn't define its own test
operation, DSC performs a synthetic test. The synthetic test compares each property for the desired
state of an instance against the actual state. The synthetic test uses strict, case-sensitive
equivalence. If the desired state for a property and the actual state aren't the same, DSC marks
the property as out of the desired state.

This command only validates instance properties under two conditions:

1. When the property is explicitly included in the desired state input.
1. When the property has a default value and isn't explicitly included in the desired state input.

## Examples

### Example 1 - Testing a resource with properties from stdin

The command tests whether the `Example` key exists in the current user hive. It specifies the
resource instance properties as JSON and passes them from stdin.

```sh
'{
    "keyPath": "HKCU\\Example",
    "_ensure": "present"
}' | dsc resource test --resource Microsoft.Windows/Registry
```

### Example 2 - Testing a resource with the input option

The command tests whether the `Example` key exists in the current user hive. It specifies the
resource instance properties as JSON and passes them with the **input** option.

```sh
dsc resource test --resource Microsoft.Windows/Registry --input '{
    "keyPath": "HKCU\\Example",
    "_ensure": "present"
}'
```

## Options

### -r, --resource

Specifies the fully qualified type name of the DSC Resource to use, like
`Microsoft.Windows/Registry`.

The fully qualified type name syntax is: `<owner>[.<group>][.<area>]/<name>`, where:

- The `owner` is the maintaining author or organization for the resource.
- The `group` and `area` are optional name components that enable namespacing for a resource.
- The `name` identifies the component the resource manages.

```yaml
Type:      String
Mandatory: true
```

### -i, --input

Specifies a JSON object with the properties defining the desired state of a DSC Resource instance.
DSC validates the JSON against the resource's instance schema. If the validation fails, DSC raises
an error.

This option can't be used with JSON over stdin. Choose whether to pass the instance JSON to the
command over stdin or with the `--input` flag.

```yaml
Type:      String
Mandatory: false
```

### -h, --help

Displays the help for the current command or subcommand. When you specify this option, the
application ignores all options and arguments after this one.

```yaml
Type:      Boolean
Mandatory: false
```

## Output

This command returns JSON output that includes the desired state of the instance, the actual state,
the list of properties that are out of the desired state, and a boolean value indicating whether
the instance is in the desired state. For more information, see
[dsc resource test result schema][02].

[01]: ../config/test.md
[02]: ../../schemas/outputs/resource/test.md
