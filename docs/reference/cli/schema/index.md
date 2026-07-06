---
description: Command line reference for the 'dsc schema' command
ms.date:     03/25/2025
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
> validates against the schemas returned by this command, DSC might raise errors when processing
> the data. For example, the returned schema for versions indicates that the valid value is a
> string - but if you specify a string that isn't a semantic version, DSC raises an error. In that
> case, the data passed the schema validation but was incorrect.
>
> Until the schemas are canonicalized, consider using the published schemas when indpendently
> testing your configuration documents and resource manifests with a JSON Schema validation tool.

## Examples

### Example 1 - Retrieve the schema for the dsc resource get command result

<a id="example-1"></a>

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

<a id="-t"></a>
<a id="--type"></a>

This option is mandatory for the `schema` command. The value for this option determines which
schema the application returns:

- `configuration` ([reference documentation][aa]) - Validates a DSC Configuration document. If the document is invalid, DSC raises an error.
- `configuration-get-result` ([reference documentation][ab]) - Represents the output from the `dsc config get` command.
- `configuration-set-result` ([reference documentation][ac]) - Represents the output from the `dsc config set` command.
- `configuration-test-result` ([reference documentation][ad]) - Represents the output from the `dsc config test` command.
- `dsc-resource` ([reference documentation][ae]) - Represents a DSC Resource as returned from the `dsc resource list` command.
- `extension-discover-result` ([reference documentation][af]) - Represents the output DSC expects an extension to return for the `discover` operation.
- `extension-manifest` ([reference documentation][ag]) - Validates an extension's manifest. If the manifest is invalid, DSC raises an error.
- `function-definition` ([reference documentation][ah]) - Represents a DSC function as returned from the `dsc function list` command.
- `get-result` ([reference documentation][ai]) - Represents the output from the `dsc resource get` command.
- `include` ([reference documentation][aj]) - represents the instance schema for the built-in `Microsoft.DSC/Include` importer resource.
- `manifest-list` ([reference documentation][ak])
- `resolve-result` ([reference documentation][al]) - Represents the resolved form of the configuration document an `importer` resource emits.
- `resource` ([reference documentation][am]) - Represents an instance of a resource in a configuration document.
- `resource-manifest` ([reference documentation][an]) - Validates a command resource's manifest. If the manifest is invalid, DSC raises an error.
- `restart-required` ([reference documentation][ao])
- `set-result` ([reference documentation][ap]) - Represents the output from the `dsc resource set` command.
- `test-result` ([reference documentation][aq]) - Represents the output from the `dsc resource test` command.

```yaml
Type:        string
Mandatory:   true
ValidValues: [
               configuration
               configuration-get-result
               configuration-set-result
               configuration-test-result
               dsc-resource
               extension-discover-result
               extension-manifest
               function-definition
               get-result
               include
               manifest-list
               resolve-result
               resource-manifest
               restart-required
               set-result
               test-result
             ]
LongSyntax  : --type <TYPE>
ShortSyntax : -t <TYPE>
```

### -o, --output-format

<a id="-o"></a>
<a id="--output-format"></a>

The `--output-format` option controls which format DSC uses for the data the command returns. The
available formats are:

- `json` to emit the data as a [JSON Line][ar].
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

This command returns formatted data representing a JSON Schema specified by the
[--type option](#--type).

For more information about the formatting of the output data, see the
[--output-format option](#--output-format).

[aa]: ../../schemas/config/document.md
[ab]: ../../schemas/outputs/config/get.md
[ac]: ../../schemas/outputs/config/set.md
[ad]: ../../schemas/outputs/config/test.md
[ae]: ../../schemas/outputs/resource/list.md
[af]: ../../schemas/extension/stdout/discover.md
[ag]: ../../schemas/extension/manifest/root.md
[ah]: ../../schemas/outputs/function/list.md
[ai]: ../../schemas/outputs/resource/get.md
[aj]: ../../../reference/resources/microsoft/dsc/include/index.md
[ak]: ../../schemas/definitions/manifest-list.md
[al]: ../../schemas/resource/stdout/resolve
[am]: ../../schemas/config/resource.md
[an]: ../../schemas/resource/manifest/root.md
[ao]: ../../schemas/resource/properties/restartRequired.md
[ap]: ../../schemas/outputs/resource/set.md
[aq]: ../../schemas/outputs/resource/test.md
[ar]: https://jsonlines.org/
