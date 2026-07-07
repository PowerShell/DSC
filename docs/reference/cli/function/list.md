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

For more information about the available builtin functions, see
[DSC Configuration document functions reference][01]

## Examples

### Example 1 - List all functions

<a id="example-1"></a>

This command returns information about all available DSC functions.

```sh
dsc function list
```

```output
Category               Function              MinArgs  MaxArgs  ReturnTypes  Description            
---------------------------------------------------------------------------------------------------
Array                  array                 1        1        a-----       Convert the value to a…
Array                  createArray           0        maxInt   a-----       Creates an array from …
Array                  range                 2        2        a-----       Creates an array of in…
Array                  tryIndexFromEnd       2        2        ab-nso       Retrieves a value from 
Array, Lambda          filter                2        2        a-----       Filters an array with …
Array, Lambda          map                   2        2        a-----       Transforms an array by 
Array, Object          intersection          2        maxInt   a----o       Returns a single array 
Array, Object          tryGet                2        2        ab-nso       Attempts to retrieve a 
// truncated
```

### Example 2 - Filter functions by name

<a id="example-2"></a>

This command filters functions by name using a wildcard pattern.

```sh
dsc function list resource*
```

```output
Category Name       MinArgs MaxArgs ArgTypes Description
-------- ----       ------- ------- -------- -----------
Resource resourceId 2       2       ---s-    Constructs a resource ID from the given type and name
```

### Example 3 - Get details for a specific function

<a id="example-3"></a>

This command returns detailed information about a specific function, displaying it in YAML format.

```sh
dsc function list concat --output-format yaml
```

```yaml
category:
- Array
- String
name: concat
description: Concatenates two or more strings or arrays
minArgs: 2
maxArgs: 18446744073709551615
acceptedArgOrderedTypes:
- - String
  - Array
- - String
  - Array
remainingArgAcceptedTypes:
- String
- Array
returnTypes:
- String
- Array
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

The `--output-format` option controls which format DSC uses for the data the command returns. The
available formats are:

- `json` to emit the data as a [JSON Line][02].
- `pretty-json` to emit the data as JSON with newlines, indentation, and spaces for readability.
- `yaml` to emit the data as YAML.
- `table-no-truncate` to emit the data as a summary table without truncating each line to the
  current console width.

> [!NOTE]
> In the current release of DSC, the `table-no-truncate` option has a bug that causes the data to
> emit as a series of YAML documents instead. This bug will be fixed in a future version of DSC.

The default output format depends on whether DSC detects that the output is being redirected or
captured as a variable:

- If the command isn't being redirected or captured, DSC displays the output as a summary table
  described in the [Output](#output) section of this document.
- If the command output is redirected or captured, DSC emits the data as the `json` format to
  stdout.

When you use this option, DSC uses the specified format regardless of whether the command is being
redirected or captured.

When the command isn't redirected or captured, the output in the console is formatted for improved
readability. When the command isn't redirected or captured, the output includes terminal sequences
for formatting.

```yaml
Type:         string
Mandatory:    false
ValidValues:  [json, pretty-json, yaml, table-no-truncate]
LongSyntax:   --output-format <<OUTPUT_FORMAT>>
ShortSyntax:  -o <<OUTPUT_FORMAT>>
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

This command returns a formatted array containing an object for each function that includes the
function's type, version, manifest settings, and other metadata. For more information, see
[dsc function list result schema reference][03].

If the output of the command isn't captured or redirected, it displays in the console by default as
a summary table for the returned functions. The summary table includes the following columns,
displayed in the listed order:

- **Category** - The category the function belongs to.
- **Function** - The name of the function.
- **MinArgs** - The minimum number of arguments the function accepts.
- **MaxArgs** - The maximum number of arguments the function accepts.
- **ReturnTypes** - The [data types][04] the function emits as flags. The valid return types are
  displayed in the following order, using a `-` instead of the appropriate letter if the function
  doesn't return that data type:

  - `a` indicates that the function returns an array value.
  - `b` indicates that the function returns a boolean value.
  - `l` indicates that the function returns a lambda value.
  - `n` indicates that the function returns a number value.
  - `s` indicates that the function returns a string value.
  - `o` indicates that the function returns an object value.

- **Description** - A synopsis of what the function does.

For more information about the formatting of the output data, see the
[--output-format option](#--output-format).

<!-- Link reference definitions -->
[01]: ../../schemas/config/functions/overview.md
[02]: https://jsonlines.org/
[03]: ../../schemas/outputs/function/list.md
[04]: ../../schemas/definitions/functions/builtin/dataTypes.md
