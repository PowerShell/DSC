---
description: JSON schema reference for a DSC Resource manifest
ms.date:     09/01/2026
ms.topic:    reference
title:       Command-based DSC Resource manifest schema reference
---

# Command-based DSC Resource manifest schema reference

## Synopsis

The data file that defines a command-based DSC Resource.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/resource/manifest.json
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

## Properties

### $schema

The `$schema` property indicates the canonical URI of this schema that the manifest validates
against. This property is mandatory. DSC uses this value to validate the manifest against the
correct JSON schema.

The JSON schemas for DSC are published in multiple versions and forms. This documentation is for
the latest version of the schema. As a convenience, you can specify either the full URI for the
schema hosted in GitHub or use the shorter `aka.ms` URI. You can specify the schema for a specific
semantic version, the latest schema for a minor version, or the latest schema for a major version
of DSC. For more information about schema URIs and versioning, see
[DSC JSON Schema URIs](../../schema-uris.md).

For every version of the schema, there are three valid URLs:

- `.../resource/manifest.json`

  The URL to the canonical non-bundled schema. When it's used for validation, the validating client
  needs to retrieve this schema and every schema it references.

- `.../bundled/resource/manifest.json`

  The URL to the canonically bundled schema. When it's used for validation, the validating client
  only needs to retrieve this schema.

  This schema uses the bundling model introduced for JSON Schema 2020-12. While DSC can still
  validate the document when it uses this schema, other tools may error or behave in unexpected
  ways if they don't fully support the 2020-12 specification.

- `.../bundled/resource/manifest.vscode.json`

  The URL to the enhanced authoring schema. This schema is much larger than the other schemas, as
  it includes additional definitions that provide contextual help and snippets that the others
  don't include.

  This schema uses keywords that are only recognized by Visual Studio Code. While DSC can still
  validate the document when it uses this schema, other tools may error or behave in unexpected
  ways.

