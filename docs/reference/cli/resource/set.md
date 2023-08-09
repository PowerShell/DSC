---
description: Command line reference for the 'dsc resource set' command
ms.date:     08/04/2023
ms.topic:    reference
title:       dsc resource set
---

# dsc resource set

## Synopsis

Invokes the set operation of a resource.

## Syntax

```sh
dsc resource set [Options] --resource <RESOURCE>
```

## Description

The `set` subcommand enforces the desired state of a resource instance and returns the final state.

This subcommand sets one instance of a specific DSC Resource. To set multiple resources,
use a resource group or the [dsc config set][01] command.

The desired state of the instance to set must be passed to this command as JSON. The JSON can be
passed to this command from stdin or with the `--input` option.

This subcommand can only be invoked for command-based DSC Resources that define the `set` section
of their resource manifest. If this subcommand is called for a resource that doesn't define a set
operation, DSC raises an error.

The subcommand's behavior depends on the value of the `set.preTest` option in the resource
manifest:

- If the resource's manifest doesn't define the `set.preTest` key as `true`, DSC invokes the
  resource's test operation to determine whether a set operation is required.

  If the instance is already in the desired state, DSC doesn't invoke the set operation. If the
  instance isn't in the desired state, DSC invokes the resource's set operation with the desired
  state as input.
- If the resource's manifest defines the `set.preTest` key as `true`, DSC invokes the resource's
  set operation without testing the resource state first.

## Examples

### Example 1 - Setting a resource with properties from stdin

The command ensures that the `Example` key exists in the current user hive. It specifies the
resource instance properties as JSON and passes them from stdin.

```sh
'{
    "keyPath": "HKCU\\Example",
    "_ensure": "present"
}' | dsc resource set --resource Microsoft.Windows/Registry
```

### Example 2 - Setting a resource with the input option

The command ensures that the `Example` key exists in the current user hive. It specifies the
resource instance properties as JSON and passes them with the **input** option.

```sh
dsc resource set --resource Microsoft.Windows/Registry --input '{
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

This command returns JSON output that includes the actual state of the instance before and after
the set operation, and the list of properties that the set operation modified. For more
information, see [dsc resource set result schema][02].

[01]: ../config/set.md
[02]: ../../schemas/outputs/resource/set.md
