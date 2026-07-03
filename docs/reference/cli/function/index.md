---
description: Command line reference for the 'dsc function list' command
ms.date:     07/20/2025
ms.topic:    reference
title:       dsc function list
---

# dsc function list

## Synopsis

List or find DSC functions.

## Syntax

```sh
dsc function list [Options] [<FUNCTION_NAME>]
```

## Description

The `dsc function list` command returns information about the available DSC functions. By default,
it returns all available functions. You can filter the results by specifying a function name or
pattern.

DSC functions are built-in functions that can be used in configuration documents to perform various
operations including:

- String manipulation and formatting
- Mathematical calculations  
- Logical operations
- Type conversions
- Parameter and variable access
- Resource references

## Examples

### Example 1 - List all functions

This command returns information about all available DSC functions.

```sh
dsc function list
```

```output
Category    Function     MinArgs  MaxArgs  ArgTypes  Description                                                                          
------------------------------------------------------------------------------------------------------------------------------------------
Array       createArray  0        maxInt   a-nso     Creates an array from the given elements
Comparison  equals       2        2        a-nso     Evaluates if the two values are the same
Deployment  parameters   1        1        ---s-     Retrieves parameters from the configuration
Deployment  variables    1        1        ---s-     Retrieves the value of a variable
Logical     and          2        maxInt   -b---     Evaluates if all arguments are true
Logical     bool         1        1        --ns-     Converts a string or number to a boolean
Logical     false        0        0        -----     Returns the boolean value false
Logical     if           3        3        abnso     Evaluates a condition and returns second value if true, otherwise returns third value
Logical     not          1        1        -b---     Negates a boolean value
Logical     or           2        maxInt   -b---     Evaluates if any arguments are true
Logical     true         0        0        -----     Returns the boolean value true
Numeric     add          2        2        --n--     Adds two or more numbers together
Numeric     div          2        2        --n--     Divides the first number by the second
Numeric     int          1        1        --ns-     Converts a string or number to an integer
Numeric     max          1        maxInt   a-n--     Returns the largest number from a list of numbers
Numeric     min          1        maxInt   a-n--     Returns the smallest number from a list of numbers
Numeric     mod          2        2        --n--     Divides the first number by the second and returns the remainder
Numeric     mul          2        2        --n--     Multiplies two or more numbers together
Numeric     sub          2        2        --n--     Subtracts the second number from the first
# truncated
```

### Example 2 - List functions with JSON output

This command returns function information in pretty JSON format.

```sh
dsc function list --output-format pretty-json
```

```jsonc
{
  "category": "Array",
  "name": "createArray",
  "description": "Creates an array from the given elements",
  "minArgs": 0,
  "maxArgs": 18446744073709551615,
  "acceptedArgTypes": [
    "String",
    "Number",
    "Object",
    "Array"
  ]
}
{
  "category": "Comparison",
  "name": "equals",
  "description": "Evaluates if the two values are the same",
  "minArgs": 2,
  "maxArgs": 2,
  "acceptedArgTypes": [
    "Number",
    "String",
    "Array",
    "Object"
  ]
}
// truncated
```

### Example 3 - Filter functions by name

This command filters functions by name using a wildcard pattern.

```sh
dsc function list "resource*"
```

```output
Category Name       MinArgs MaxArgs ArgTypes Description
-------- ----       ------- ------- -------- -----------
Resource resourceId 2       2       ---s-    Constructs a resource ID from the given type and name
```

### Example 4 - Get details for a specific function

This command returns detailed information about a specific function.

```sh
dsc function list "concat" --output-format json
```

```json
{"category":"String","name":"concat","description":"Concatenates two or more strings or arrays","minArgs":2,"maxArgs":18446744073709551615,"acceptedArgTypes":["String","Array"]}
```

## Parameters

### FUNCTION_NAME

The name of the function to retrieve information about. You can use wildcard patterns to filter
functions. When you specify this parameter, DSC only returns information about functions that match
the pattern.

```yaml
Type:      string
Required:  false
Position:  0
```

## Options

### -o, --output-format

<a id="-o"></a>
<a id="--output-format"></a>

Defines the output format for the command. By default, when run in an interactive terminal, the
command outputs a human-readable table. When run non-interactively or when the output is
redirected, the command outputs JSON.

```yaml
Type:         string
Mandatory:    false
DefaultValue: table (when interactive), json (when non-interactive)
ValidValues:  [json, pretty-json, yaml, table-no-truncate]
LongSyntax:   --output-format <FORMAT>
ShortSyntax:  -o <FORMAT>
```

### -h, --help

<a id="-h"></a>
<a id="--help"></a>

Displays the help for the current command. When you specify this option, the application ignores
all other options and arguments.

```yaml
Type        : boolean
Mandatory   : false
LongSyntax  : --help
ShortSyntax : -h
```

## Output

The command returns information about DSC functions. The output includes:

- **name** - The name of the function
- **category** - The category the function belongs to (Logical, Numeric, String, Array, System, Resource)
- **description** - A description of what the function does
- **minArgs** - The minimum number of arguments the function accepts
- **maxArgs** - The maximum number of arguments the function accepts
- **acceptedArgTypes** - The types of arguments the function accepts

### Argument Types

The `acceptedArgTypes` field uses the following abbreviations in table format:

- **a** - Array
- **b** - Boolean  
- **n** - Number
- **o** - Object
- **s** - String
- **-** - No specific type (any type accepted)

When multiple types are accepted, multiple letters are shown. For example, `sn` means the function
accepts both String and Number arguments.
