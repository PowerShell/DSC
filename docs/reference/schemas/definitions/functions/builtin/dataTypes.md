---
description: JSON schema reference for the data types that DSC functions operate on.
ms.date:     09/01/2026
ms.topic:    reference
title:       Function data types schema reference
---

# Function data types schema reference

## Synopsis

Defines the available data types that DSC functions operate on.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/definitions/functions/builtin/argKind.json
Type:          string
ValidValues:   [
                 array,
                 boolean,
                 lambda,
                 null,
                 number,
                 object,
                 string
               ]
```

## Description

Functions in DSC only support a subset of possible data types for input arguments and output
values. The `dsc function list` command reports these data types in the `acceptedArgOrderedTypes`,
`remainingArgAcceptedTypes`, and `returnTypes` properties for each function. For more information,
see [dsc function list result schema reference][01].

The supported data types are:

- `array` - A collection of items
- `boolean` - Either `true` or `false`
- `lambda` - A valid lambda expression
- `null` - The JSON value `null`
- `number` - A 64-bit integer
- `object` - A collection of key-value pairs
- `string` - UTF-8 text

<!-- Link reference definitions -->
[01]: ../../../outputs/function/list.md
