# DSC Configuration document schema reference

## Synopsis

The YAML or JSON file that defines a DSC Configuration.

## Metadata

```yaml
Schema Dialect : https://json-schema.org/draft/2020-12/schema
Schema ID      : https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/08/config/document.json
Type           : object
```

## Description

DSC Configurations enable users to define state by combining different DSC Resources. A
configuration document uses parameters and variables to pass to a set of one or more resources that
define a desired state.

A configuration document can be defined as either YAML or JSON. For ease of authoring, Microsoft
recommneds drafting configuration documents in YAML.

For DSC's authoring tools to recognize a file as a DSC Configuration document, the filename must
end with `.dsc.config.json` or `.dsc.config.yaml`.

For more information, see [DSC Configurations overview][01]

The rest of this document describes the schema DSC uses to validation configuration documents.

## Examples

<!-- To-Do -->

## Required Properties

Every configuration document must include these properties:

- [$schema]
- [resources]

## Properties

### $schema

The `$schema` property indicates the canonical URI for the version of this schema that the document
adheres to. DSC uses this property when validating the configuration document before any
configuration operations.

```yaml
Type:     string
Required: true
Format:   URI
Valid Values:
  - https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/08/config/document.json
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
[DSC Configuration document parameter schema][02]. For more information about using parameters in a
configuration, see [DSC Configuration parameters][03]

```yaml
Type:                  object
Required:              false
Valid Property Schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/08/config/document.parameter.json
```

### variables

The `variables` property defines a set of reusable values for the resources in the document as
key-value pairs. The key for each pair defines the name of the variable. Resources that reference
the variable by name can access the variable's value.

This can help reduce the amount of copied values and options for resources in the configuration,
which makes the document easier to read and maintain. Unlike parameters, variables can only be
defined in the configuration and can't be overridden at run-time.

For more information about using variables in a configuration, see
[DSC Configuration variables][04].

```yaml
Type:     object
Required: false
```

### resources

The `resources` property defines a list of DSC Resource instances that the configuration manages.
Every instance in the list must be unique, but instances may share the same DSC Resource type.

For more information about defining a valid resource instance in a configuration, see
[DSC Configuration document resource schema][05]. For more information about how DSC uses resources
in a configuration, see [DSC Configuration resources][06] and
[DSC Configuration resource groups][07].

```yaml
Type:               array
Required:           false
Minimum Item Count: 1
Valid Item Schema:  https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/08/config/document.resource.json
```

[01]: ../../../configurations/overview.md
[02]: parameter.md
[03]: ../../../configurations/parameters.md
[04]: ../../../configurations/variables.md
[05]: resource.md
[06]: ../../../configurations/resources.md
[07]: ../../../configurations/resource-groups.md
