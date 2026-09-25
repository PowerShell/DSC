---
description: JSON schema reference for the data returned by the 'dsc function list' command.
ms.date:     09/01/2026
ms.topic:    reference
title:       dsc function list result schema reference
---

# dsc function list result schema reference

## Synopsis

The result output from the `dsc function list` command.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/outputs/function/list.json
Type:          object
```

## Description

The output from the `dsc function list` command includes a representation of discovered DSC
functions as a series of [JSON Lines][01]. This schema describes the JSON object returned for each
function.

## Examples

The following example shows the output of `dsc function list concat -o pretty-json`:

```json
{
  "category": [
    "array",
    "string"
  ],
  "name": "concat",
  "description": "Concatenates two or more strings or arrays",
  "syntax": "concat( <string | array>, <string | array>, ... )",
  "constraints": "All arguments must be of the same type (all strings or all arrays)",
  "minArgs": 2,
  "maxArgs": 18446744073709551615,
  "acceptedArgOrderedTypes": [
    [
      "string",
      "array"
    ],
    [
      "string",
      "array"
    ]
  ],
  "remainingArgAcceptedTypes": [
    "string",
    "array"
  ],
  "returnTypes": [
    "string",
    "array"
  ]
}
```

## Required properties

Each function in the output always includes these properties:

- [category](#category)
- [name](#name)
- [description](#description)
- [syntax](#syntax)
- [constraints](#constraints)
- [minArgs](#minargs)
- [maxArgs](#maxargs)
- [acceptedArgOrderedTypes](#acceptedargorderedtypes)
- [remainingArgAcceptedTypes](#remainingargacceptedtypes)
- [returnTypes](#returntypes)

## Properties

### category

Identifies the categories that the function belongs to. Every function belongs to one or more
categories. The defined categories are:

- `array` - functions for constructing and operating on arrays.
- `cidr` - functions for working with CIDR notation.
- `comparison` - functions for comparing values and return a boolean value.
- `date` - functions for working with dates.
- `deployment` - functions for working with runtime data, like retrieving secrets.
- `lambda` - functions for processing data with subexpressions.
- `logical` - functions for defining conditional logic and working with boolean values.
- `numeric` - functions for constructing and operating on numbers.
- `object` - functions for constructing and operating on objects.
- `resource` - functions for operating on resource instances.
- `string` - functions for constructing and operating on strings.
- `system` - functions that retrieve information from the operating system.

> [!NOTE]
> This list is _not_ guaranteed to be stable. In future minor releases, DSC may add new function
> categories. No categories will be _removed_ except in a major version release with breaking
> changes.

```yaml
Type:              array
Required:          true
ItemsType:         string
ItemsValidValues: [
                    array,
                    cidr,
                    comparison,
                    date,
                    deployment,
                    lambda,
                    logical,
                    numeric,
                    object,
                    resource,
                    string,
                    system
                  ]
```

### name

Defines the name of the function as you would specify it in a configuration document or manifest
field that supports functions. Function names always use `camelCase`, like `tryWhich`. Function
names are always defined as ASCII alphabetical characters.

```yaml
Type:     string
Required: true
Pattern:  ^[a-z][a-zA-Z]+$
```

### description

Defines a synopsis for the function's purpose as a short string.

```yaml
Type:     string
Required: true
```

### syntax

Defines the syntax for calling the function as a short string, like
`concat( <string | array>, <string | array>, ... )`. The syntax string shows the arguments the
function accepts and the types it accepts for each argument. This property was added in DSC
version 3.3.0.

```yaml
Type:     string
Required: true
```

### constraints

Defines any additional constraints on the arguments for the function that the argument types alone
don't express, like `All arguments must be of the same type (all strings or all arrays)`. When the
function doesn't have any additional constraints, this property is `null`. This property was added
in DSC version 3.3.0.

```yaml
Type:     [string, 'null']
Required: true
```

### minArgs

Indicates the minimum number of arguments for the function. When you provide fewer than the minimum
required arguments DSC raises a parsing error.

```yaml
Type:         integer
Required:     true
MinimumValue: 0
```

### maxArgs

Indicates the maximum number of arguments for the function. When you provide more than the maximum
allowed arguments DSC raises a parsing error. For functions that accept an unlimited number of
arguments, this property is the largest value DSC can represent for an unsigned integer, like
`18446744073709551615` on 64-bit platforms.

```yaml
Type:         integer
Required:     true
MinimumValue: 0
```

### acceptedArgOrderedTypes

Indicates the acceptable types for each argument in the order that the function expects them. This
property is an array of arrays. Each inner array contains a set of strings that map to allowed
[argument types][02].

DSC raises a parsing error when the value for an argument isn't a valid type for that argument.

```yaml
Type:              array
Required:          true
ItemsMustBeUnique: false
ItemsType:         array(Valid function argument types)
```

### remainingArgAcceptedTypes

Indicates the acceptable types for the last parameter of a variadic function. Variadic functions
are functions that accept multiple values for the final parameter.

When this field is defined as `null`, the function doesn't support multiple values for the last
parameter. When this field is defined as an array of [argument types][02] the function supports
passing multiple arguments of those types for the final parameter.

DSC raises a parsing error when the value for a remaining argument isn't a valid type.

```yaml
Type:              ['null', 'array']
Required:          true
ItemsMustBeUnique: true
ItemsReference:    https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/definitions/functions/builtin/argKind.json
```

### returnTypes

Indicates the [types][02] of values the function can return. When the only item in this array is
the `null` type the function doesn't return any data. When this field contains more than one item
the function may return any of the listed types. For more information about how the function
returns data, see the reference documentation for that function.

```yaml
Type:              array
Required:          true
ItemsMustBeUnique: true
ItemsReference:    https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/definitions/functions/builtin/argKind.json
```

<!-- Link reference definitions -->
[01]: https://jsonlines.org/
[02]: ../../definitions/functions/builtin/dataTypes.md
