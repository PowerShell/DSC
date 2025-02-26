---
description: Command line reference for the 'dsc schema' command
ms.date:     02/28/2025
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

> [!NOTE]
> Currently, the schemas returned by the `dsc schema` command and those published to GitHub are
> not the same. The published schemas more fully describe and validate the data than the schemas
> emitted by the command. The DSC team is working to canonicalize the schemas returned from the
> command.
>
> Both the published schemas and those returned from this command correctly validate the data. The
> schemas returned from this command are less strict than the published schemas. Even though data
> validates against the schemas returned by this command, DSC may raise errors when processing the
> data. For example, the returned schema for versions indicates that the valid value is a string -
> but if you specify a string that isn't a semantic version, DSC raises an error. In that case, the
> data passed the schema validation but was incorrect.
>
> Until the schemas are canonicalized, consider using the published schemas when indpendently
> testing your configuration documents and resource manifests with a JSON Schema validation tool.

## Examples

### Example 1 - Retrieve the schema for the dsc resource get command result

```sh
dsc schema --type get-result
```

```yaml
$schema: http://json-schema.org/draft-07/schema#
title: GetResult
anyOf:
- $ref: '#/definitions/ResourceGetResponse'
- type: array
  items:
    $ref: '#/definitions/ResourceGetResult'
definitions:
  ResourceGetResponse:
    type: object
    required:
    - actualState
    properties:
      actualState:
        description: The state of the resource as it was returned by the Get method.
    additionalProperties: false
  ResourceGetResult:
    type: object
    required:
    - name
    - result
    - type
    properties:
      metadata:
        anyOf:
        - $ref: '#/definitions/Metadata'
        - type: 'null'
      name:
        type: string
      type:
        type: string
      result:
        $ref: '#/definitions/GetResult'
    additionalProperties: false
  Metadata:
    type: object
    properties:
      Microsoft.DSC:
        anyOf:
        - $ref: '#/definitions/MicrosoftDscMetadata'
        - type: 'null'
  MicrosoftDscMetadata:
    type: object
    properties:
      version:
        description: Version of DSC
        type:
        - string
        - 'null'
      operation:
        description: The operation being performed
        anyOf:
        - $ref: '#/definitions/Operation'
        - type: 'null'
      executionType:
        description: The type of execution
        anyOf:
        - $ref: '#/definitions/ExecutionKind'
        - type: 'null'
      startDatetime:
        description: The start time of the configuration operation
        type:
        - string
        - 'null'
      endDatetime:
        description: The end time of the configuration operation
        type:
        - string
        - 'null'
      duration:
        description: The duration of the configuration operation
        type:
        - string
        - 'null'
      securityContext:
        description: The security context of the configuration operation, can be specified to be required
        anyOf:
        - $ref: '#/definitions/SecurityContextKind'
        - type: 'null'
      context:
        description: Identifies if the operation is part of a configuration
        anyOf:
        - $ref: '#/definitions/ContextKind'
        - type: 'null'
  Operation:
    type: string
    enum:
    - Get
    - Set
    - Test
    - Export
  ExecutionKind:
    type: string
    enum:
    - Actual
    - WhatIf
  SecurityContextKind:
    type: string
    enum:
    - Current
    - Elevated
    - Restricted
  ContextKind:
    type: string
    enum:
    - Configuration
    - Resource
  GetResult:
    anyOf:
    - $ref: '#/definitions/ResourceGetResponse'
    - type: array
      items:
        $ref: '#/definitions/ResourceGetResult'
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
