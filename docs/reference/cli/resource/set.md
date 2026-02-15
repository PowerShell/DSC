---
description: Command line reference for the 'dsc resource set' command
ms.date:     03/25/2025
ms.topic:    reference
title:       dsc resource set
---

# dsc resource set

## Synopsis

Enforces the desired state for a resource instance.

## Syntax

### Instance properties from input option

```sh
dsc resource set --input <INPUT> --resource <RESOURCE> [--what-if]
```

### Instance properties from file

```sh
dsc resource set --file <FILE> --resource <RESOURCE> [--what-if]
```

### Instance properties from stdin

```sh
cat <FILE> | dsc resource set [Options] --resource <RESOURCE> --file - [--what-if]
```

### What-if mode

```sh
dsc resource set --input <INPUT> --resource <RESOURCE> --what-if
```

## Description

The `set` subcommand enforces the desired state of a resource instance and returns the final state.

This subcommand sets one instance of a specific DSC Resource. To set multiple resources,
use a resource group or the [dsc config set][01] command.

The desired state of the instance to set must be passed to this command as a JSON or YAML object
with the `--input` or `--file` option.

This subcommand can only be invoked for command resources that define the `set` section of their
resource manifest. If this subcommand is called for a resource that doesn't define a set operation,
DSC raises an error.

> [!IMPORTANT]
> The `dsc resource set` command always invokes the `set` operation for the resource. Resources
> might, but aren't required to, implement logic that pretests an instance for the `set` operation.
>
> This is different from how [dsc config set][02] works, where DSC always tests an instance, either
> synthetically or by invoking the `test` operation for the resource, and only invokes `set` for an
> instance if it's not in the desired state.
>
> Command-based resources indicate whether they implement pretest for the `set` operation by
> defining the [set.implementsPretest][03] property in their resource manifest. If that property is
> defined as `true`, it indicates that the resource implements pretest. If `set.implementsPretest`
> is set to `false` or is undefined, the manifest indicates that the resource doesn't implement
> pretest.
>
> If a resource indicates that it implements pretest, users should expect that the resource only
> modifies an instance during a `set` operation if the pretest shows that the instance isn't in the
> desired state.
>
> If a resource doesn't implement pretest, users should expect that the resource always modifies an
> instance during a `set` operation.
>
> For resources that don't implement pretest for the `set` operation, Microsoft recommends always
> calling `dsc resource test` against an instance to see whether it's in the desired state _before_
> invoking `dsc resource set`. This can help avoid accidental errors caused by resources that don't
> implement a fully idempotent `set` command.

## Examples

### Example 1 - Setting a resource with properties from stdin

<a id="example-1"></a>

The command ensures that the `Example` key exists in the current user hive. It specifies the
desired state for the resource instance as JSON and passes it from stdin.

```sh
'{
    "keyPath": "HKCU\\Example",
    "_exist": true
}' | dsc resource set --resource Microsoft.Windows/Registry --file -
```

### Example 2 - Setting a resource with the input option

<a id="example-2"></a>

The command ensures that the `Example` key exists in the current user hive. It specifies the
desired state for the resource instance as JSON and passes it with the `--input` option.

```sh
dsc resource set --resource Microsoft.Windows/Registry --input '{
    "keyPath": "HKCU\\Example",
    "_exist": true
}'
```

### Example 3 - Setting a resource with properties from a YAML file

<a id="example-3"></a>

The command ensures that the `Example` key exists in the current user hive. It specifies the path
to a YAML file defining the desired state for the resource instance with the `--file` option.

```yaml
# ./example.yaml
keyPath: HKCU\\Example
_exist:  true
```

```sh
dsc resource set --resource Microsoft.Windows/Registry --path ./example.yaml
```

## Options

### -r, --resource

<a id="-r"></a>
<a id="--resource"></a>

Specifies the fully qualified type name of the DSC Resource to use, like
`Microsoft.Windows/Registry`.

The fully qualified type name syntax is: `<owner>[.<group>][.<area>]/<name>`, where:

- The `owner` is the maintaining author or organization for the resource.
- The `group` and `area` are optional name components that enable namespacing for a resource.
- The `name` identifies the component the resource manages.

```yaml
Type        : string
Mandatory   : true
LongSyntax  : --resource <RESOURCE>
ShortSyntax : -r <RESOURCE>
```

### -i, --input

Specifies the desired state of the resource instance to enforce on the system.

The instance must be a string containing a JSON or YAML object. DSC validates the object against
the resource's instance schema. If the validation fails, DSC raises an error.

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

Defines the path to a file defining the desired state of the resource instance to enforce on the
system.

The specified file must contain a JSON or YAML object that represents valid properties for the
resource. DSC validates the object against the resource's instance schema. If the validation fails,
or if the specified file doesn't exist, DSC raises an error.

You can also use this option to pass an instance from stdin, as shown in [Example 1](#example-1).

This option is mutually exclusive with the `--input` option.

```yaml
Type        : string
Mandatory   : false
LongSyntax  : --file <FILE>
ShortSyntax : -f <FILE>
```

### -o, --output-format

<a id="-o"></a>
<a id="--output-format"></a>

The `--output-format` option controls which format DSC uses for the data the command returns. The
available formats are:

- `json` to emit the data as a [JSON Line][04].
- `pretty-json` to emit the data as JSON with newlines, indentation, and spaces for readability.
- `yaml` to emit the data as YAML.

The default output format depends on whether DSC detects that the output is being redirected or
captured as a variable:

- If the command isn't being redirected or captured, DSC displays the output as the `yaml` format
  in the console.
- If the command output is redirected or captured, DSC emits the data as the `json` format to
  stdout.

When you use this option, DSC uses the specified format regardless of whether the command is being
redirected or captured.

When the command isn't redirected or captured, the output in the console is formatted for improved
readability. When the command isn't redirected or captured, the output include terminal sequences
for formatting.

```yaml
Type        : string
Mandatory   : false
ValidValues : [json, pretty-json, yaml]
LongSyntax  : --output-format <OUTPUT_FORMAT>
ShortSyntax : -o <OUTPUT_FORMAT>
```

### -w, --what-if

<a id="-w"></a>
<a id="--what-if"></a>

When you specify this flag option, DSC doesn't actually change the system state with the `set`
operation. Instead, it returns output indicating _how_ the operation will change system state when
called without this option. Use this option to preview the changes DSC will make to a system.

This option is useful for interactive and exploratory operations, allowing you to see what changes
would be made before actually applying them.

This option also supports the `--dry-run` and `--noop` aliases for compatibility with other tools.

```yaml
Type        : boolean
Mandatory   : false
LongSyntax  : --what-if
ShortSyntax : -w
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

This command returns a formatted data object that includes the actual state of the instance before and after
the set operation, and the list of properties that the set operation modified. For more
information, see [dsc resource set result schema][05].

For more information about the formatting of the output data, see the
[--output-format option](#--output-format).

<!-- Link reference definitions -->
[01]: ../config/set.md
[02]: ../config/set.md
[03]: ../../schemas/resource/manifest/set.md#implementspretest
[04]: https://jsonlines.org/
[05]: ../../schemas/outputs/resource/set.md
