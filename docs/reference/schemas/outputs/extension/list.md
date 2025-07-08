---
description: JSON schema reference for the data returned by the 'dsc extension list' command.
ms.date:     02/28/2025
ms.topic:    reference
title:       dsc extension list result schema reference
---

# dsc extension list result schema reference

## Synopsis

The result output from the `dsc extension list` command.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/outputs/extension/list.json
Type:          object
```

## Description

The output from the `dsc extension list` command includes a representation of discovered DSC
extensions as a series of [JSON Lines][01]. This schema describes the JSON object returned for each
extension.

## Required properties

Each extension in the output always includes these properties:

- [type](#type)
- [version](#version)
- [path](#path)
- [directory](#directory)
- [author](#author)
- [manifest](#manifest)

## Properties

### type

Identifies the fully qualified type name of the extension. For more information about extension type names, see
[DSC extension fully qualified type name schema reference][02].

```yaml
Type:     string
Required: true
Pattern:  ^\w+(\.\w+){0,2}\/\w+$
```

### version

Represents the current version of the extension as a valid semantic version (semver) string.

```yaml
Type:     string
Required: true
Pattern:  ^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-((?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+([0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?$
```

### capabilities

Defines the operations and behaviors the extension is implemented to support. This property is an
array of capabilities.

The following list describes the available capabilities for an extension:

- <a id="capability-discover" ></a> `discover` - The extension supports finding DSC resource
  manifests that aren't in the `PATH` or `DSC_RESOURCE_PATH`, as with resources installed as Appx
  packages.

```yaml
Type:              array
Required:          true
ItemsMustBeUnique: true
ItemsType:         string
ItemsValidValues: [
                    discover
                  ]
```

### description

Defines a synopsis for the extension's purpose as a short string. If the extension doesn't have a
description, this property is `null`.

```yaml
Type:     [string, 'null']
Required: true
```

### path

Represents the path to the extension's manifest on the machine.

```yaml
Type:     string
Required: true
```

### directory

Represents the path to the folder containing the extension's manifest on the machine.

```yaml
Type:     string
Required: true
```

### author

Indicates the name of the person or organization that developed and maintains the DSC extension. If
this property is `null`, the author is unknown.

```yaml
Type:     [string, 'null']
Required: true
```

### manifest

Represents the values defined in the extension's manifest. For more information on the value for
this property, see [Command-based DSC extension manifest schema reference][03].

```yaml
Type:     [object]
Required: true
```

<!-- Link reference definitions -->
[01]: https://jsonlines.org/
[02]: ../../definitions/extensionType.md
[03]: ../../extension/manifest/root.md
