---
description: JSON schema reference for the data types that DSC functions operate on.
ms.date:     07/03/2025
ms.topic:    reference
title:       Function data types schema reference
---

# Function data types schema reference

## Synopsis

Defines the available data types that DSC functions operate on.

## Metadata

```yaml
SchemaDialect : https://json-schema.org/draft/2020-12/schema
SchemaID      : https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.0/definitions/functions/builtin/dataTypes.json
Type          : string
ValidValues   : [
                  'array'
                  'boolean'
                  'lambda'
                  'null'
                  'number'
                  'object'
                  'string'
                ]
```

## Description

Functions in DSC only support a subset of possible data types for input arguments and output
values. The supported data types are:

- `array` - A collection of items
- `boolean` - Either `true` or `false`
- `lambda` - A valid lambda expression
- `null` - The JSON value `null`
- `number` - A 64-bit integer
- `object` - A collection of key-value pairs
- `string` - UTF-8 text
