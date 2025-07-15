---
description: Command line reference for the 'dsc config' command
ms.date:     03/25/2025
ms.topic:    reference
title:       dsc config
---

# dsc config

## Synopsis

Apply a configuration document.

## Syntax

```sh
dsc config [Options] <COMMAND>
```

## Description

The `dsc config` command includes subcommands for managing the resource instances defined in a DSC
configuration document. To manage resources directly, see the [dsc resource][01] command.

## Commands

### export

The `export` command generates a configuration document that defines the existing instances of a
set of resources. For more information, see [dsc config export][02].

### get

The `get` command retrieves the current state of the resource instances in a configuration
document. For more information, see [dsc config get][03].

### set

The `set` command enforces the desired state of the resource instances in a configuration document.
For more information, see [dsc config set][04].

### test

The `test` command verifies whether the resource instances in a configuration document are in the
desired state. For more information, see [dsc config test][05].

### help

The `help` command returns help information for this command or a subcommand.

To get the help for a command or subcommand, use the syntax:

```sh
dsc config help [<SUBCOMMAND>]
```

For example, `dsc config help` gets the help for this command. `dsc config help set` gets the help
for the `set` subcommand.

You can also use the [--help](#--help) option on the command or subcommand to display the help
information. For example, `dsc config --help` or `dsc config set --help`.

## Options

### -f, --parameters-file

<a id="-f"></a>
<a id="--parameters-file"></a>

Specifies the path to a data file containing the parameters to pass to the configuration as JSON or
YAML. When you specify this option, DSC interprets the keys in the data file as parameters and uses
the specified values. The values in the data file override any defaults defined in the
configuration itself.

The data file must contain an object with the `parameters` key. The value of the `parameters` key
must be an object where each key is the name of a defined parameter and each value is a valid value
for that parameter.

This option is mutually exclusive with the `--parameters` option.

Starting with DSC version 3.1.0, you can pass the parameters data to a subcommand over stdin. When
you do, you must pass the configuration document as an input string or the path to a file on the
system. You can't pass both the parameters file and the configuration document to a command from
stdin.

For more information about defining parameters in a configuration document, see
[DSC Configuration document parameter schema][06]. For more information about using parameters in
configuration document, see the [parameters function reference][07].

```yaml
Type        : string
Mandatory   : false
LongSyntax  : --parameters-file <PARAMETERS_FILE>
ShortSyntax : -f <PARAMETERS_FILE>
```

### -p, --parameters

<a id="-p"></a>
<a id="--parameters"></a>

Specifies the parameters to pass to the configuration document as a string of data formatted as
JSON or YAML. When you specify this option, DSC interprets the keys in the data string as
parameters and uses the specified values. The values in the data string override any defaults
defined in the configuration document itself.

The data string must contain an object with the `parameters` key. The value of the `parameters` key
must be an object where each key is the name of a defined parameter and each value is a valid value
for that parameter.

This option is mutually exclusive with the `--parameters_file` option.

For more information about defining parameters in a configuration document, see
[DSC Configuration document parameter schema][06]. For more information about using parameters in
configuration document, see the [parameters function reference][07].

```yaml
Type        : string
Mandatory   : false
LongSyntax  : --parameters <PARAMETERS>
ShortSyntax : -p <PARAMETERS>
```

### -r, --system-root

<a id="-r"></a>
<a id="--system-root"></a>

Use this option to specify the path to the operating system root when you aren't targeting the
current running OS.

```yaml
Type        : string
Mandatory   : false
LongSyntax  : --system-root <SYSTEM_ROOT>
ShortSyntax : -r <SYSTEM_ROOT>
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

## Environment variables

The `dsc config *` subcommands create the `DSC_CONFIG_ROOT` environment variable when you call a
command with the `--path` option to specify the configuration document to use for the command. DSC
sets the value of the `DSC_CONFIG_ROOT` environment variable to the full path of the folder
containing the specified configuration document.

> [!NOTE]
> If you define the `DSC_CONFIG_ROOT` variable outside of DSC, DSC raises a warning when it
> overrides the existing environment variable's value for an operation.

You can use the [envvar][08] configuration function to reference that folder path for resource
instances in the configuration.

[01]: ../resource/index.md
[02]: ./export.md
[03]: ./get.md
[04]: ./set.md
[05]: ./test.md
[06]: ../../schemas/config/parameter.md
[07]: ../../schemas/config/functions/parameters.md
[08]: ../../schemas/config/functions/envvar.md
