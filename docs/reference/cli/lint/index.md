---
description: Command line reference for the 'dsc lint' command
ms.date:     10/20/2026
ms.topic:    reference
title:       dsc lint
---

# dsc lint

## Synopsis

Checks a configuration document for problems without applying it.

## Syntax

### Configuration document from file

```sh
dsc lint [Options] --file <FILE>
```

### Configuration document from option string

```sh
dsc lint [Options] --input <INPUT>
```

### Configuration document from stdin

```sh
cat <FILE> | dsc lint [Options] --file -
```

### Several configuration documents

```sh
dsc lint [Options] --file <FILE> --file <FILE>
```

### List the rules

```sh
dsc lint --list-rules [--output-format <OUTPUT_FORMAT>]
```

## Description

The `lint` command analyzes a configuration document and reports problems as diagnostics. It never
invokes a resource operation, so it's safe to run on any machine, including CI agents.

The command works in stages:

1. It parses the document as JSON or YAML. A document that doesn't parse is reported with the
   `parse-error` rule.
1. It validates the document against the configuration document JSON schema. Every violation is
   reported with the `schema-invalid` rule and the JSON pointer to the problem.
1. It runs the static rules, which only need the document. These rules find duplicate resources,
   expressions that don't parse or call unknown functions, undeclared parameters and variables,
   unresolved and circular dependencies, invalid parameter constraints, deprecated features, and
   more.
