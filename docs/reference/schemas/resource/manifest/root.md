---
description: JSON schema reference for a DSC Resource manifest
ms.date:     01/17/2024
ms.topic:    reference
title:       Command-based DSC Resource manifest schema reference
---

# Command-based DSC Resource manifest schema reference

## Synopsis

The data file that defines a command-based DSC Resource.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/resource/manifest.json
Type:          object
```

## Description

Every command-based DSC Resource must have a manifest. The manifest file must:

1. Be discoverable in the `PATH` environment variable.
1. Be formatted as either JSON or YAML.
1. Follow the naming convention `<name>.dsc.resource.<extension>`. Valid extensions include `json`,
   `yml`, and `yaml`.
1. Be valid for the schema described in this document.

The rest of this document describes the manifest's schema.

## Required properties

The manifest must include these properties:

- [$schema](#schema)
- [type](#type)
- [version](#version)
- [get](#get)

## Properties

### $schema

The `$schema` property indicates the canonical URI of this schema that the manifest validates
against. This property is mandatory. DSC uses this value to validate the manifest against the
correct JSON schema.

For every version of the schema, there are three valid urls:

- `.../resource/manifest.json`

  The URL to the canonical non-bundled schema. When it's used for validation, the validating client
  needs to retrieve this schema and every schema it references.

- `.../bundled/resource/manifest.json`

  The URL to the bundled schema. When it's used for validation, the validating client only needs to
  retrieve this schema.

  This schema uses the bundling model introduced for JSON Schema 2020-12. While DSC can still
  validate the document when it uses this schema, other tools may error or behave in unexpected
  ways.

- `.../bundled/resource/manifest.vscode.json`

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
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/resource/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/bundled/resource/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/bundled/resource/manifest.vscode.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/10/resource/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/10/bundled/resource/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/10/bundled/resource/manifest.vscode.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/08/resource/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/08/bundled/resource/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/08/bundled/resource/manifest.vscode.json
             ]
```

### type

The `type` property represents the fully qualified type name of the resource. It's used to specify
the resource in configuration documents and as the value of the `--resource` flag when using the
`dsc resource *` commands. For more information about resource type names, see
[DSC Resource fully qualified type name schema reference][01].

```yaml
Type:     string
Required: true
Pattern:  ^\w+(\.\w+){0,2}\/\w+$
```

### version

The `version` property must be the current version of the resource as a valid semantic version
(semver) string. The version applies to the resource, not the software it manages.

```yaml
Type:     string
Required: true
Pattern:  ^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-((?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+([0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?$
```

### description

The `description` property defines a synopsis for the resource's purpose. The value for this
property must be a short string.

```yaml
Type:     string
Required: false
```

### kind

The `kind` property defines how DSC should handle the resource. DSC supports three kinds of
command-based DSC Resources: `Resource`, `Group`, and `Adapter`.

When `kind` isn't defined in the resource manifest, DSC infers the value for the property. If the
`adapter` property is defined in the resource manifest, DSC infers the value of `kind` as
`Adapter`. If the `adapter` property isn't defined, DSC infers the value of `kind` as `Resource`.
DSC can't infer whether a manifest is for a group resource.

When defining a group resource, always explicitly define the `kind` property in the manifest as
`Group`.

For more information, see [DSC Resource kind schema reference][02].

```yaml
Type:        string
Required:    false
ValidValues: [Resource, Adapter, Group]
```

### tags

The `tags` property defines a list of searchable terms for the resource. The value of this
property must be an array of strings. Each tag must contain only alphanumeric characters and
underscores. No other characters are permitted. Each tag must be unique.

```yaml
Type:              array
Required:          false
ItemsMustBeUnique: true
ItemsType:         string
ItemsPattern:      ^\w+$
```

### export

The `export` property defines how to call the resource to get the current state of every instance.
When this property is defined, users can:

- Specify an instance of the resource in the input configuration for the [dsc config export][03]
  command to generate an usable configuration document.
- Specify the resource with the [dsc resource export][04] command to generate a configuration
  document that defines every instance of the resource.
- Specify the resource with the [dsc resource get][05] command and the [--all][06] option to return
  the current state for every instance of the resource.

