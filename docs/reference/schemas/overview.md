---
description: >-
    Overview of the reference documentation for the JSON schemas describing data types for
    Microsoft's Desired State Configuration platform.
ms.date:     03/25/2025
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
configuration documents.

The article [DSC Configuration document functions reference][04] describes DSC configuration
functions generally and links to the reference documentation for the available functions.

## Extension schemas

The article [DSC command extension manifest schema reference][05] describes the root JSON schema for
extension manifests.

## Resource schemas

The article [DSC command resource manifest schema reference][06] describes the root JSON schema for
resource manifests.

The article [# DSC canonical properties reference][07] describes DSC canonical resource properties
generally and links to the reference documentation for the available canonical properties.

## Output schemas

The following table links to the reference documentation for the JSON schemas describing the output
DSC returns for its commands:

| Command              | Article link                                     |
|:---------------------|:-------------------------------------------------|
| `dsc config get`     | [dsc config get result schema reference][08]     |
| `dsc config set`     | [dsc config set result schema reference][09]     |
| `dsc config test`    | [dsc config test result schema reference][10]    |
| `dsc extension list` | [dsc extension list result schema reference][11] |
| `dsc resource get`   | [dsc resource get result schema reference][12]   |
| `dsc resource list`  | [dsc resource list result schema reference][13]  |
| `dsc resource set`   | [dsc resource set result schema reference][14]   |
| `dsc resource test`  | [dsc resource test result schema reference][15]  |

## Definition schemas

The following list defines the reference documentation for JSON schemas included as subschemas
throughout DSC.

- For more information about the `Microsoft.DSC` metadata property, see
  [Microsoft.DSC metadata property schema reference][16]
- For more information about the messages DSC emits, see [Structured message schema reference][17]
- For more information about the kinds of DSC resources and how they affect schema validation, see
  [DSC Resource kind schema reference][18].
- For more information about the naming of DSC resources and how they're validated, see
  [DSC Resource fully qualified type name schema reference][19]

<!-- Reference linki definitions -->
[01]: https://json-schema.org/overview/what-is-jsonschema
[02]: ./schema-uris.md
[03]: ./config/document.md
[04]: ./config/functions/overview.md
[05]: ./extension/manifest/root.md
[06]: ./resource/manifest/root.md
[07]: ./resource/properties/overview.md
[08]: ./outputs/config/get.md
[09]: ./outputs/config/set.md
[10]: ./outputs/config/test.md
[11]: ./outputs/extension/list.md
[12]: ./outputs/resource/get.md
[13]: ./outputs/resource/list.md
[14]: ./outputs/resource/set.md
[15]: ./outputs/resource/test.md
[16]: ./metadata/Microsoft.DSC/properties.md
[17]: ./definitions/message.md
[18]: ./definitions/resourceKind.md
[19]: ./definitions/resourceType.md