1. Unless you specify [--no-discovery](#--no-discovery), it discovers the resource types the
   document uses and runs the resource rules, which report types that aren't installed, deprecated
   types, and literal properties that don't match the resource's schema.

Each diagnostic has a level: `error`, `warning`, or `info`. By default the command exits with code
`5` when it reports at least one error. Use [--fail-on](#--fail-on) to fail on warnings or
informational diagnostics instead. Use [--rule](#--rule) or the `lint` section of the DSC settings
file to change the level of a rule or turn it off. A few rules have a fixed level that can't be
changed: `parse-error`, `schema-invalid`, `unsupported-document-type`, and `discovery-failed`.

For the list of rules and what each one checks, see [dsc lint rules][01].

## Examples

### Example 1 - Lint a configuration document

<a id="example-1"></a>

The command reports a duplicate resource name and exits with code `5`. In the console, each
diagnostic is rendered as a report that shows the problem in the document.

```yaml
# example.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: EnsureTest1Value
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: one
- name: EnsureTest1Value
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: two
```

```sh
dsc lint --file ./example.dsc.config.yaml
```

```text
duplicate-resource

  × Resource 'EnsureTest1Value' of type 'Microsoft.DSC.Debug/Echo' is declared more than once
   ╭─[./example.dsc.config.yaml:7:3]
 5 │   properties:
 6 │     output: one
 7 │ - name: EnsureTest1Value
   ·   ──────────────────────
 8 │   type: Microsoft.DSC.Debug/Echo
 9 │   properties:
   ╰────
  help: Resource names must be unique among resources of the same type at the same level.

errors: 1, warnings: 0, info: 0
```

When the output is redirected, or when you specify [--output-format](#--output-format), the
command emits the result as data instead:

```sh
dsc lint --file ./example.dsc.config.yaml --output-format yaml
```

```yaml
diagnostics:
- level: error
  rule: duplicate-resource
  category: document
  message: Resource 'EnsureTest1Value' of type 'Microsoft.DSC.Debug/Echo' is declared more than once
  path: /resources/1
  file: ./example.dsc.config.yaml
  location:
    line: 7
    column: 3
summary:
  errors: 1
  warnings: 0
  info: 0
valid: false
discovery: true
```

### Example 2 - Fail on warnings in a pipeline

<a id="example-2"></a>

The command exits with code `5` when any warning or error is reported, so a pipeline step fails on
an unused parameter as well as on an error. Discovery is skipped because the build agent doesn't
have the resources installed.

```sh
dsc lint --file ./example.dsc.config.yaml --no-discovery --fail-on warning
```

### Example 3 - Change the level of a rule

<a id="example-3"></a>

The command turns off the `no-unused-parameters` rule and raises `missing-require-version`, which
is off by default, to a warning.

```sh
dsc lint --file ./example.dsc.config.yaml --rule no-unused-parameters=off --rule missing-require-version=warning
```

### Example 4 - Check a document that targets another system root

<a id="example-4"></a>

When you pass `--system-root`, the `system-root-unused` rule warns if no expression in the document
calls the `systemRoot()` function, because the document wouldn't target the specified root.

```sh
dsc lint --file ./example.dsc.config.yaml --system-root D:\
```

### Example 5 - List the rules

<a id="example-5"></a>

The command lists every rule with its category, the level it reports at, whether it needs resource
discovery, and a description.

```sh
dsc lint --list-rules
```

## Options

### -i, --input

<a id="-i"></a>
<a id="--input"></a>

Specifies the configuration document to lint.

The document must be a string containing a JSON or YAML object.

This option is mutually exclusive with the `--file` option.

```yaml
Type        : string
Mandatory   : false
LongSyntax  : --input <INPUT>
ShortSyntax : -i <INPUT>
```

### -f, --file

<a id="-f"></a>
<a id="--file"></a>

Defines the path to a configuration document to lint. Specify the option more than once to lint
several documents in one run; the diagnostics then include the `file` property.

You can also use this option to pass a configuration document from stdin with `-` as the path.

This option is mutually exclusive with the `--input` option.

```yaml
Type        : string
Mandatory   : false
LongSyntax  : --file <FILE>
ShortSyntax : -f <FILE>
```

### -p, --parameters

<a id="-p"></a>
<a id="--parameters"></a>

Specifies the parameter values that would be passed to the configuration, as a JSON or YAML
object. When you specify parameters, the command checks that every supplied value matches a
declared parameter, that every parameter without a default value has a value, and that each value
satisfies the parameter's type and constraints.

```yaml
Type        : string
Mandatory   : false
LongSyntax  : --parameters <PARAMETERS>
ShortSyntax : -p <PARAMETERS>
```

### --parameters-file

<a id="--parameters-file"></a>

Defines the path to a JSON or YAML file with the parameter values. Use `-` to read the parameters
from stdin. Values from `--parameters` take precedence over values from the file.

```yaml
Type        : string
Mandatory   : false
LongSyntax  : --parameters-file <PARAMETERS_FILE>
```

### -r, --system-root

<a id="-r"></a>
<a id="--system-root"></a>

Specifies the operating system root path the configuration would target. The command doesn't
check that the path exists; it only enables the `system-root-unused` rule.

```yaml
Type        : string
Mandatory   : false
LongSyntax  : --system-root <SYSTEM_ROOT>
ShortSyntax : -r <SYSTEM_ROOT>
```

### --no-discovery

<a id="--no-discovery"></a>

Skips resource discovery. The rules that inspect installed resources don't run, and the output
reports `discovery: false`. Use this option on machines that don't have the resources installed,
or to lint faster.

```yaml
Type        : boolean
Mandatory   : false
LongSyntax  : --no-discovery
```

### --fail-on

<a id="--fail-on"></a>

Defines the lowest diagnostic level that makes the command exit with code `5`. The default is
`error`.

```yaml
Type         : string
Mandatory    : false
DefaultValue : error
ValidValues  : [error, warning, info]
LongSyntax   : --fail-on <LEVEL>
```

### --rule

<a id="--rule"></a>

Overrides the level of a rule for this run, as `<rule>=<level>`. The level is one of `off`,
`info`, `warning`, or `error`. Specify the option more than once to override several rules. The
command exits with code `1` for an unknown rule.

Overrides from this option take precedence over the `lint` section of the DSC settings file. The
policy settings file takes precedence over this option.

```yaml
Type        : string
Mandatory   : false
LongSyntax  : --rule <RULE=LEVEL>
```

### --list-rules

<a id="--list-rules"></a>

Lists the rules with the level each reports at in this run instead of linting a document. When
the output isn't redirected, the command shows a table; otherwise it emits the list in the format
specified by `--output-format`. For the schema of the list, see [dsc lint rules schema][03].

This option is mutually exclusive with the `--input` and `--file` options.

```yaml
Type        : boolean
Mandatory   : false
LongSyntax  : --list-rules
```

### -o, --output-format

<a id="-o"></a>
<a id="--output-format"></a>

The `--output-format` option controls which format DSC uses for the data the command returns. The
available formats are:

- `json` to emit the data as a [JSON Line][02].
- `pretty-json` to emit the data as JSON with newlines, indentation, and spaces for readability.
- `yaml` to emit the data as YAML.

The default output depends on whether DSC detects that the output is being redirected or captured
as a variable:

- If the command isn't being redirected or captured, DSC renders each diagnostic in the console as
  a report that shows the problem in the document text, with the rule identifier as the code and
  what the rule requires as the help, followed by the counts of diagnostics by level.
- If the command output is redirected or captured, DSC emits the data as the `json` format to
  stdout.

Specify a format to get the data in the console too.

```yaml
Type        : string
Mandatory   : false
ValidValues : [json, pretty-json, yaml]
LongSyntax  : --output-format <OUTPUT_FORMAT>
ShortSyntax : -o <OUTPUT_FORMAT>
```

### -h, --help

<a id="-h"></a>
<a id="--help"></a>

Displays the help for the current command or subcommand. When you specify this option, the
application ignores all other options and arguments.

```yaml
Type        : boolean
Mandatory   : false
LongSyntax  : --help
ShortSyntax : -h
```

## Output

This command returns the diagnostics for the document, counts of the diagnostics by level, whether
the document has no errors, and whether resource discovery ran. The command always writes the
result to stdout, including when the document doesn't parse. In the console it renders the
diagnostics as reports; otherwise, or when you specify [--output-format](#--output-format), it
emits the result as data. For more information, see [dsc lint result schema][04].

## Exit codes

| Exit Code |                                     Meaning                                      |
| :-------: | :------------------------------------------------------------------------------- |
|    `0`    | No diagnostic at or above the `--fail-on` level was reported.                    |
|    `1`    | The command received invalid arguments, such as an unknown rule.                 |
|    `2`    | The command failed for another reason, such as an unreadable settings file.      |
|    `4`    | A document or the parameters couldn't be read.                                   |
|    `5`    | At least one diagnostic at or above the `--fail-on` level was reported.          |

## Settings

The `lint` section of the DSC settings file sets the level of rules for every run. The same
section in the policy settings file takes precedence over the `--rule` option.

```json
{
  "lint": {
    "rules": {
      "no-unused-parameters": { "level": "off" },
      "missing-require-version": { "level": "warning" }
    }
  }
}
```

<!-- Link reference definitions -->
[01]: rules.md
[02]: https://jsonlines.org/
[03]: ../../schemas/outputs/lint/rules.md
[04]: ../../schemas/outputs/lint/result.md