The value of this property must be an object. The object's `executable` property, defining the name
of the command to call, is mandatory. The `args` property is optional. For more
information, see [DSC Resource manifest export property schema reference][07].

```yaml
Type:     object
Required: true
```

### get

The `get` property defines how to call the resource to get the current state of an instance. This
property is mandatory for all resources.

The value of this property must be an object. The object's `executable` property, defining the name
of the command to call, is mandatory. The `args` and `input` properties are optional. For more
information, see [DSC Resource manifest get property schema reference][08].

```yaml
Type:     object
Required: true
```

### set

The `set` property defines how to call the resource to set the desired state of an instance. It
also defines how to process the output from the resource for this method. When this property isn't
defined, the DSC can't manage instances of the resource. It can only get their current state and
test whether the instance is in the desired state.

The value of this property must be an object. The `executable` property, defining the name of the
command to call, is mandatory. The `args` `input`, `implementsPretest`, and `returns` properties
are optional. For more information, see [DSC Resource manifest set property schema reference][09].

```yaml
Type:     object
Required: false
```

### test

The `test` property defines how to call the resource to test whether an instance is in the desired
state. It also defines how to process the output from the resource for this method. When this
property isn't defined, DSC performs a basic synthetic test for instances of the DSC Resource.

The value of this property must be an object. The object's `executable` property, defining the name
of the command to call, is mandatory. The `args` `input`, and `returns` properties are optional.
For more information, see [DSC Resource manifest test property schema reference][10].

```yaml
Type:     object
Required: false
```

### validate

The `validate` property defines how to call a DSC Group Resource to validate its instances. This
property is mandatory for DSC Group Resources. DSC ignores this property for all other resources.

The value of this property must be an object. The object's `executable` property, defining the name
of the command to call, is mandatory. The `args` property is optional. For more information, see
[DSC Resource manifest validate property schema reference][11].

```yaml
Type:     object
Required: false
```

### provider

When specified, the `provider` property defines the resource as a DSC Resource Provider.

The value of this property must be an object. The object's `list` and `config` properties are
mandatory. The `list` property defines how to call the provider to return the resources that the
provider can manage. The `config` property defines how the provider expects input. For more
information, see the [DSC Resource manifest provider property schema reference][12].

### exitCodes

The `exitCodes` property defines a set of valid exit codes for the resource and their meaning.
Define this property as a set of key-value pairs where:

- The key is a string containing a signed integer that maps to a known exit code for the resource.
  The exit code must be a literal signed integer. You can't use alternate formats for the exit
  code. For example, instead of the hexadecimal value `0x80070005` for "Access denied", specify the
  exit code as `-2147024891`.
- The value is a string describing the semantic meaning of that exit code for a human reader.

DSC interprets exit code `0` as a successful operation and any other exit code as an error.

> [!TIP]
> If you're authoring your resource manifest in yaml, be sure to wrap the exit code in single
> quotes to ensure the YAML file can be parsed correctly. For example:
>
> ```yaml
> exitCodes:
>   '0': Success
>   '1': Invalid parameter
>   '2': Invalid input
>   '3': Registry error
>   '4': JSON serialization failed
> ```

```yaml
Type:                object
Required:            false
PropertyNamePattern: ^[0-9]+#
PropertyValueType:   string
```

### schema

The `schema` property defines how to get the JSON schema that validates an instance of the
resource. This property must always be an object that defines one of the following properties:

- `command` - When you specify the `command` property, DSC calls the defined command to get the
  JSON schema.
- `embedded` - When you specify the `embedded` property, DSC uses the defined value as the JSON
  schema.

For more information, see [DSC Resource manifest schema property reference][13].

```yaml
Type:     object
Required: true
```

[01]: ../../definitions/resourceType.md
[02]: ../../definitions/resourceKind.md
[03]: ../../../cli/config/export.md
[04]: ../../../cli/resource/export.md
[05]: ../../../cli/resource/get.md
[06]: ../../../cli/resource/get.md#-a---all
[07]: export.md
[08]: get.md
[09]: set.md
[10]: test.md
[11]: validate.md
[12]: provider.md
[13]: schema/property.md
