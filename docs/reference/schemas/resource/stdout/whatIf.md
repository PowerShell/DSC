---
description: >-
  JSON schema reference for the expected stdout from the set resource operation in what-if mode
ms.date:     09/01/2026
ms.topic:    reference
title:       DSC resource what-if operation stdout schema reference
---

# DSC resource what-if operation stdout schema reference

## Synopsis

Defines the JSON DSC expects a resource to emit to stdout for the **Set** operation in what-if
mode.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/resource/stdout/whatIf.json
```

## Description

Defines the JSON DSC expects a resource to emit to stdout for the **Set** operation in what-if
mode. DSC invokes a resource in what-if mode when a user specifies the `--what-if` option for the
`dsc config set` or `dsc resource set` commands:

- When the `set` definition in the resource manifest includes a [what-if argument][01], DSC calls
  the `set` command with that argument.
- Otherwise, when the resource manifest defines the deprecated [whatIf][02] method, DSC calls that
  command.
- Otherwise, DSC synthesizes the what-if result from the **Test** operation and doesn't invoke the
  resource in what-if mode.

DSC expects different output from the command resource depending on the return kind for the
invoked method. The return kind is the value of the [whatIfReturns][03] field for the method when
it's defined, or the value of the [return][04] field otherwise:

- If neither field is defined, DSC doesn't expect the resource to return any JSON to stdout.
  Instead, DSC invokes the **Get** operation on the resource after the command concludes and
  synthesizes the result from the state of the resource.
- If the return kind is `state`, DSC expects the resource to emit a JSON Line to stdout
  representing the expected state of the resource instance after the **Set** operation would
  change the system.
- If the return kind is `stateAndDiff`, DSC expects the resource to emit two JSON Lines. The first
  JSON Line should be an object representing the expected state of the resource after the **Set**
  operation. The second JSON Line should be an array representing the names of the resource
  properties that the operation would change on the system.

## Null output

When the return kind for the invoked method isn't defined, DSC doesn't expect the resource to emit
any JSON to stdout in what-if mode.

```yaml
Type: 'null'
```

## state output

When the return kind for the invoked method is `state` or `stateAndDiff`, DSC expects the resource
to emit a JSON Line to stdout representing the expected actual state of the resource instance after
the **Set** operation would change the system.

The output must be a JSON object. The object must be a valid representation of an instance of the
resource.

Command resources define their instance schema with the [schema.command][05] or
[schema.embedded][06] fields in their resource manifest. If a command resource returns JSON that is
invalid against the resource instance schema, DSC raises an error.

Adapted resource instances are validated by their adapter when the adapter invokes them.

```yaml
Type: object
```

## diff output

When the return kind for the invoked method is `stateAndDiff`, DSC expects the resource to emit a
second JSON Line to stdout representing the names of the resource properties that the operation
would change on the system.

This output must be emitted after the JSON Line representing the expected state of the resource
instance after the operation would change the system.

The output must be a JSON array. The array may be empty, or it may contain one or more strings.
Each string in the array must be the name of one of the resource's properties. Each string in the
array must be unique.

```yaml
Type:              array
ItemsMustBeUnique: true
ItemsType:         string
```

<!-- Reference link definitions -->
[01]: ../manifest/set.md#what-if-argument
[02]: ../manifest/whatif.md
[03]: ../manifest/set.md#whatifreturns
[04]: ../manifest/set.md#return
[05]: ../manifest/schema/property.md
[06]: ../manifest/schema/embedded.md
