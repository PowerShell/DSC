---
description: JSON schema reference for a DSC extension manifest
ms.date:     02/28/2025
ms.topic:    reference
title:       Command-based DSC extension manifest schema reference
---

# Command-based DSC extension manifest schema reference

## Synopsis

The data file that defines a command-based DSC extension.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/extension/manifest.json
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

The JSON schemas for DSC are published in multiple versions and forms. This documentation is for
the latest version of the schema. As a convenience, you can specify either the full URI for the
schema hosted in GitHub or use the shorter `aka.ms` URI. You can specify the schema for a specific
semantic version, the latest schema for a minor version, or the latest schema for a major version
of DSC. For more information about schema URIs and versioning, see
[DSC JSON Schema URIs](../../schema-uris.md).

For every version of the schema, there are three valid urls:

- `.../extension/manifest.json`

  The URL to the canonical non-bundled schema. When it's used for validation, the validating client
  needs to retrieve this schema and every schema it references.

- `.../bundled/extension/manifest.json`

  The URL to the canonically bundled schema. When it's used for validation, the validating client
  only needs to retrieve this schema.

  This schema uses the bundling model introduced for JSON Schema 2020-12. While DSC can still
  validate the document when it uses this schema, other tools may error or behave in unexpected
  ways if they don't fully support the 2020-12 specification.

- `.../bundled/extension/manifest.vscode.json`

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
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/extension/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/bundled/extension/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/bundled/extension/manifest.vscode.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1/extension/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1/bundled/extension/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1/bundled/extension/manifest.vscode.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/extension/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/bundled/extension/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/bundled/extension/manifest.vscode.json
               https://aka.ms/dsc/schemas/v3/extension/manifest.json
               https://aka.ms/dsc/schemas/v3/bundled/extension/manifest.json
               https://aka.ms/dsc/schemas/v3/bundled/extension/manifest.vscode.json
               https://aka.ms/dsc/schemas/v3.1/extension/manifest.json
               https://aka.ms/dsc/schemas/v3.1/bundled/extension/manifest.json
               https://aka.ms/dsc/schemas/v3.1/bundled/extension/manifest.vscode.json
               https://aka.ms/dsc/schemas/v3.1.0/extension/manifest.json
               https://aka.ms/dsc/schemas/v3.1.0/bundled/extension/manifest.json
               https://aka.ms/dsc/schemas/v3.1.0/bundled/extension/manifest.vscode.json
             ]
```

### type

The `type` property represents the fully qualified type name of the extension. For more information
about extension type names, see [DSC extension fully qualified type name schema reference][01].

```yaml
Type:     string
Required: true
Pattern:  ^\w+(\.\w+){0,3}\/\w+$
```

### version

The `version` property must be the current version of the extension as a valid semantic version
(semver) string.

```yaml
Type:     string
Required: true
Pattern:  ^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-((?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+([0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?$
```

### description

The `description` property defines a synopsis for the extension's purpose. The value for this
property must be a short string.

```yaml
Type:     string
Required: false
```

### tags

The `tags` property defines a list of searchable terms for the extension. The value of this
property must be an array of strings. Each tag must contain only alphanumeric characters and
underscores. No other characters are permitted. Each tag must be unique.

```yaml
Type:              array
Required:          false
ItemsMustBeUnique: true
ItemsType:         string
ItemsPattern:      ^\w+$
```

### discover

The `discover` property defines how to call the extension to discover DSC resources that aren't
available in the `PATH` or `DSC_RESOURCE_PATH` environment variables. When this property is
defined, DSC invokes the `discover` operation for the extension during the resource discovery phase
of any `dsc config` or `dsc resource` command.

The value of this property must be an object. The object's `executable` property, defining the name
of the command to call, is mandatory. The `args` property is optional. For more
information, see [DSC extension manifest discover property schema reference][02].

```yaml
Type:     object
Required: true
```

### exitCodes

The `exitCodes` property defines a set of valid exit codes for the extension and their meaning.
Define this property as a set of key-value pairs where:

- The key is a string containing a signed integer that maps to a known exit code for the extension.
  The exit code must be a literal signed integer. You can't use alternate formats for the exit
  code. For example, instead of the hexadecimal value `0x80070005` for "Access denied", specify the
  exit code as `-2147024891`.
- The value is a string describing the semantic meaning of that exit code for a human reader.

DSC interprets exit code `0` as a successful operation and any other exit code as an error.

> [!TIP]
> If you're authoring your extension manifest in yaml, be sure to wrap the exit code in single
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
PropertyNamePattern: ^-?[0-9]+#
PropertyValueType:   string
```

[01]: ../../definitions/extensionType.md
[02]: discover.md
