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
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.0/extension/manifest.json
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

## JSON-RPC extension operations

The `discover` and `secret` properties can independently opt into JSON-RPC instead of a command.
Other extension operations keep their command interfaces. Existing command definitions don't
change. Validate JSON-RPC definitions against the full extension manifest schema; the standalone
`manifest.discover.json` and `manifest.secret.json` schemas describe the command forms.

```json
{
  "discover": {
    "jsonrpc": {
      "executable": "example-discover",
      "args": ["rpc"],
      "initializeTimeoutMS": 5000,
      "requestTimeoutMS": 15000,
      "shutdownTimeoutMS": 5000
    }
  }
}
```

The `jsonrpc` object requires a nonempty `executable` and accepts an optional array of static
string `args`. Don't mix `jsonrpc` with command fields such as `executable`, `args`, or `input`
at the operation level. Initialize and shutdown timeouts must be positive and default to 250
milliseconds. The request timeout defaults to zero, meaning no timeout. Explicit DSC
`resourceTimeouts` settings override these manifest defaults.

DSC starts a private process for each extension operation with the extension manifest's directory
as its working directory. The connection uses UTF-8 JSON-RPC 2.0 over stdin and stdout, one complete
JSON object per line, flushed after each write. Stdout must contain protocol messages only.
The lifecycle is `initialize`, `dsc_invoke`, `shutdown`, and the `exit` notification.
Select `protocolVersion: 1` in the initialize result. Return `null` for shutdown and exit cleanly.
Malformed messages, initialization failures, or timeouts terminate the process without retrying.
Extension processes aren't pooled.

The initialize params include `dscVersion`, `protocolVersions: [1]`, `traceLevel`, `traceFormat`,
`securityContext`, optional `configRoot`, and `hostServices`. Discover extensions and JSON-RPC
resources receive `hostServices: ["secret.get"]`. Secret extensions receive an empty array to
prevent recursive secret resolution.

### Reverse secret requests

A Discover extension can read its own settings after `dsc_invoke`, then optionally request one or
more secrets. No secret names or vaults need to be declared in the manifest. Check `hostServices`
before using the callback:

```json
{"jsonrpc":"2.0","id":"secret-1","method":"secret.get","params":{"name":"api-token","vault":"production"}}
```

While its invocation is pending, DSC resolves the request against already-loaded Secret
extensions. Both command and JSON-RPC providers are supported. DSC doesn't recursively run
discovery to find a provider. Multiple providers returning equal values are accepted; different
values are a conflict. A provider failure doesn't prevent another provider from returning a
secret.

The callback result is exactly one secure wrapper:

```json
{"jsonrpc":"2.0","id":"secret-1","result":{"secureString":"example-token"}}
```

For an object secret, return `{"secureObject": {...}}` instead. The `name` param must be a nonempty
string, `vault` is optional, and additional params aren't accepted. Callback IDs can be strings or
integers, but not null. Each direction has an independent ID namespace. The extension must keep
reading callback responses while its original invocation is pending. It can send multiple requests,
but DSC resolves them sequentially; match each response by its ID.

| Error code | Meaning |
| :--------- | :------ |
| `-32601` | Unknown method, or callbacks aren't available for this operation or lifecycle phase. |
| `-32602` | Invalid `secret.get` params. |
| `-32001` | No usable secret, including absent providers, missing secrets, or conflicting values. |

Callbacks don't extend the original invocation deadline. Provider initialization, retrieval,
and cleanup are bounded by the remaining deadline, including command-based providers. A timeout
terminates the operation's process rather than retrying or resetting its deadline.

DSC doesn't log callback values and uses generic callback/provider diagnostics instead of
forwarding provider-controlled error text. Secrets cross private process pipes in plaintext, so
extensions remain responsible for not logging or embedding them in discovered manifests, trace
notifications, progress messages, or other output. This callback isn't an authorization boundary
between trusted extensions.

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
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.0/extension/manifest.json
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
Pattern:  ^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-((?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+([0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?$
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

The value of this property must be an object. For the command form, the `executable` property,
defining the name of the command to call, is mandatory and `args` is optional. Alternatively,
define `jsonrpc` as described in [JSON-RPC extension operations](#json-rpc-extension-operations).
For more information, see
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

The value of this property must be an object. The object's `executable` and `fileExtensions`
properties are mandatory. The `args` and `output` properties are optional. For more information,
see [DSC extension manifest import property schema reference][04].

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
invoke the extension for the [secret()][05] configuration function.

The value of this property must be an object. For the command form, the `executable` property,
defining the name of the command to call, and `args`, defining the command's arguments, are both
mandatory. The `args` property must define the secret name input argument exactly once and may
define the vault input argument at most once. DSC doesn't load a command-form `secret` operation
without a secret name input argument. Alternatively, define `jsonrpc` as described in
[JSON-RPC extension operations](#json-rpc-extension-operations); names and vaults are then passed
as request params instead of command arguments. For more information, see
[DSC extension manifest secret property schema reference][06].

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
[04]: import.md
[05]: ../../config/functions/secret.md
[06]: secret.md
