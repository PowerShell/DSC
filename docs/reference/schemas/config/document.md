---
description: JSON schema reference for a Desired State Configuration document.
ms.date:     02/28/2025
ms.topic:    reference
title:       DSC Configuration document schema reference
---

# DSC Configuration document schema reference

## Synopsis

The YAML or JSON file that defines a DSC Configuration.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/config/document.json
Type:          object
```

## Description

DSC Configurations enable users to define state by combining different DSC Resources. A
configuration document uses parameters and variables to pass to a set of one or more resources that
define a desired state.

A configuration document can be defined as either YAML or JSON. For ease of authoring, Microsoft
recommends drafting configuration documents in YAML.

For DSC's authoring tools to recognize a file as a DSC Configuration document, the filename must
end with one of the following:

- `.dsc.config.json`
- `.dsc.config.yml`
- `.dsc.config.yaml`.
- `.dsc.json`
- `.dsc.yml`
- `.dsc.yaml`

You can use configuration document functions to dynamically determine values in the document at
runtime. For more information, see [DSC Configuration document functions reference][01]

<!-- For more information, see [DSC Configurations overview][01]. -->

The rest of this document describes the schema DSC uses to validation configuration documents.

## Examples

<!-- To-Do -->

## Required Properties

Every configuration document must include these properties:

- [$schema](#schema)
- [resources](#resources)

## Properties

### $schema

The `$schema` property indicates the URI that resolves to the version of this schema that the
document adheres to. DSC uses this property when validating and processing the configuration
document.

The JSON schemas for DSC are published in multiple versions and forms. This documentation is for
the latest version of the schema. As a convenience, you can specify either the full URI for the
schema hosted in GitHub or use the shorter `aka.ms` URI. You can specify the schema for a specific
semantic version, the latest schema for a minor version, or the latest schema for a major version
of DSC. For more information about schema URIs and versioning, see
[DSC JSON Schema URIs](../schema-uris.md).

For every version of the schema, there are three valid urls:

- `.../config/document.json`

  The URL to the canonical non-bundled schema. When it's used for validation, the validating client
  needs to retrieve this schema and every schema it references.

- `.../bundled/config/document.json`

  The URL to the canonically bundled schema. When it's used for validation, the validating client
  only needs to retrieve this schema.

  This schema uses the bundling model introduced for JSON Schema 2020-12. While DSC can still
  validate the document when it uses this schema, other tools may error or behave in unexpected
  ways if they don't fully support the 2020-12 specification.

- `.../bundled/config/document.vscode.json`

  The URL to the enhanced authoring schema. This schema is much larger than the other schemas, as
  it includes additional definitions that provide contextual help and snippets that the others
  don't include.

  This schema uses keywords that are only recognized by VS Code. While DSC can still validate the
  document when it uses this schema, other tools may error or behave in unexpected ways.

```yaml
Type:        string
Required:    true
Format:      URI
ValidValues: [
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/bundled/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/bundled/config/document.vscode.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/bundled/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/bundled/config/document.vscode.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/bundled/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/bundled/config/document.vscode.json
               https://aka.ms/dsc/schemas/v3/config/document.json
               https://aka.ms/dsc/schemas/v3/bundled/config/document.json
               https://aka.ms/dsc/schemas/v3/bundled/config/document.vscode.json
               https://aka.ms/dsc/schemas/v3.0/config/document.json
               https://aka.ms/dsc/schemas/v3.0/bundled/config/document.json
               https://aka.ms/dsc/schemas/v3.0/bundled/config/document.vscode.json
               https://aka.ms/dsc/schemas/v3.0.0/config/document.json
               https://aka.ms/dsc/schemas/v3.0.0/bundled/config/document.json
               https://aka.ms/dsc/schemas/v3.0.0/bundled/config/document.vscode.json
             ]
```

### metadata

The `metadata` property defines a set of key-value pairs as annotations for the configuration. DSC
doesn't validate the metadata. A configuration can include any arbitrary information in this
property.

```yaml
Type:     object
Required: false
```

### parameters

The `parameters` property defines a set of runtime options for the configuration. Each parameter is
defined as key-value pair. The key for each pair defines the name of the parameter. The value for
each pair must be an object that defines the `type` keyword to indicate how DSC should process the
parameter.

Parameters may be overridden at run-time, enabling re-use of the same configuration document for
different contexts.

For more information about defining parameters in a configuration, see
[DSC Configuration document parameter schema][02].

<!-- For more information about using parameters in a configuration, see
[DSC Configuration parameters][03] -->

```yaml
Type:                object
Required:            false
ValidPropertySchema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/config/document.parameter.json
```

### variables

The `variables` property defines a set of reusable values for the resources in the document as
key-value pairs. The key for each pair defines the name of the variable. Resources that reference
the variable by name can access the variable's value.

This can help reduce the amount of copied values and options for resources in the configuration,
which makes the document easier to read and maintain. Unlike parameters, variables can only be
defined in the configuration and can't be overridden at run-time.

<!-- For more information about using variables in a configuration, see
[DSC Configuration variables][04]. -->

```yaml
Type:     object
Required: false
```

### resources

The `resources` property defines a list of DSC Resource instances that the configuration manages.
Every instance in the list must be unique, but instances may share the same DSC Resource type.

For more information about defining a valid resource instance in a configuration, see
[DSC Configuration document resource schema][05].

<!-- For more information about how DSC uses resources in a configuration, see
[DSC Configuration resources][06] and [DSC Configuration resource groups][07]. -->

```yaml
Type:             array
Required:         true
MinimumItemCount: 1
ValidItemSchema:  https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/config/document.resource.json
```

<!-- Link reference definitions -->
[01]: functions/resourceId.md
<!-- [01]: ../../../configurations/overview.md -->
[02]: parameter.md
<!-- [03]: ../../../configurations/parameters.md -->
<!-- [04]: ../../../configurations/variables.md -->
[05]: resource.md
<!-- [06]: ../../../configurations/resources.md -->
<!-- [07]: ../../../configurations/resource-groups.md -->
