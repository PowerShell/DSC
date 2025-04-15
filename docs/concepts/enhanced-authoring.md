---
description: >-
  DSC configuration documents and resource manifests are YAML or JSON data files that adhere to a
  published JSON schema. You can use enhanced schemas when authoring these files for an improved
  experience.
ms.date: 03/25/2025
title:   Authoring with enhanced schemas
---

# Authoring with enhanced schemas

<!-- markdownlint-capture -->

Working with Microsoft's Desired State Configuration (DSC) platform involves writing DSC
[configuration documents][01] and [resource manifests][02]. Configuration documents are YAML or
JSON data files that declare the desired state of a system. Resource manifests are JSON or YAML
data files that define a DSC command resource.

DSC validates these data files with a JSON schema. While the schemas DSC uses for validation are
useful for authoring configuration documents and resource manifests, Microsoft also defines a set
of enhanced schemas for authoring the files in VS Code. These schemas define extra keywords
specific to VS Code that:

- Improve the contextual help when hovering on or selecting a property in the data file.
- Add contextual help for enum values.
- Improve the error messages for invalid values.
- Provide IntelliSense and default snippets.

> [!NOTE]
> The enhanced schemas are noncanonical. Only specify them for the `$schema` keyword in a
> configuration document or resource manifest when your tools support it.
>
> These schemas are only for improving the authoring experience. If you try to validate the
> configuration document or resource manifest with a tool that doesn't support the extended
> vocabulary, the tool might raise an error.
>
> The enhanced schemas share the same source definition as the canonical schemas and validate the
> data in the same way. However, they include noncanonical keywords. For maximum compatibility
> with other tools, the canonical schemas only use the core JSON schema vocabularies.

For the full list of recognized and supported schema URIs, including the enhanced authoring
schemas, see the `$schema` sections in [DSC Configuration document schema reference][03] and
[DSC resource manifest schema reference][04].

## Enhanced schema examples

### Example 1 - Documentation on hover

<!-- markdownlint-disable MD013 -->
:::image type="complex" source="media/enhanced-authoring/hover-help.png" alt-text="This screenshot shows enhanced hover documentation.":::
   This screenshot shows enhanced hover documentation for the 'type' property in a configuration document. The documentation includes a link at the top to the online documentation, followed by prose explaining the syntax for the property.
:::image-end:::
<!-- markdownlint-restore -->

With the enhanced schemas, hovering on a property displays contextual help rendered from Markdown.
When possible, the hover help includes a link to the online documentation.

### Example 2 - IntelliSense quick info

<!-- markdownlint-disable MD013 -->
:::image type="complex" source="media/enhanced-authoring/peek-help.png" alt-text="This screenshot shows enhanced documentation with autocomplete.":::
   This screenshot shows the DSC Resource instance autocomplete option and contextual documentation in a configuration document. The contextual help is shown to the side of the completion option list. The help includes a link to the online documentation, the descriptive prose, and the required properties.
:::image-end:::
<!-- markdownlint-restore -->

When you use IntelliSense while authoring with an enhanced schema, the quick info shown for the
completion options displays as rendered Markdown. When possible, the quick info includes a link to
the online documentation.

### Example 3 - Enum documentation

<!-- markdownlint-disable MD013 -->
:::image type="complex" source="media/enhanced-authoring/enum-help.png" alt-text="This screenshot shows contextual documentation for an enum.":::
   This screenshot shows the contextual documentation for the 'return' property enum values in a resource manifest. The contextual help is shown beneath the enum list, describing the currently selected value.
:::image-end:::
<!-- markdownlint-restore -->

The enhanced schemas add documentation for enum values when using IntelliSense to select a valid
value. By default, schemas can't provide per-enum documentation. They can only define documentation
for the property an enum belongs to.

### Example 4 - Snippets

<!-- markdownlint-disable MD013 -->
:::image type="complex" source="media/enhanced-authoring/snippet-selection.png" alt-text="This screenshot shows autocomplete snippet options with documentation.":::
   This screenshot shows the autocomplete snippets for the metadata section in a configuration document. The currently selected snippet displays contextual help.