```yaml
Type:        string
Required:    true
Format:      URI
ValidValues: [
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/resource/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/bundled/resource/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/bundled/resource/manifest.vscode.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2/resource/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2/bundled/resource/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2/bundled/resource/manifest.vscode.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.3/resource/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.3/bundled/resource/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.3/bundled/resource/manifest.vscode.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.2/resource/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.2/bundled/resource/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.2/bundled/resource/manifest.vscode.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.1/resource/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.1/bundled/resource/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.1/bundled/resource/manifest.vscode.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.0/resource/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.0/bundled/resource/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.0/bundled/resource/manifest.vscode.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1/resource/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1/bundled/resource/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1/bundled/resource/manifest.vscode.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.3/resource/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.3/bundled/resource/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.3/bundled/resource/manifest.vscode.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.2/resource/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.2/bundled/resource/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.2/bundled/resource/manifest.vscode.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.1/resource/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.1/bundled/resource/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.1/bundled/resource/manifest.vscode.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/resource/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/bundled/resource/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/bundled/resource/manifest.vscode.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/resource/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/bundled/resource/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/bundled/resource/manifest.vscode.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.2/resource/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.2/bundled/resource/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.2/bundled/resource/manifest.vscode.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.1/resource/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.1/bundled/resource/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.1/bundled/resource/manifest.vscode.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/resource/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/bundled/resource/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/bundled/resource/manifest.vscode.json
               https://aka.ms/dsc/schemas/v3/resource/manifest.json
               https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.json
               https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.vscode.json
               https://aka.ms/dsc/schemas/v3.2/resource/manifest.json
               https://aka.ms/dsc/schemas/v3.2/bundled/resource/manifest.json
               https://aka.ms/dsc/schemas/v3.2/bundled/resource/manifest.vscode.json
               https://aka.ms/dsc/schemas/v3.2.3/resource/manifest.json
               https://aka.ms/dsc/schemas/v3.2.3/bundled/resource/manifest.json
               https://aka.ms/dsc/schemas/v3.2.3/bundled/resource/manifest.vscode.json
               https://aka.ms/dsc/schemas/v3.2.2/resource/manifest.json
               https://aka.ms/dsc/schemas/v3.2.2/bundled/resource/manifest.json
               https://aka.ms/dsc/schemas/v3.2.2/bundled/resource/manifest.vscode.json
               https://aka.ms/dsc/schemas/v3.2.1/resource/manifest.json
               https://aka.ms/dsc/schemas/v3.2.1/bundled/resource/manifest.json
               https://aka.ms/dsc/schemas/v3.2.1/bundled/resource/manifest.vscode.json
               https://aka.ms/dsc/schemas/v3.2.0/resource/manifest.json
               https://aka.ms/dsc/schemas/v3.2.0/bundled/resource/manifest.json
               https://aka.ms/dsc/schemas/v3.2.0/bundled/resource/manifest.vscode.json
               https://aka.ms/dsc/schemas/v3.1/resource/manifest.json
               https://aka.ms/dsc/schemas/v3.1/bundled/resource/manifest.json
               https://aka.ms/dsc/schemas/v3.1/bundled/resource/manifest.vscode.json
               https://aka.ms/dsc/schemas/v3.1.3/resource/manifest.json
               https://aka.ms/dsc/schemas/v3.1.3/bundled/resource/manifest.json
               https://aka.ms/dsc/schemas/v3.1.3/bundled/resource/manifest.vscode.json
               https://aka.ms/dsc/schemas/v3.1.2/resource/manifest.json
               https://aka.ms/dsc/schemas/v3.1.2/bundled/resource/manifest.json
               https://aka.ms/dsc/schemas/v3.1.2/bundled/resource/manifest.vscode.json
               https://aka.ms/dsc/schemas/v3.1.1/resource/manifest.json
               https://aka.ms/dsc/schemas/v3.1.1/bundled/resource/manifest.json
               https://aka.ms/dsc/schemas/v3.1.1/bundled/resource/manifest.vscode.json
               https://aka.ms/dsc/schemas/v3.1.0/resource/manifest.json
               https://aka.ms/dsc/schemas/v3.1.0/bundled/resource/manifest.json
               https://aka.ms/dsc/schemas/v3.1.0/bundled/resource/manifest.vscode.json
               https://aka.ms/dsc/schemas/v3.0/resource/manifest.json
               https://aka.ms/dsc/schemas/v3.0/bundled/resource/manifest.json
               https://aka.ms/dsc/schemas/v3.0/bundled/resource/manifest.vscode.json
               https://aka.ms/dsc/schemas/v3.0.2/resource/manifest.json
               https://aka.ms/dsc/schemas/v3.0.2/bundled/resource/manifest.json
               https://aka.ms/dsc/schemas/v3.0.2/bundled/resource/manifest.vscode.json
               https://aka.ms/dsc/schemas/v3.0.1/resource/manifest.json
               https://aka.ms/dsc/schemas/v3.0.1/bundled/resource/manifest.json
               https://aka.ms/dsc/schemas/v3.0.1/bundled/resource/manifest.vscode.json
               https://aka.ms/dsc/schemas/v3.0.0/resource/manifest.json
               https://aka.ms/dsc/schemas/v3.0.0/bundled/resource/manifest.json
               https://aka.ms/dsc/schemas/v3.0.0/bundled/resource/manifest.vscode.json
             ]
```

### type

The `type` property represents the fully qualified type name of the resource. It's used to specify
the resource in configuration documents and as the value of the `--resource` flag when using the
`dsc resource *` commands. The type name must define an owner segment, any number of optional
namespace segments separated by periods (`.`), and a name segment separated from the preceding
segments by a forward slash (`/`), like `Microsoft.Windows/Registry`. Each segment must contain
only alphanumeric characters and underscores. For more information about resource type names, see
[DSC Resource fully qualified type name schema reference][01].

```yaml
Type:     string
Required: true
Pattern:  ^\w+(\.\w+)*\/\w+$
```

### condition

The `condition` property defines a DSC expression that DSC evaluates during resource discovery to
decide whether the manifest is active. The value must be an expression string that returns a
boolean value, like `[not(equals(tryWhich('pwsh'), null()))]`. When the expression returns
`false`, DSC skips the manifest and doesn't include the resource in discovery results. When the
expression returns any value other than a boolean, DSC raises an error.

Use this property to hide a resource when its dependencies aren't available on the system, like an
adapter that requires a specific shell or runtime. For more information about the available
functions, see [DSC configuration document functions reference][02] and the [tryWhich()][03]
function.

```yaml
Type:     string
Required: false
```

### deprecationMessage

The `deprecationMessage` property indicates that the resource is deprecated. When a manifest
defines this property, DSC emits the message as a warning whenever a user invokes an operation for
the resource and includes the message in the output of the `dsc resource list` command. Use this
property to direct users to a replacement resource, like
`Use the 'Microsoft.Adapter/PowerShell' adapter instead.`

```yaml
Type:     string
Required: false
```

### kind

The `kind` property defines how DSC should handle the resource. DSC supports several kinds of
resources: `resource`, `adapter`, `group`, `importer`, and `exporter`.

