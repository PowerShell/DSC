---
description: Command line reference for the 'dsc schema' command
ms.date:     08/04/2023
ms.topic:    reference
title:       dsc schema
---

# dsc schema

## Synopsis

Gets the JSON schema for a DSC type.

## Syntax

```sh
dsc schema [Options] --type <TYPE>
```

## Description

The `schema` command returns the JSON schema for a specific DSC type. These schemas can be used to
validate the return data from the application or to generate compatible type definitions for an
integrating tool.

The application uses these schemas to validate data internally when it's received or represent the
output for one of the application's commands.

## Examples

### Example 1 - Retrieve the schema for the dsc resource get command result

```sh
dsc schema --type get-result
```

```yaml
$schema: http://json-schema.org/draft-07/schema#
title: GetResult
type: object
required:
- actualState
properties:
  actualState:
    description: The state of the resource as it was returned by the Get method.
additionalProperties: false
```

## Options

### -t, --type

This option is mandatory for the `schema` command. The value for this option determines which
schema the application returns:

- `dsc-resource` ([reference documentation][01]) - Represents a DSC Resource as returned from the
  `dsc resource list` command.
- `resource-manifest` ([reference documentation][02]) - Validates a command-based DSC Resource's
  manifest. If the manifest is invalid, DSC raises an error.
- `get-result` ([reference documentation][03]) - Represents the output from the `dsc resource get`
  command.
- `set-result` ([reference documentation][04]) - Represents the output from the `dsc resource set`
  command.
- `test-result` ([reference documentation][05]) - Represents the output from the
  `dsc resource test` command.
- `configuration` ([reference documentation][06]) - Validates a DSC Configuration document. If the
  document is invalid, DSC raises an error.
- `configuration-get-result` ([reference documentation][07]) - Represents the output from the
  `dsc config get` command.
- `configuration-set-result` ([reference documentation][08]) - Represents the output from the
  `dsc config set` command.
- `configuration-test-result` ([reference documentation][09]) - Represents the output from the
  `dsc config test` command.

```yaml
Type:        String
Mandatory:   true
ValidValues: [
               dsc-resource,
               resource-manifest,
               get-result,
               set-result,
               test-result,
               configuration,
               configuration-get-result,
               configuration-set-result,
               configuration-test-result
             ]
```

### -f, --format

The `--format` option controls the console output format for the command. If the command output is
redirected or captured as a variable, the output is always JSON.

```yaml
Type:         String
Mandatory:    false
DefaultValue: yaml
ValidValues:  [json, pretty-json, yaml]
```

### -h, --help

Displays the help for the current command or subcommand. When you specify this option, the
application ignores all options and arguments after this one.

```yaml
Type:      Boolean
Mandatory: false
```

[01]: ../../schemas/outputs/resource/list.md
[02]: ../../schemas/resource/manifest/root.md
[03]: ../../schemas/outputs/resource/get.md
[04]: ../../schemas/outputs/resource/set.md
[05]: ../../schemas/outputs/resource/test.md
[06]: ../../schemas/config/document.md
[07]: ../../schemas/outputs/config/get.md
[08]: ../../schemas/outputs/config/set.md
[09]: ../../schemas/outputs/config/test.md
