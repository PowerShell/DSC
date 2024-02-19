---
description: Command line reference for the 'dsc config' command
ms.date:     02/05/2024
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

You can also use the [--help](#-h---help) option on the command or subcommand to display the help
information. For example, `dsc config --help` or `dsc config set --help`.

## Options

### -f, --parameters_file

Specifies the path to a data file containing the parameters to pass to the configuration as JSON or
YAML. When you specify this option, DSC interprets the keys in the data file as parameters and uses
the specified values. The values in the data file override any defaults defined in the
configuration itself.

The data file must contain an object with the `parameters` key. The value of the `parameters` key
must be an object where each key is the name of a defined parameter and each value is a valid value
for that parameter.

This option can't be used with the `--parameters` option. Choose whether to pass the parameters as
a data string with the `--parameters` option or in a data file with the `--parameters_file` option.

For more information about defining parameters in a configuration document, see
[DSC Configuration document parameter schema][06]. For more information about using parameters in
configuration document, see the [parameters function reference][07].

### -p, --parameters

Specifies the parameters to pass to the configuration as a JSON or YAML string. When you specify
this option, DSC interprets the keys in the data string as parameters and uses the specified
values. The values in the data string override any defaults defined in the configuration itself.

The data string must contain an object with the `parameters` key. The value of the `parameters` key
must be an object where each key is the name of a defined parameter and each value is a valid value
for that parameter.

This option can't be used with the `--parameters_file` option. Choose whether to pass the
parameters as a data string with the `--parameters` option or in a data file with the
`--parameters_file` option.

For more information about defining parameters in a configuration document, see
[DSC Configuration document parameter schema][06]. For more information about using parameters in
configuration document, see the [parameters function reference][07].

### -h, --help

Displays the help for the current command or subcommand. When you specify this option, the
application ignores all options and arguments after this one.

```yaml
Type:      Boolean
Mandatory: false
```

[01]: ../resource/command.md
[02]: export.md
[03]: get.md
[04]: set.md
[05]: test.md
[06]: ../../schemas/config/parameter.md
[07]: ../../schemas/config/functions/parameters.md