When `kind` isn't defined in the resource manifest, DSC infers the value for the property. If the
[adapter](#adapter) property is defined in the resource manifest, DSC infers the value of `kind`
as `adapter`. If the `adapter` property isn't defined, DSC infers the value of `kind` as
`resource`. DSC can't infer whether a manifest is for a `group`, `importer`, or `exporter`
resource.

When defining a group, importer, or exporter resource, always explicitly define the `kind` property
in the manifest.

For more information, see [DSC Resource kind schema reference][04].

```yaml
Type:        string
Required:    false
ValidValues: [adapter, exporter, group, importer, resource]
```

### version

The `version` property must be the current version of the resource as a valid semantic version
(SemVer) string. The version applies to the resource, not the software it manages.

For backward compatibility, DSC also accepts a date-based version in the format `YYYY-MM-DD` with
an optional prerelease suffix, like `2026-08-31-preview`. Date-based versions are deprecated. DSC
emits a warning when it discovers a manifest that defines a date-based version.

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

### get

The `get` property defines how to call the resource to get the current state of an instance.
Although the schema doesn't require this property, nearly every resource should define it. When a
manifest doesn't define `get`, the resource doesn't have the `get` capability and DSC can't
retrieve the current state of the resource's instances or synthesize results for the `test` and
`set` operations.

The value of this property must be an object. The object's `executable` property, defining the name
of the command to call, is mandatory. The `args`, `input`, and `requireSecurityContext` properties
are optional. For more information, see [DSC Resource manifest get property schema reference][05].

```yaml
Type:     object
Required: false
```

### set

The `set` property defines how to call the resource to set the desired state of an instance. It
also defines how to process the output from the resource for this method. When this property isn't
defined, the DSC can't manage instances of the resource. It can only get their current state and
test whether the instance is in the desired state.

The value of this property must be an object. The `executable` property, defining the name of the
command to call, is mandatory. The `args`, `input`, `implementsPretest`, `handlesExist`, `return`,
`requireSecurityContext`, and `whatIfReturns` properties are optional. For more information, see
[DSC Resource manifest set property schema reference][06].

```yaml
Type:     object
Required: false
```

### whatIf

The `whatIf` property defines how to call the resource to indicate whether and how the `set`
operation would modify an instance without changing the system. This property uses the same schema
as the [set](#set) property.

Defining a separate `whatIf` command is deprecated. Instead, define a [what-if argument][07] in the
`args` array of the `set` property. When the `set` definition includes a what-if argument, DSC
ignores the `whatIf` property. When the `set` definition doesn't include a what-if argument and the
manifest defines `whatIf`, DSC calls the `whatIf` command in what-if mode and emits a warning. When
the manifest defines neither, DSC synthesizes the what-if result by converting the result of the
`test` operation for the resource into a set result.

The value of this property must be an object. The `executable` property, defining the name of the
command to call, is mandatory. The `args`, `input`, `implementsPretest`, `handlesExist`, `return`,
`requireSecurityContext`, and `whatIfReturns` properties are optional. For more information, see
[DSC Resource manifest whatIf property schema reference][08].

```yaml
Type:     object
Required: false
```

### test

The `test` property defines how to call the resource to test whether an instance is in the desired
state. It also defines how to process the output from the resource for this method. When this
property isn't defined, DSC performs a basic synthetic test for instances of the DSC Resource.

The value of this property must be an object. The object's `executable` property, defining the name
of the command to call, is mandatory. The `args`, `input`, `return`, and `requireSecurityContext`
properties are optional. For more information, see
[DSC Resource manifest test property schema reference][09].

```yaml
Type:     object
Required: false
```

### delete

The `delete` property defines how to call the resource to remove an instance. When this property is
defined, the resource has the `delete` capability. Define this property as an alternative to
handling the [_exist][10] property in the `set` operation. If the resource's `set` command handles
removing an instance when `_exist` is `false`, define the `handlesExist` property of the `set`
method as `true` instead.

The value of this property must be an object. The object's `executable` property, defining the name
of the command to call, is mandatory. The `args`, `input`, and `requireSecurityContext` properties
are optional. For more information, see
[DSC Resource manifest delete property schema reference][11].

```yaml
Type:     object
Required: false
```

### export

The `export` property defines how to call the resource to get the current state of every instance.
When this property is defined, the resource has the `export` capability and users can:

- Specify an instance of the resource in the input configuration for the [dsc config export][12]
  command to generate an usable configuration document.
- Specify the resource with the [dsc resource export][13] command to generate a configuration
  document that defines every instance of the resource.
- Specify the resource with the [dsc resource get][14] command and the [--all][15] option to return
  the current state for every instance of the resource.

The value of this property must be an object. The object's `executable` property, defining the name
of the command to call, is mandatory. The `args`, `input`, `requireSecurityContext`, `schema`, and
`supportsFiltering` properties are optional. For more information, see
[DSC Resource manifest export property schema reference][16].

```yaml
Type:     object
Required: false
```

### resolve

The `resolve` property defines how to call an importer resource to resolve an external source into
a nested configuration document. When this property is defined, the resource has the `resolve`
capability. Define this property for [importer resources][04] and set the `kind` property to
`importer`.

The value of this property must be an object. The object's `executable` property, defining the name
of the command to call, is mandatory. The `args` and `input` properties are optional. For more
information, see [DSC Resource manifest resolve property schema reference][17].

```yaml
Type:     object
Required: false
```

### validate

The `validate` property defines how to call the resource to validate the JSON for an instance.
When a manifest defines this property, DSC calls the command to validate instance JSON instead of
validating the JSON against the resource's instance schema. Group resources, importer resources,
and resource adapters process nested resource instances that don't share a single instance schema.
Always define this property for those resources.

The value of this property must be an object. The object's `executable` property, defining the name
of the command to call, is mandatory. The `args` and `input` properties are optional. For more
information, see [DSC Resource manifest validate property schema reference][18].

```yaml
Type:     object
Required: false
```

### adapter

When specified, the `adapter` property defines the resource as a DSC Resource Adapter. When the
manifest doesn't define the `kind` property, DSC infers the kind as `adapter`.

The value of this property must be an object. The object's `inputKind` property is mandatory and
defines how the adapter expects to receive input. The `list` property is optional and defines how to
call the adapter to return the resources that the adapter can manage. For more information, see the
[DSC Resource manifest adapter property schema reference][19].

```yaml
Type:     object
Required: false
```

### exitCodes

The `exitCodes` property defines a set of valid exit codes for the resource and their meaning.
Define this property as a set of key-value pairs where:

- The key is a string containing a signed integer that maps to a known exit code for the resource.
  The exit code must be a literal signed integer. You can't use alternate formats for the exit
  code. For example, instead of the hexadecimal value `0x80070005` for "Access denied", specify the
  exit code as `-2147024891`.
- The value is a string describing the semantic meaning of that exit code for a human reader.

DSC interprets exit code `0` as a successful operation and any other exit code as an error. When
the manifest doesn't define this property, DSC describes exit code `0` as `Success` and every other
exit code as `Error` in its messages.

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
PropertyNamePattern: ^-?[0-9]+$
PropertyValueType:   string
```

### schema

The `schema` property defines how to get the JSON schema that validates an instance of the
resource. This property must always be an object that defines one of the following properties:

- `command` - When you specify the `command` property, DSC calls the defined command to get the
  JSON schema.
- `embedded` - When you specify the `embedded` property, DSC uses the defined value as the JSON
  schema.

DSC uses the schema to validate the input for an operation and, for resources with the `resource`
kind, the output the resource returns. When a manifest defines the [validate](#validate) property,
DSC calls that command instead of validating against the schema. When a manifest defines neither
`schema` nor `validate`, DSC raises an error when it needs to validate instance JSON for the
resource.

For more information, see [DSC Resource manifest schema property reference][20].

```yaml
Type:     object
Required: false
```

### metadata

The `metadata` property defines an arbitrary set of key-value pairs for the resource. DSC doesn't
validate or process the values in this object. Resource authors and integrating tools can use this
property to store additional information about the resource, like links to documentation or the
source repository for the resource.

```yaml
Type:     object
Required: false
```

<!-- Link reference definitions -->
[01]: ../../definitions/resourceType.md
[02]: ../../config/functions/overview.md
[03]: ../../config/functions/tryWhich.md
[04]: ../../definitions/resourceKind.md
[05]: get.md
[06]: set.md
[07]: set.md#what-if-argument
[08]: whatif.md
[09]: test.md
[10]: ../properties/exist.md
[11]: delete.md
[12]: ../../../cli/config/export.md
[13]: ../../../cli/resource/export.md
[14]: ../../../cli/resource/get.md
[15]: ../../../cli/resource/get.md#-a---all
[16]: export.md
[17]: resolve.md
[18]: validate.md
[19]: adapter.md
[20]: schema/property.md
