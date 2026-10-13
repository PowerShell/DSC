---
description: JSON schema reference for the data returned by the 'dsc extension list' command.
ms.date:     09/01/2026
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
- [capabilities](#capabilities)
- [import](#import)
- [path](#path)
- [deprecation_message](#deprecation_message)
- [description](#description)
- [directory](#directory)
- [author](#author)
- [manifest](#manifest)

## Properties

### type

Identifies the fully qualified type name of the extension. Extension type names use the same
syntax as resource type names. For more information, see
[DSC Resource fully qualified type name schema reference][02].

```yaml
Type:     string
Required: true
Pattern:  ^\w+(\.\w+)*\/\w+$
```

### version

Represents the current version of the extension as a valid semantic version (SemVer) string.

```yaml
Type:     string
Required: true
Pattern:  ^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-((?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+([0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?$
```

### capabilities

Defines the operations and behaviors the extension is implemented to support. This property is an
array of capabilities. DSC infers the capabilities from the properties defined in the extension
manifest.

The following list describes the available capabilities for an extension:

- <a id="capability-discover" ></a> `discover` - The extension supports finding DSC resource
  manifests that aren't in the `PATH` or `DSC_RESOURCE_PATH`, as with resources installed as Appx
  packages. An extension has this capability when its manifest defines the [discover][03]
  property.

- <a id="capability-secret"></a> `secret` - The extension supports retrieving secret values from a
  vault at runtime instead of passing sensitive values directly to a command or with a parameter
  file. An extension has this capability when its manifest defines the [secret][04] property.

- <a id="capability-import"></a> `import` - The extension supports resolving files as DSC
  configuration documents to enable passing those files to DSC for `dsc config` commands. An
  extension has this capability when its manifest defines the [import][05] property.

```yaml
Type:              array
Required:          true
ItemsMustBeUnique: true
ItemsType:         string
ItemsValidValues: [
                    discover,
                    secret,
                    import
                  ]
```

### import

Represents the definition of the `import` operation for the extension as defined in its manifest.
If the extension doesn't define the `import` property in its manifest, or defines it with an empty
`fileExtensions` list, this property is `null`. For more information about the value for this
property, see the [import][05] property in the extension manifest schema reference.

```yaml
Type:     [object, 'null']
Required: true
```

### path

Represents the path to the extension's manifest on the machine.

```yaml
Type:     string
Required: true
```

### deprecation_message

Represents the deprecation message for the extension as defined in its manifest. If the extension
isn't deprecated, this property is `null`. Note that this property uses an underscore in its name
rather than the camel case used by the `deprecationMessage` property in the manifest.

```yaml
Type:     [string, 'null']
Required: true
```

### description

Defines a synopsis for the extension's purpose as a short string. If the extension doesn't have a
description, this property is `null`.

```yaml
Type:     [string, 'null']
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
this property, see [Command-based DSC extension manifest schema reference][06].

```yaml
Type:     object
Required: true
```

<!-- Link reference definitions -->
[01]: https://jsonlines.org/
[02]: ../../definitions/resourceType.md
[03]: ../../extension/manifest/root.md#discover
[04]: ../../extension/manifest/root.md#secret
[05]: ../../extension/manifest/root.md#import
[06]: ../../extension/manifest/root.md
