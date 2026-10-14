---
description: JSON schema reference for a Desired State Configuration document.
ms.date:     10/13/2026
ms.topic:    reference
title:       DSC Configuration document schema reference
---

# DSC Configuration document schema reference

## Synopsis

The YAML or JSON file that defines a DSC Configuration.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/config/document.json
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

For every version of the schema, there are three valid URLs:

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
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2/bundled/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2/bundled/config/document.vscode.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.3/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.3/bundled/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.3/bundled/config/document.vscode.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.2/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.2/bundled/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.2/bundled/config/document.vscode.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.1/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.1/bundled/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.1/bundled/config/document.vscode.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.0/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.0/bundled/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.0/bundled/config/document.vscode.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1/bundled/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1/bundled/config/document.vscode.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.3/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.3/bundled/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.3/bundled/config/document.vscode.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.2/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.2/bundled/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.2/bundled/config/document.vscode.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.1/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.1/bundled/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.1/bundled/config/document.vscode.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/bundled/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/bundled/config/document.vscode.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/bundled/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/bundled/config/document.vscode.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.2/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.2/bundled/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.2/bundled/config/document.vscode.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.1/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.1/bundled/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.1/bundled/config/document.vscode.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/bundled/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/bundled/config/document.vscode.json
               https://aka.ms/dsc/schemas/v3/config/document.json
               https://aka.ms/dsc/schemas/v3/bundled/config/document.json
               https://aka.ms/dsc/schemas/v3/bundled/config/document.vscode.json
               https://aka.ms/dsc/schemas/v3.2/config/document.json
               https://aka.ms/dsc/schemas/v3.2/bundled/config/document.json
               https://aka.ms/dsc/schemas/v3.2/bundled/config/document.vscode.json
               https://aka.ms/dsc/schemas/v3.2.3/config/document.json
               https://aka.ms/dsc/schemas/v3.2.3/bundled/config/document.json
               https://aka.ms/dsc/schemas/v3.2.3/bundled/config/document.vscode.json
               https://aka.ms/dsc/schemas/v3.2.2/config/document.json
               https://aka.ms/dsc/schemas/v3.2.2/bundled/config/document.json
               https://aka.ms/dsc/schemas/v3.2.2/bundled/config/document.vscode.json
               https://aka.ms/dsc/schemas/v3.2.1/config/document.json
               https://aka.ms/dsc/schemas/v3.2.1/bundled/config/document.json
               https://aka.ms/dsc/schemas/v3.2.1/bundled/config/document.vscode.json
               https://aka.ms/dsc/schemas/v3.2.0/config/document.json
               https://aka.ms/dsc/schemas/v3.2.0/bundled/config/document.json
               https://aka.ms/dsc/schemas/v3.2.0/bundled/config/document.vscode.json
               https://aka.ms/dsc/schemas/v3.1/config/document.json
               https://aka.ms/dsc/schemas/v3.1/bundled/config/document.json
               https://aka.ms/dsc/schemas/v3.1/bundled/config/document.vscode.json
               https://aka.ms/dsc/schemas/v3.1.3/config/document.json
               https://aka.ms/dsc/schemas/v3.1.3/bundled/config/document.json
               https://aka.ms/dsc/schemas/v3.1.3/bundled/config/document.vscode.json
               https://aka.ms/dsc/schemas/v3.1.2/config/document.json
               https://aka.ms/dsc/schemas/v3.1.2/bundled/config/document.json
               https://aka.ms/dsc/schemas/v3.1.2/bundled/config/document.vscode.json
               https://aka.ms/dsc/schemas/v3.1.1/config/document.json
               https://aka.ms/dsc/schemas/v3.1.1/bundled/config/document.json
               https://aka.ms/dsc/schemas/v3.1.1/bundled/config/document.vscode.json
               https://aka.ms/dsc/schemas/v3.1.0/config/document.json
               https://aka.ms/dsc/schemas/v3.1.0/bundled/config/document.json
               https://aka.ms/dsc/schemas/v3.1.0/bundled/config/document.vscode.json
               https://aka.ms/dsc/schemas/v3.0/config/document.json
               https://aka.ms/dsc/schemas/v3.0/bundled/config/document.json
               https://aka.ms/dsc/schemas/v3.0/bundled/config/document.vscode.json
               https://aka.ms/dsc/schemas/v3.0.2/config/document.json
               https://aka.ms/dsc/schemas/v3.0.2/bundled/config/document.json
               https://aka.ms/dsc/schemas/v3.0.2/bundled/config/document.vscode.json
               https://aka.ms/dsc/schemas/v3.0.1/config/document.json
               https://aka.ms/dsc/schemas/v3.0.1/bundled/config/document.json
               https://aka.ms/dsc/schemas/v3.0.1/bundled/config/document.vscode.json
               https://aka.ms/dsc/schemas/v3.0.0/config/document.json
               https://aka.ms/dsc/schemas/v3.0.0/bundled/config/document.json
               https://aka.ms/dsc/schemas/v3.0.0/bundled/config/document.vscode.json
             ]
