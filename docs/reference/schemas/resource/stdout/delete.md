---
description: JSON schema reference for the expected stdout from the delete resource operation
ms.date:     09/01/2026
ms.topic:    reference
title:       DSC resource delete operation stdout schema reference
---

# DSC resource delete operation stdout schema reference

## Synopsis

Defines the JSON DSC expects a resource to emit to stdout for the **Delete** operation.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/resource/stdout/delete.json
Type:          ['null', object]
```

## Description

DSC expects different output from the command resource depending on whether the user invokes the
**Delete** operation in what-if mode:

- For an actual **Delete** operation, DSC doesn't expect the resource to return any JSON to stdout
  and ignores any data emitted to stdout.
- For a **Delete** operation in what-if mode, when the resource defines a [what-if argument][01]
  for the `delete` method, DSC expects the resource to emit a JSON object to stdout describing the
  expected result of the operation.

## Null output

DSC resources that implement the **Delete** operation shouldn't emit any data to stdout for an
actual **Delete** operation. DSC doesn't expect any output for the operation and ignores any data
emitted to stdout when invoking the operation.

```yaml
Type: 'null'
```

## What-if output

When a resource defines a [what-if argument][01] for the `delete` method, the resource has the
`deleteWhatIf` capability. When a user invokes the **Delete** operation in what-if mode, DSC calls
the `delete` command with the what-if argument and expects the resource to emit a single JSON
object to stdout without modifying the system. DSC returns this object as the result of the
operation. The `deleteWhatIf` capability was added in DSC version 3.3.0.

When a resource doesn't define a what-if argument for the `delete` method, DSC synthesizes the
what-if result from the **Test** operation and doesn't invoke the `delete` command.

The object may be empty. DSC ignores any properties of the object other than `_metadata`.

```yaml
Type: object
```

### _metadata

Defines metadata for the what-if result. When defined, this property must be an object that only
defines the `whatIf` property.

```yaml
Type:     object
Required: false
```

#### whatIf

Describes how the resource would change the system when the user invokes the **Delete** operation
without what-if mode. The value can be any valid JSON value, like a string describing the change or
an object representing the instance that the resource would remove.

```yaml
Type:     any
Required: false
```

For example, a resource might emit the following object in what-if mode:

```json
{
  "_metadata": {
    "whatIf": {
      "message": "Would remove the registry key HKCU\\Example"
    }
  }
}
```

<!-- Reference link definitions -->
[01]: ../manifest/delete.md#what-if-argument