:::image-end:::

:::image type="complex" source="media/enhanced-authoring/snippet-completion.png" alt-text="This screenshot shows the editable output for the chosen snippet.":::
   This screenshot shows the editable output for the 'New metadata property (object)' snippet. It defined a new property named 'metadataName' with a nested key-value pair. The property name, key, and value are editable text that a user can tab through, like any other VS Code snippet.
:::image-end:::
<!-- markdownlint-restore -->

For common use cases, the enhanced schemas define sets of default snippets. These snippets are
contextual and make it easier and faster to define the file. The snippets work like any other
snippets in VS Code.

### Example 5 - Error messages

<!-- markdownlint-disable MD013 -->
:::image type="complex" source="media/enhanced-authoring/error-messages.png" alt-text="This screenshot shows an enhanced error message for failed validation.":::
   This screenshot shows a contextual error message when the name property for a resource instance doesn't match the validating regular expression. The value is the string 'invalid?' and the error message says "Invalid value for instance name. An instance name must be a nonempty string containing only letters, numbers, and spaces."
:::image-end:::
<!-- markdownlint-restore -->

When you define values, the enhanced schemas provide contextual error messages instead of the
default error messages that JSON schema validation raises. These messages are helpful for
properties that must match a regular expression, where the default message just indicates that the
value is invalid and lists the regular expression pattern.

## Using the enhanced configuration document schema

To associate every configuration document with the enhanced schema, add the following snippet to
your `settings.json` file in VS Code. You can define these options in your user settings or for a
specific workspace.

<!-- markdownlint-disable MD013 -->
```json
"json.schemas": [
    {
        "fileMatch": [
            "**/*.dsc.json",
            "**/*.dsc.config.json"
        ],
        "url": "https://aka.ms/dsc/schemas/v3/bundled/config/document.vscode.json"
    }
],
"yaml.schemas": {
    "https://aka.ms/dsc/schemas/v3/bundled/config/document.vscode.json": "**.dsc.{yaml,yml,config.yaml,config.yml}"
}
```
<!-- markdownlint-restore -->

These settings depend on the configuration documents having a name that ends with one of the
following suffixes:

- `.dsc.config.json`
- `.dsc.config.yaml`
- `.dsc.config.yml`
- `.dsc.json`
- `.dsc.yaml`
- `.dsc.yml`

To associate a specific configuration document formatted as YAML with the enhanced schema, add the
following comment to the top of the document:

<!-- markdownlint-disable MD013 -->
```yml
# yaml-language-server: $schema=https://aka.ms/dsc/schemas/v3/bundled/config/document.vscode.json
```
<!-- markdownlint-resore -->

This option works regardless of the filename, but only for YAML files. To use the enhanced schema
when authoring configuration documents in JSON, you must define the schema association in your
`settings.json`.

## Using the enhanced resource manifest schema

To use the enhanced schema when authoring resource manifests, add the following snippet to
your `settings.json` file in VS Code. You can define this option in your user settings or for a
specific workspace.

<!-- markdownlint-disable MD013 -->
```json
"json.schemas": [
    {
        "fileMatch": ["**/*.dsc.resource.json", ],
        "url": "https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.vscode.json"
    }
]
"yaml.schemas": {
    "https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.vscode.json": "**.dsc.resource.{yaml,yml}"
}
```
<!-- markdownlint-restore -->

To associate a specific resource manifest formatted as YAML with the enhanced schema, add the
following comment to the top of the manifest:

<!-- markdownlint-disable MD013 -->
```yml
# yaml-language-server: $schema=https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.vscode.json
```
<!-- markdownlint-restore -->

This option works regardless of the filename, but only for YAML files. To use the enhanced schema
when authoring resource manifests in JSON, you must define the schema association in your
`settings.json`.

[01]: ./configuration-documents/overview.md
[02]: ./resources/anatomy.md#dsc-resource-manifests
[03]: ../reference/schemas/config/document.md#schema
[04]: ../reference/schemas/resource/manifest/root.md#schema
