---
description: JSON schema reference for the data returned by the 'dsc function list' command.
ms.date:     07/03/2025
ms.topic:    reference
title:       dsc function list result schema reference
---

# dsc function list result schema reference

## Synopsis

The result output from the `dsc function list` command.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.0/outputs/function/list.json
Type:          object
```

## Description

The output from the `dsc function list` command includes a representation of discovered DSC
functions as a series of [JSON Lines][01]. This schema describes the JSON object returned for each
function.

## Required properties

Each function in the output always includes these properties:

- [category](#category)
- [name](#name)
- [description](#description)
- [minArgs](#minargs)
- [maxArgs](#maxargs)
- [acceptedArgOrderedTypes](#acceptedargorderedtypes)
- [remainingArgAcceptedTypes](#remainingargacceptedtypes)
- [returnTypes](#returntypes)

## Properties

### category

Identifies the categories that the function belongs to. Every function belongs to one or more
categories. The defined categories are:

- `Array` - functions for constructing and operating on arrays.
- `Cidr` - functions for working with CIDR notation.
- `Comparison` - functions for comparing values and return a boolean value.
- `Date` - functions for working with dates.
- `Deployment` - functions for working with runtime data, like retrieving secrets.
- `Lambda` - functions for processing data with subexpressions.
- `Logical` - functions for defining conditional logic and working with boolean values.
- `Numeric` - functions for constructing and operating on numbers.
- `Object` - functions for constructing and operating on objects.
- `Resource` - functions for operating on resource instances.
- `String` - functions for constructing and operating on strings.
- `System` - functions that retrieve information from the operating system.

> [!NOTE]
> This list is _not_ guaranteed to be stable. In future minor releases, DSC may add new function
> categories. No categories will be _removed_ except in a major version release with breaking
> changes.

```yaml
Type:              array
Required:          true
ItemsType:         string
ItemsValidValues: [
                    Array,
                    Cidr,
                    Comparison,
                    Date,
                    Deployment,
                    Lambda,
                    Logical,
                    Numeric,
                    Object,
                    Resource,
                    String,
                    System
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

### minArgs

Indicates the minimum number of arguments for the function. When you provide fewer than the minimum
required arguments DSC raises a parsing error.

```yaml
Type:     integer
Required: true
MinimumValue: 0
```

### maxArgs

Indicates the maximum number of arguments for the function. When you provide more than the maximum
allowed arguments DSC raises a parsing error.

```yaml
Type:     integer
Required: true
MinimumValue: 0
```

### acceptedArgOrderedTypes

Indicates the acceptable types for each argument in the order that the function expects them. This
property is an array of arrays. Each inner array contains a set of strings that map to allowed
[argument types](../../definitions/functions/builtin/dataTypes.md).

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
parameter. When this field is defined as an array of
[argument types](../../definitions/functions/builtin/dataTypes.md) the function supports passing multiple
arguments of those types for the final parameter.

DSC raises a parsing error when the value for a remaining argument isn't a valid type.

```yaml
Type:     ['null', 'array']
Required: true
ItemsMustBeUnique: true
ItemsReference: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.0/definitions/functions/builtin/argTypes.json
```

### returnTypes

Indicates the [types](../../definitions/functions/builtin/dataTypes.md) of values the function can
return. When the only item in this array is the `null` type the function doesn't return any data.
When this field contains more than one item the function may return any of the listed types. For
more information about how the function returns data, see the reference documentation for that
function.

```yaml
Type:     array
Required: true
ItemsMustBeUnique: true
ItemsReference: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.0/definitions/functions/builtin/argTypes.json
```

<!-- Link reference definitions -->
[01]: https://jsonlines.org/
