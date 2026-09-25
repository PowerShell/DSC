---
description: Command line reference for the 'dsc function list' command
ms.date:     09/01/2026
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
Category       Function         Syntax                               Description
-----------------------------------------------------------------------------------------
Array          array            array( <string | number | object …  Convert the value to…
Array          createArray      createArray( [value], ... )          Creates an array fro…
Array          range            range( <startIndex>, <count> )       Creates an array of …
Array          tryIndexFromEnd  tryIndexFromEnd( <array>, <index> )  Retrieves a value fr…
Array, Lambda  filter           filter( <array>, <lambda> )          Filters an array wit…
Array, Lambda  map              map( <array>, <lambda> )             Transforms an array …
Array, Object  intersection     intersection( <array | object>, <…   Returns a single arr…
Array, Object  tryGet           tryGet( <array | object>, <key | …   Attempts to retrieve…
// truncated
```

### Example 2 - Filter functions by name

<a id="example-2"></a>

This command filters functions by name using a wildcard pattern.

```sh
dsc function list resource*
```

```output
Category  Function    Syntax                        Description
----------------------------------------------------------------------------------------
Resource  resourceId  resourceId( <type>, <name> )  Constructs a resource ID from the gi…
```

### Example 3 - Get details for a specific function

<a id="example-3"></a>

This command returns detailed information about a specific function, displaying it in YAML format.

```sh
dsc function list concat --output-format yaml
```

```yaml
category:
- array
- string
name: concat
description: Concatenates two or more strings or arrays
syntax: concat( <string | array>, <string | array>, ... )
constraints: All arguments must be of the same type (all strings or all arrays)
minArgs: 2
maxArgs: 18446744073709551615
acceptedArgOrderedTypes:
- - string
  - array
- - string
  - array
remainingArgAcceptedTypes:
- string
- array
returnTypes:
- string
- array
```

### Example 4 - Filter functions by category

<a id="example-4"></a>

This command uses the `--category` option to list only the functions in the `lambda` category.

```sh
dsc function list --category lambda
```

```output
Category       Function         Syntax                                     Description
----------------------------------------------------------------------------------------
Array, Lambda  filter           filter( <array>, <lambda> )                Filters an ar…
Array, Lambda  map              map( <array>, <lambda> )                   Transforms an…
Lambda         lambda           lambda( <param1>, [param2], ..., <body> )  Creates a lam…
Lambda         lambdaVariables  lambdaVariables( <name> )                  Retrieves the…
```

### Example 5 - Filter functions by description

<a id="example-5"></a>

This command uses the `--description` option to list only the functions whose description
matches a wildcard pattern.

```sh
dsc function list --description *CIDR*
```

```output
Category  Function    Syntax                                          Description
-----------------------------------------------------------------------------------------
CIDR      cidrHost    cidrHost( <cidr>, <hostIndex> )                 Calculates the usab…
CIDR      cidrSubnet  cidrSubnet( <cidr>, <newCidr>, <subnetIndex> )  Splits the specifie…
CIDR      parseCidr   parseCidr( <cidr> )                             Parses an IP addres…
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

### -c, --category

<a id="-c"></a>
<a id="--category"></a>

The `--category` option filters the results by function category. You can specify the option more
than once to filter for multiple categories. When you specify more than one category, DSC returns
only the functions that belong to every specified category.

```yaml
Type:         string
Mandatory:    false
ValidValues:  [array, cidr, comparison, date, deployment, lambda, logical,
               numeric, object, resource, string, system]
LongSyntax:   --category <CATEGORY>
ShortSyntax:  -c <CATEGORY>
```

### -d, --description

<a id="-d"></a>
<a id="--description"></a>

The `--description` option filters the results by function description. You can use wildcard
patterns in the value. DSC returns only the functions whose description matches the pattern.

```yaml
Type:         string
Mandatory:    false
LongSyntax:   --description <PATTERN>
ShortSyntax:  -d <PATTERN>
```

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

This command returns an object for each function that includes the function's name, categories,
syntax, argument metadata, and return types. For more information, see
[dsc function list result schema reference][03]. For more information about the [data types][04]
used in the argument and return type metadata, see the linked schema reference.

If the output of the command isn't captured or redirected, it displays in the console by default as
a summary table for the returned functions. The summary table includes the following columns,
displayed in the listed order:

- **Category** - The categories the function belongs to.
- **Function** - The name of the function.
- **Syntax** - The syntax for calling the function, showing its expected arguments.
- **Description** - A synopsis of what the function does.

For more information about the formatting of the output data, see the
[--output-format option](#--output-format).

<!-- Link reference definitions -->
[01]: ../../schemas/config/functions/overview.md
[02]: https://jsonlines.org/
[03]: ../../schemas/outputs/function/list.md
[04]: ../../schemas/definitions/functions/builtin/dataTypes.md
