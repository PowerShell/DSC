---
description: JSON schema reference for the expected stdout from the test resource operation
ms.date:     07/29/2025
ms.topic:    reference
title:       DSC resource test operation stdout schema reference
---

# DSC resource test operation stdout schema reference

## Synopsis

Defines the JSON DSC expects a resource to emit to stdout for the **Test** operation.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/resource/stdout/test.json
```

## Description

Defines the JSON DSC expects a resource to emit to stdout for the **Test** operation.

DSC expects different output from the command resource depending on the definition of
[test.return][01] in the resource manifest:

- If the field is omitted or defined as `state` (the default value), DSC expects the resource to
  emit a JSON Line to stdout representing the actual state of the resource instance with the
  [_inDesiredState][02] canonical resource property included in the returned object.
- If the field is defined as `stateAndDiff`, DSC expects the resource to emit two JSON Lines. The
  first JSON Line should be an object representing the actual state of the resource instance with
  the `_inDesiredState` included in the returned object. The second JSON Line should be an array
  representing the names of the resource properties that aren't in the desired state.

## state output

For the **Test** operation, DSC always expects the resource to emit a JSON Line to stdout
representing the actual state of the resource instance with the [_inDesiredState][02] canonical
resource property included in the returned object.

The output must be a JSON object. The object must be a valid representation of an instance of the
resource.

Command resources define their instance schema with the [schema.command][03] or
[schema.embedded][04] fields in their resource manifest. If a command resource returns JSON that is
invalid against the resource instance schema, DSC raises an error.

Adapted resource instances are validated by their adapter when the adapter invokes them.

## diff output

When a command resource defines [test.return][01] in its manifest as `stateAndDiff`, DSC expects
the resource to emit a second JSON Line to stdout representing the names of the resource properties
that aren't in the desired state.

This output must be emitted after the JSON Line representing the actual state of the resource
instance with the [_inDesiredState][02] canonical resource property included in the returned
object.

The output must be a JSON array. The array may be empty, or it may contain one or more strings.
Each string in the array must be the name of one of the resource's properties. Each string in the
array must be unique. The array should never include the `_inDesiredState` property.

<!-- Reference link definitions -->
[01]: ../manifest/test.md#return
[02]: ../properties/inDesiredState.md
[03]: ../manifest/schema/property.md
[04]: ../manifest/schema/embedded.md
