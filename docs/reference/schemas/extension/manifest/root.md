---
description: JSON schema reference for a DSC extension manifest
ms.date:     09/01/2026
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

Every command-based DSC extension must have a manifest. The manifest file must:

1. Be discoverable in the `PATH` environment variable.
1. Be formatted as either JSON or YAML.
1. Follow the naming convention `<name>.dsc.extension.<extension>`. Valid extensions include
   `json`, `yml`, and `yaml`.
1. Be valid for the schema described in this document.

DSC infers the capabilities of an extension from the operation properties defined in the manifest.
An extension has the `discover` capability when the manifest defines the [discover](#discover)
property, the `import` capability when the manifest defines the [import](#import) property, and
the `secret` capability when the manifest defines the [secret](#secret) property. An extension
that doesn't define any of these properties has no capabilities.

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
of DSC. DSC recognizes the URIs for every version folder listed below, but the schemas aren't
published to every recognized folder. For more information about schema URIs and versioning, see
[DSC JSON Schema URIs][01].

For every version of the schema, there are three valid URLs:

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

  This schema uses keywords that are only recognized by Visual Studio Code. While DSC can still
  validate the document when it uses this schema, other tools may error or behave in unexpected
  ways.

```yaml
Type:        string
Required:    true
Format:      URI
ValidValues: [
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/extension/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/bundled/extension/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/bundled/extension/manifest.vscode.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2/extension/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2/bundled/extension/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2/bundled/extension/manifest.vscode.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.3/extension/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.3/bundled/extension/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.3/bundled/extension/manifest.vscode.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.2/extension/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.2/bundled/extension/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.2/bundled/extension/manifest.vscode.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.1/extension/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.1/bundled/extension/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.1/bundled/extension/manifest.vscode.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.0/extension/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.0/bundled/extension/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.0/bundled/extension/manifest.vscode.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1/extension/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1/bundled/extension/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1/bundled/extension/manifest.vscode.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.3/extension/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.3/bundled/extension/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.3/bundled/extension/manifest.vscode.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.2/extension/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.2/bundled/extension/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.2/bundled/extension/manifest.vscode.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.1/extension/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.1/bundled/extension/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.1/bundled/extension/manifest.vscode.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/extension/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/bundled/extension/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/bundled/extension/manifest.vscode.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/extension/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/bundled/extension/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/bundled/extension/manifest.vscode.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.2/extension/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.2/bundled/extension/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.2/bundled/extension/manifest.vscode.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.1/extension/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.1/bundled/extension/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.1/bundled/extension/manifest.vscode.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/extension/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/bundled/extension/manifest.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/bundled/extension/manifest.vscode.json
               https://aka.ms/dsc/schemas/v3/extension/manifest.json
               https://aka.ms/dsc/schemas/v3/bundled/extension/manifest.json
               https://aka.ms/dsc/schemas/v3/bundled/extension/manifest.vscode.json
               https://aka.ms/dsc/schemas/v3.2/extension/manifest.json
               https://aka.ms/dsc/schemas/v3.2/bundled/extension/manifest.json
               https://aka.ms/dsc/schemas/v3.2/bundled/extension/manifest.vscode.json
               https://aka.ms/dsc/schemas/v3.2.3/extension/manifest.json
               https://aka.ms/dsc/schemas/v3.2.3/bundled/extension/manifest.json
               https://aka.ms/dsc/schemas/v3.2.3/bundled/extension/manifest.vscode.json
               https://aka.ms/dsc/schemas/v3.2.2/extension/manifest.json
               https://aka.ms/dsc/schemas/v3.2.2/bundled/extension/manifest.json
               https://aka.ms/dsc/schemas/v3.2.2/bundled/extension/manifest.vscode.json
               https://aka.ms/dsc/schemas/v3.2.1/extension/manifest.json
               https://aka.ms/dsc/schemas/v3.2.1/bundled/extension/manifest.json
               https://aka.ms/dsc/schemas/v3.2.1/bundled/extension/manifest.vscode.json
               https://aka.ms/dsc/schemas/v3.2.0/extension/manifest.json
               https://aka.ms/dsc/schemas/v3.2.0/bundled/extension/manifest.json
               https://aka.ms/dsc/schemas/v3.2.0/bundled/extension/manifest.vscode.json
               https://aka.ms/dsc/schemas/v3.1/extension/manifest.json
               https://aka.ms/dsc/schemas/v3.1/bundled/extension/manifest.json
               https://aka.ms/dsc/schemas/v3.1/bundled/extension/manifest.vscode.json
               https://aka.ms/dsc/schemas/v3.1.3/extension/manifest.json
               https://aka.ms/dsc/schemas/v3.1.3/bundled/extension/manifest.json
               https://aka.ms/dsc/schemas/v3.1.3/bundled/extension/manifest.vscode.json
               https://aka.ms/dsc/schemas/v3.1.2/extension/manifest.json
               https://aka.ms/dsc/schemas/v3.1.2/bundled/extension/manifest.json
               https://aka.ms/dsc/schemas/v3.1.2/bundled/extension/manifest.vscode.json
               https://aka.ms/dsc/schemas/v3.1.1/extension/manifest.json
               https://aka.ms/dsc/schemas/v3.1.1/bundled/extension/manifest.json
               https://aka.ms/dsc/schemas/v3.1.1/bundled/extension/manifest.vscode.json
               https://aka.ms/dsc/schemas/v3.1.0/extension/manifest.json
               https://aka.ms/dsc/schemas/v3.1.0/bundled/extension/manifest.json
               https://aka.ms/dsc/schemas/v3.1.0/bundled/extension/manifest.vscode.json
               https://aka.ms/dsc/schemas/v3.0/extension/manifest.json
               https://aka.ms/dsc/schemas/v3.0/bundled/extension/manifest.json
               https://aka.ms/dsc/schemas/v3.0/bundled/extension/manifest.vscode.json
               https://aka.ms/dsc/schemas/v3.0.2/extension/manifest.json
               https://aka.ms/dsc/schemas/v3.0.2/bundled/extension/manifest.json
               https://aka.ms/dsc/schemas/v3.0.2/bundled/extension/manifest.vscode.json
               https://aka.ms/dsc/schemas/v3.0.1/extension/manifest.json
               https://aka.ms/dsc/schemas/v3.0.1/bundled/extension/manifest.json
               https://aka.ms/dsc/schemas/v3.0.1/bundled/extension/manifest.vscode.json
               https://aka.ms/dsc/schemas/v3.0.0/extension/manifest.json
               https://aka.ms/dsc/schemas/v3.0.0/bundled/extension/manifest.json
               https://aka.ms/dsc/schemas/v3.0.0/bundled/extension/manifest.vscode.json
             ]
```

### type

The `type` property represents the fully qualified type name of the extension. Extension type
names use the same syntax as resource type names: an owner segment, any number of namespace
segments, a forward slash (`/`), and a name segment. For more information about type names, see
[DSC Resource fully qualified type name schema reference][02].

```yaml
Type:     string
Required: true
Pattern:  ^\w+(\.\w+)*\/\w+$
```

### version

The `version` property must be the current version of the extension as a valid semantic version
(SemVer) string.

```yaml
Type:     string
Required: true
Pattern:  ^(?<major>(?:0|[1-9]\d*))\.(?<minor>(?:0|[1-9]\d*))\.(?<patch>(?:0|[1-9]\d*))(?:-(?<prerelease>(?:(?:0|[1-9]\d*)|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:(?:0|[1-9]\d*)|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+(?<buildmetadata>[0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?$
```

### condition

The `condition` property defines a DSC configuration expression that DSC evaluates during
discovery to determine whether the extension is usable on the system. If the expression evaluates
to `false`, DSC discards the extension during discovery and writes a debug message indicating that
the manifest's condition wasn't met. If the manifest doesn't define this property, or the
expression evaluates to `true`, DSC discovers the extension as normal.

Use this property for extensions with external prerequisites. For example, the
`Microsoft.PowerShell/Discover` extension defines the condition
`[not(equals(tryWhich('pwsh'), null()))]` so that DSC ignores the extension when `pwsh` isn't
available on the system.

```yaml
Type:     string
Required: false
```

### deprecationMessage

The `deprecationMessage` property indicates that the extension is deprecated. When this property
is defined, DSC raises a warning that includes the message whenever it invokes the extension. DSC
also reports the message in the output of the `dsc extension list` command.

```yaml
Type:     string
Required: false
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
defined, the extension has the `discover` capability and DSC invokes the `discover` operation for
the extension during the resource discovery phase of any `dsc config` or `dsc resource` command.

The value of this property must be an object. The object's `executable` property, defining the name
of the command to call, is mandatory. The `args` property is optional. For more information, see
[DSC extension manifest discover property schema reference][03].

```yaml
Type:     object
Required: false
```

### import

The `import` property defines how to call the extension to convert a file that DSC can't parse
directly into a configuration document. When this property is defined, the extension has the
`import` capability.

When you pass a file to a `dsc config` command with the `--file` option, DSC checks whether any
extension with the `import` capability handles the file's extension. DSC uses the content returned
by the first extension that successfully imports the file as the configuration document. If no
extension imports the file, DSC reads the file directly.

The value of this property must be an object with the following properties:

- `fileExtensions` (required) - An array of strings defining the file extensions the extension can
  import, like `["bicep"]`. Define the extensions without the leading period. If this array is
  empty, DSC writes a warning during discovery and the extension can't import any files.
- `executable` (required) - The name of the command to run. The value must be the name of a
  command discoverable in the system's `PATH` environment variable or the full path to the command.
- `args` (optional) - The list of arguments to pass to the command. Each item in the array can be
  a string representing a static argument, like `--format`, or an object with the `fileArg`
  property. For the `fileArg` item, DSC passes the value of `fileArg` followed by the absolute path
  to the file to import. If `fileArg` is an empty string, DSC passes only the path.
- `output` (optional) - A DSC configuration expression that DSC evaluates after the command
  completes to transform the command's output into the configuration document. Use the `stdout()`
  function in the expression to access the text the command wrote to stdout. When this property
  isn't defined, DSC uses the command's output as the configuration document without modification.

```yaml
Type:     object
Required: false
```

### importParameters

The `importParameters` property has the same structure as the [import](#import) property. DSC
reserves this property for a future operation that imports parameters from files in formats DSC
can't parse directly. DSC validates this property when it loads the manifest but doesn't currently
invoke the command it defines.

```yaml
Type:     object
Required: false
```

### secret

The `secret` property defines how to call the extension to retrieve a secret from a vault at
runtime. When this property is defined, the extension has the `secret` capability and DSC can
invoke the extension for the [secret()][04] configuration function.

The value of this property must be an object with the following properties:

- `executable` (required) - The name of the command to run. The value must be the name of a
  command discoverable in the system's `PATH` environment variable or the full path to the command.
- `args` (optional) - The list of arguments to pass to the command. Each item in the array can be
  a string representing a static argument, an object with the `nameArg` property, or an object
  with the `vaultArg` property. For the `nameArg` item, DSC passes the value of `nameArg` followed
  by the name of the secret to retrieve. For the `vaultArg` item, DSC passes the value of
  `vaultArg` followed by the name of the vault when the `secret()` function specifies a vault. When
  the function doesn't specify a vault, DSC omits the `vaultArg` item entirely.

The command must write the secret value to stdout as a single line. If the command writes more
than one line to stdout, DSC raises an error. If the command writes nothing to stdout, DSC treats
the secret as not found for that extension.

```yaml
Type:     object
Required: false
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
PropertyNamePattern: ^-?[0-9]+$
PropertyValueType:   string
```

### metadata

The `metadata` property defines an object of arbitrary additional data for the extension. DSC
doesn't validate or use the values in this object. Use this property to include any information
about the extension that isn't covered by the other manifest properties, like details for
integrating tools.

```yaml
Type:     object
Required: false
```

<!-- Link reference definitions -->
[01]: ../../schema-uris.md
[02]: ../../definitions/resourceType.md
[03]: discover.md
[04]: ../../config/functions/secret.md
