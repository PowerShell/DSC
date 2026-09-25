---
description: >-
    Overview of the reference documentation for the JSON schemas describing data types for
    Microsoft's Desired State Configuration platform.
ms.date:     09/01/2026
ms.topic:    reference
title:       DSC JSON Schema reference overview
---

# DSC JSON Schema reference overview

Microsoft's Desired State Configuration platform uses [JSON schemas][01] to describe and validate
the data that DSC takes as input and returns as output.

These schemas define the structure, purpose, and validation for data in DSC and are published to
the DSC GitHub repository. DSC publishes updated schemas with every release. Each schema has an
`$id` keyword that uniquely identifies the schema. For convenience, DSC provides shortened links to
the schemas under the `aka.ms/dsc/schemas` namespace.

For more information about how the DSC schemas are published and the URIs that identify them, see
[DSC JSON Schema URIs][02].

The articles in this section provide reference documentation for the latest supported version of
the DSC schemas.

## Configuration document schemas

The article [DSC configuration document schema reference][03] describes the root JSON schema for
configuration documents. The following articles describe the subschemas for the properties of a
configuration document:

- [DSC Configuration document metadata schema][04]
- [DSC Configuration document parameter schema][05]
- [DSC configuration parameter data type schema reference][06]
- [DSC Configuration document resource instance schema][07]

The article [DSC Configuration document functions reference][08] describes DSC configuration
functions generally and links to the reference documentation for the available functions. The
article [Function data types schema reference][09] describes the data types that configuration
functions accept and return.

## Extension schemas

The article [DSC command extension manifest schema reference][10] describes the root JSON schema for
extension manifests. The following articles describe the schemas for the operations an extension
can define:

- [DSC extension manifest discover property schema reference][11]
- [DSC extension discover operation stdout schema reference][12]

## Resource schemas

The article [DSC command resource manifest schema reference][13] describes the root JSON schema for
resource manifests. The following articles describe the subschemas for the properties of a resource
manifest:

- [DSC Resource manifest adapter property schema reference][14]
- [DSC Resource manifest delete property schema reference][15]
- [DSC Resource manifest export property schema reference][16]
- [DSC Resource manifest get property schema reference][17]
- [DSC Resource manifest resolve property schema reference][18]
- [DSC Resource manifest schema property schema reference][19]
- [DSC Resource manifest embedded schema reference][20]
- [DSC Resource manifest set property schema reference][21]
- [DSC Resource manifest test property schema reference][22]
- [DSC Resource manifest validate property schema reference][23]
- [DSC Resource manifest whatIf property schema reference][24]

The article [DSC canonical properties reference][25] describes DSC canonical resource properties
generally and links to the reference documentation for the available canonical properties:

- [DSC Resource _ensure property schema][26]
- [DSC Resource _exist property schema][27]
- [DSC Resource _inDesiredState property schema][28]
- [DSC Resource _purge property schema][29]

The article [Overview of DSC resource operation stdout schemas][30] describes the data a command
resource must return for each operation and links to the reference documentation for each
operation:

- [DSC resource delete operation stdout schema reference][31]
- [DSC resource export operation stdout schema reference][32]
- [DSC resource get operation stdout schema reference][33]
- [DSC resource list operation stdout schema reference][34]
- [DSC resource resolve operation stdout schema reference][35]
- [DSC resource schema command stdout schema reference][36]
- [DSC resource set operation stdout schema reference][37]
- [DSC resource test operation stdout schema reference][38]
- [DSC resource validate operation stdout schema reference][39]
- [DSC resource what-if operation stdout schema reference][40]

## Output schemas

The following table links to the reference documentation for the JSON schemas describing the output
DSC returns for its commands:

| Command              | Article link                                     |
|:---------------------|:-------------------------------------------------|
| `dsc config get`     | [dsc config get result schema reference][41]     |
| `dsc config set`     | [dsc config set result schema reference][42]     |
| `dsc config test`    | [dsc config test result schema reference][43]    |
| `dsc extension list` | [dsc extension list result schema reference][44] |
| `dsc function list`  | [dsc function list result schema reference][45]  |
| `dsc resource get`   | [dsc resource get result schema reference][46]   |
| `dsc resource list`  | [dsc resource list result schema reference][47]  |
| `dsc resource set`   | [dsc resource set result schema reference][48]   |
| `dsc resource test`  | [dsc resource test result schema reference][49]  |

## Definition schemas

The following list defines the reference documentation for JSON schemas included as subschemas
throughout DSC.

- For more information about the `Microsoft.DSC` metadata property, see
  [Microsoft.DSC metadata property schema reference][50].
- For more information about the messages DSC emits, see [Structured message schema reference][51].
- For more information about the kinds of DSC resources and how they affect schema validation, see
  [DSC Resource kind schema reference][52].
- For more information about the naming of DSC resources and how they're validated, see
  [DSC Resource fully qualified type name schema reference][53].
- For more information about the operations a DSC resource supports, see
  [DSC Resource capabilities schema reference][54].
- For more information about the data types for configuration document parameters, see
  [DSC configuration parameter data type schema reference][06].
- For more information about the data types that configuration functions operate on, see
  [Function data types schema reference][09].

<!-- Reference link definitions -->
[01]: https://json-schema.org/overview/what-is-jsonschema
[02]: ./schema-uris.md
[03]: ./config/document.md
[04]: ./config/metadata.md
[05]: ./config/parameter.md
[06]: ./definitions/parameters/dataTypes.md
[07]: ./config/resource.md
[08]: ./config/functions/overview.md
[09]: ./definitions/functions/builtin/dataTypes.md
[10]: ./extension/manifest/root.md
[11]: ./extension/manifest/discover.md
[12]: ./extension/stdout/discover.md
[13]: ./resource/manifest/root.md
[14]: ./resource/manifest/adapter.md
[15]: ./resource/manifest/delete.md
[16]: ./resource/manifest/export.md
[17]: ./resource/manifest/get.md
[18]: ./resource/manifest/resolve.md
[19]: ./resource/manifest/schema/property.md
[20]: ./resource/manifest/schema/embedded.md
[21]: ./resource/manifest/set.md
[22]: ./resource/manifest/test.md
[23]: ./resource/manifest/validate.md
[24]: ./resource/manifest/whatif.md
[25]: ./resource/properties/overview.md
[26]: ./resource/properties/ensure.md
[27]: ./resource/properties/exist.md
[28]: ./resource/properties/inDesiredState.md
[29]: ./resource/properties/purge.md
[30]: ./resource/stdout/index.md
[31]: ./resource/stdout/delete.md
[32]: ./resource/stdout/export.md
[33]: ./resource/stdout/get.md
[34]: ./resource/stdout/list.md
[35]: ./resource/stdout/resolve.md
[36]: ./resource/stdout/schema.md
[37]: ./resource/stdout/set.md
[38]: ./resource/stdout/test.md
[39]: ./resource/stdout/validate.md
[40]: ./resource/stdout/whatIf.md
[41]: ./outputs/config/get.md
[42]: ./outputs/config/set.md
[43]: ./outputs/config/test.md
[44]: ./outputs/extension/list.md
[45]: ./outputs/function/list.md
[46]: ./outputs/resource/get.md
[47]: ./outputs/resource/list.md
[48]: ./outputs/resource/set.md
[49]: ./outputs/resource/test.md
[50]: ./metadata/Microsoft.DSC/properties.md
[51]: ./definitions/message.md
[52]: ./definitions/resourceKind.md
[53]: ./definitions/resourceType.md
[54]: ./definitions/resourceCapabilities.md