```

### contentVersion

The `contentVersion` property defines a version string for the configuration document. You can use
this property to track revisions of the document. DSC doesn't validate or use this value when
processing a configuration document. The document that the `dsc config export` command returns
always defines this property as `1.0.0`.

```yaml
Type:     string
Required: false
```

### directives

The `directives` property defines how DSC processes the configuration document as a whole, like
the security context the document requires, the version of DSC it requires, and when DSC discovers
resources. DSC validates and acts on the directives before it invokes any resource instance. Every
directive is optional.

For more information about the available directives, see
[DSC Configuration document directives schema][03].

```yaml
Type:     object
Required: false
```

### executionInformation

The `executionInformation` property describes the DSC operation that produced a configuration
document. DSC adds this property to the document returned by the `dsc config export` command. The
schema accepts this property in any configuration document, but DSC ignores it when it processes
the document.

The value is an object with the same properties as the execution information that DSC returns in
command output. For more information, see [DSC execution information schema reference][04].

```yaml
Type:     object
Required: false
```

### functions

The `functions` property defines user-defined functions that you can call in configuration
expressions anywhere in the document. Each item in the list defines a namespace and the functions
that belong to it. You call a user-defined function as `<namespace>.<name>()`.

For more information about defining user-defined functions, see
[DSC Configuration document user-defined function schema][05].

```yaml
Type:      array
Required:  false
ItemsType: object
```

### metadata

The `metadata` property defines a set of key-value pairs as annotations for the configuration.
Except for the `Microsoft.DSC` property, DSC doesn't validate the metadata. A configuration can
include any arbitrary information in this property.

The `Microsoft.DSC` property is reserved for DSC. For more information, see
[DSC Configuration document metadata schema][02].

```yaml
Type:     object
Required: false
```

### outputs

The `outputs` property defines values that DSC evaluates after it processes every resource instance
in the document and returns in the `outputs` property of the command output. Each output is defined
as a key-value pair. The key is the name of the output. The value is an object that defines the
data type of the output and the expression that DSC evaluates to get its value.

For more information about defining outputs, see [DSC Configuration document output schema][06].

```yaml
Type:     object
Required: false
```

### parameters

The `parameters` property defines a set of runtime options for the configuration. Each parameter is
defined as key-value pair. The key for each pair defines the name of the parameter. The value for
each pair must be an object that defines the `type` keyword to indicate how DSC should process the
parameter.

Parameters may be overridden at runtime, enabling re-use of the same configuration document for
different contexts.

For more information about defining parameters in a configuration, see
[DSC Configuration document parameter schema][07].

<!-- For more information about using parameters in a configuration, see
[DSC Configuration parameters][08] -->

```yaml
Type:                object
Required:            false
ValidPropertySchema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/config/document.parameter.json
```

### resources

The `resources` property defines a list of DSC Resource instances that the configuration manages.
Instances may share the same DSC Resource type, but every instance must have a unique combination
of `type` and `name`. If two instances share the same type and name, DSC raises an error.

For more information about defining a valid resource instance in a configuration, see
[DSC Configuration document resource schema][09].

<!-- For more information about how DSC uses resources in a configuration, see
[DSC Configuration resources][10] and [DSC Configuration resource groups][11]. -->

```yaml
Type:            array
Required:        true
ValidItemSchema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/config/document.resource.json
```

### variables

The `variables` property defines a set of reusable values for the resources in the document as
key-value pairs. The key for each pair defines the name of the variable. Resources that reference
the variable by name can access the variable's value.

This can help reduce the amount of copied values and options for resources in the configuration,
which makes the document easier to read and maintain. Unlike parameters, variables can only be
defined in the configuration and can't be overridden at runtime.

<!-- For more information about using variables in a configuration, see
[DSC Configuration variables][12]. -->

```yaml
Type:     object
Required: false
```

<!-- Link reference definitions -->
[01]: functions/overview.md
<!-- [01]: ../../../configurations/overview.md -->
[02]: metadata.md
[03]: directives.md
[04]: executionInformation.md
[05]: function.md
[06]: output.md
[07]: parameter.md
<!-- [08]: ../../../configurations/parameters.md -->
[09]: resource.md
<!-- [10]: ../../../configurations/resources.md -->
<!-- [11]: ../../../configurations/resource-groups.md -->
<!-- [12]: ../../../configurations/variables.md -->
