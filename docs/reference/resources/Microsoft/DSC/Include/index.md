---
description: Microsoft.DSC/Include resource reference documentation
ms.date:     07/24/2026
ms.topic:    reference
title:       Microsoft.DSC/Include
---

# Microsoft.DSC/Include

## Synopsis

Includes a nested configuration document, with optional parameters, into the current configuration.

## Metadata

```yaml
Version    : 0.1.0
Kind       : importer
Tags       : [Windows, Linux, MacOS]
Author     : Microsoft
```

## Instance definition syntax

```yaml
resources:
  - name: <instance name>
    type: Microsoft.DSC/Include
    properties:
      # Specify the nested configuration with one of the following properties:
      configurationFile:    string # Path to a configuration document file.
      configurationContent: string # Inline configuration document as text.
      # Optionally specify parameters with one of the following properties:
      parametersFile:    string # Path to a parameters file.
      parametersContent: string # Inline parameters as text.
```

## Description

The `Microsoft.DSC/Include` resource lets you compose a configuration from more than one
configuration document. Instead of defining every resource instance in a single document, you can
author smaller documents and reference them from a parent document. When DSC processes an instance
of the `Microsoft.DSC/Include` resource, it resolves the nested configuration document, applies any
parameters you specify, and runs the operation against the nested resources as a group.

Use the `Microsoft.DSC/Include` resource when you want to:

- Reuse a common configuration document across multiple parent configurations.
- Split a large configuration into smaller, focused documents that are easier to maintain.
- Apply the same configuration with different parameter values in different contexts.

You define the nested configuration in one of two mutually exclusive ways:

- Reference a document on disk with the [configurationFile](#configurationfile) property.
- Embed the document inline as text with the [configurationContent](#configurationcontent) property.

Similarly, you can optionally pass parameters to the nested document in one of two mutually
exclusive ways:

- Reference a parameters file on disk with the [parametersFile](#parametersfile) property.
- Embed the parameters inline as text with the [parametersContent](#parameterscontent) property.

Both the configuration and the parameters can be authored as either YAML or JSON. DSC detects the
format when it parses the content.

> [!NOTE]
> This resource is installed with DSC itself on any systems.
>
> You can update this resource by updating DSC. When you update DSC, the updated version of this
> resource is automatically available.

### Path resolution and security

When you specify a relative path for the [configurationFile](#configurationfile) or
[parametersFile](#parametersfile) properties, DSC resolves the path against the directory of the
parent configuration document. DSC uses the `DSC_CONFIG_ROOT` environment variable to determine
that directory. When you invoke DSC with configuration content instead of a file on disk, DSC
resolves relative paths against the current working directory.

For security, relative paths **can't** reference a parent directory. A path that contains a `..`
segment raises an error. To include a configuration outside of the parent document's directory, use
an absolute path or construct the path with a configuration function like [path()][05] or
[systemRoot()][06].

### Nested and repeated includes

An included configuration document can itself contain instances of the `Microsoft.DSC/Include`
resource. DSC resolves each level of nesting in turn, so you can build a hierarchy of configuration
documents. You can also define more than one `Microsoft.DSC/Include` instance in the same document
to compose several configurations together.

## Capabilities

The resource has the following capabilities:

- `get` - You can use the resource to retrieve the actual state of the nested resource instances.
- `set` - You can use the resource to enforce the desired state for the nested resource instances.
- `test` - You can use the resource to check whether the nested resource instances are in the
  desired state.

Because this resource is an [importer resource][03], it doesn't manage state directly. Instead, it
resolves the nested configuration document and DSC invokes the requested operation against the
resources defined in that document.

For more information about resource capabilities, see [DSC resource capabilities][01].

## Examples

1. [Include a configuration file][07] - Shows how to reference a configuration document and pass it
   a parameters file.
1. [Include inline configuration content][08] - Shows how to embed a configuration document and its
   parameters directly in the parent document.

## Properties

The following list describes the properties for the resource.

- **Configuration properties:** <a id="configuration-properties"></a> You must define exactly one of
  the following properties to specify the nested configuration document. An instance that defines
  neither property, or both, is invalid.

  - [configurationFile](#configurationfile) - The path to a configuration document file.
  - [configurationContent](#configurationcontent) - An inline configuration document as text.

- **Parameter properties:** <a id="parameter-properties"></a> You can optionally define one of the
  following properties to pass parameters to the nested configuration document. An instance that
  defines both properties is invalid.

  - [parametersFile](#parametersfile) - The path to a parameters file.
  - [parametersContent](#parameterscontent) - Inline parameters as text.

### configurationFile

<details><summary>Expand for <code>configurationFile</code> property metadata</summary>

```yaml
Type             : string
IsRequired       : false
IsKey            : false
IsReadOnly       : false
IsWriteOnly      : false
```

</details>

Defines the path to the configuration document file to include. The file can be authored as YAML or
JSON. When you specify a relative path, DSC resolves it against the parent configuration document's
directory and the path can't reference a parent directory. For more information, see
[Path resolution and security](#path-resolution-and-security).

Define either this property or [configurationContent](#configurationcontent), but not both.

### configurationContent

<details><summary>Expand for <code>configurationContent</code> property metadata</summary>

```yaml
Type             : string
IsRequired       : false
IsKey            : false
IsReadOnly       : false
IsWriteOnly      : false
```

</details>

Defines the configuration document to include as inline text. The content can be authored as YAML or
JSON. Use this property when you want to keep the nested configuration in the same document rather
than referencing a separate file.

Define either this property or [configurationFile](#configurationfile), but not both.

### parametersFile

<details><summary>Expand for <code>parametersFile</code> property metadata</summary>

```yaml
Type             : string
IsRequired       : false
IsKey            : false
IsReadOnly       : false
IsWriteOnly      : false
```

</details>

Defines the path to a parameters file that supplies values for the parameters defined in the nested
configuration document. The file can be authored as YAML or JSON. When you specify a relative path,
DSC resolves it against the parent configuration document's directory and the path can't reference a
parent directory. For more information, see
[Path resolution and security](#path-resolution-and-security).

Define either this property or [parametersContent](#parameterscontent), but not both. This property
is optional. When you don't specify parameters, the nested configuration uses the default value for
each of its parameters.

### parametersContent

<details><summary>Expand for <code>parametersContent</code> property metadata</summary>

```yaml
Type             : string
IsRequired       : false
IsKey            : false
IsReadOnly       : false
IsWriteOnly      : false
```

</details>

Defines the parameters for the nested configuration document as inline text. The content can be
authored as YAML or JSON. Use this property when you want to keep the parameter values in the same
document rather than referencing a separate file.

Define either this property or [parametersFile](#parametersfile), but not both. This property is
optional. When you don't specify parameters, the nested configuration uses the default value for
each of its parameters.

## Instance validating schema

The following snippet contains the JSON Schema that validates an instance of the resource. The
validating schema only includes schema keywords that affect how the instance is validated. All
non validating keywords are omitted.

```json
{
  "type": "object",
  "properties": {
    "configurationFile": {
      "title": "Configuration file",
      "description": "The path to the configuration document file to include.",
      "type": "string"
    },
    "configurationContent": {
      "title": "Configuration content",
      "description": "The configuration document to include as inline text.",
      "type": "string"
    },
    "parametersFile": {
      "title": "Parameters file",
      "description": "The path to a parameters file for the included configuration.",
      "type": "string"
    },
    "parametersContent": {
      "title": "Parameters content",
      "description": "The parameters for the included configuration as inline text.",
      "type": "string"
    }
  },
  "oneOf": [
    { "required": ["configurationFile"] },
    { "required": ["configurationContent"] }
  ],
  "not": {
    "required": ["parametersFile", "parametersContent"]
  },
  "additionalProperties": false
}
```

## See also

- [Microsoft.DSC/Group resource][09]
- [Microsoft.DSC/Assertion resource][10]
- [Microsoft/OSInfo resource][11]
- [DSC configuration documents][04]
- [DSC resource capabilities][01]

<!-- Link reference definitions -->
[01]: ../../../../../concepts/resources/capabilities.md
[03]: ../../../../../concepts/resources/kinds.md
[04]: ../../../../../concepts/configuration-documents/overview.md
[05]: ../../../../schemas/config/functions/path.md
[06]: ../../../../schemas/config/functions/systemRoot.md
[07]: ./examples/include-a-configuration-file.md
[08]: ./examples/include-inline-configuration-content.md
[09]: ../Group/index.md
[10]: ../Assertion/index.md
[11]: ../../../Microsoft/OSInfo/index.md
