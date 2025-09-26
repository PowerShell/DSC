---
description: JSON schema reference for the expected stdout from the set resource operation in what-if mode
ms.date:     07/29/2025
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
mode.

DSC expects different output from the command resource depending on the definition of
[whatIf.return][01] in the resource manifest:
  
- If the field is defined as `state`, DSC expects the resource to emit a JSON Line to stdout
  representing the actual state of the resource instance after the **Set** operation changes the
  system.
- If the field is defined as `stateAndDiff`, DSC expects the resource to emit two JSON Lines. The
  first JSON Line should be an object representing the actual state of the resource after the
  **Set** operation. The second JSON Line should be an array representing the names of the resource
  properties that the operation changed on the system.

## state output

When a command resource defines [whatIf.return][01] in its manifest as `state` or `stateAndDiff`,
DSC expects the resource to emit a JSON Line to stdout representing the expected actual state of
the resource instance after the **Set** operation changes the system.

The output must be a JSON object. The object must be a valid representation of an instance of the
resource.

Command resources define their instance schema with the [schema.command][02] or
[schema.embedded][03] fields in their resource manifest. If a command resource returns JSON that is
invalid against the resource instance schema, DSC raises an error.

Adapted resource instances are validated by their adapter when the adapter invokes them.

```yaml
type: object
```

## diff output

When a command resource defines [whatIf.return][01] in its manifest as `stateAndDiff`, DSC expects
the resource to emit a second JSON Line to stdout representing the names of the resource properties
that the operation changed on the system.

This output must be emitted after the JSON Line representing the state of the resource instance
after the operation changes the system.

The output must be a JSON array. The array may be empty, or it may contain one or more strings.
Each string in the array must be the name of one of the resource's properties. Each string in the
array must be unique.

```yaml
Type:              array
ItemsMustBeUnique: true
ItemsType:         string
```

<!-- Reference link definitions -->
[01]: ../manifest/whatif.md#return
[02]: ../manifest/schema/property.md
[03]: ../manifest/schema/embedded.md
