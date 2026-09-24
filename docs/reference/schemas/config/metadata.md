---
description: JSON schema reference for metadata in a Desired State Configuration document.
ms.date:     09/01/2026
ms.topic:    reference
title:       DSC Configuration document metadata schema
---

# DSC Configuration document metadata schema

## Synopsis

Defines a set of informational key-value pairs for the configuration.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/config/document.metadata.json
Type:          object
```

## Description

Defines a set of informational key-value pairs for the configuration. Except for the
`Microsoft.DSC` property, this metadata isn't validated. You can pass any data into your
configuration as a property of `metadata`.

For example, you could define information about the configuration used by your teams or internal
tools:

```yaml
$schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/bundled/config/document.vscode.json

metadata:
  owner: security.ops@contoso.com
  name:  WebAppBaseline
  purpose: |-
    Define a baseline for securing web application servers.
```

The same schema applies to the `metadata` property of a resource instance in the configuration
document. For more information, see [DSC Configuration document resource instance schema][01].

## Microsoft.DSC

The `Microsoft.DSC` metadata property contains directives and information that DSC itself uses when
processing a configuration document. Unlike other metadata key-value pairs, DSC validates these
properties. This property is reserved and shouldn't contain any custom user-defined metadata.

The schema for this property also accepts the execution information properties that DSC returns in
command output, like `operation` and `version`. DSC doesn't use those properties when it processes
a configuration document. For the full list of properties, see
[Microsoft.DSC metadata property schema reference][02].

### Properties

#### securityContext

This property defines the security context a configuration requires. If you invoke a DSC operation
against the configuration document in a security context that conflicts with this metadata, DSC
raises an error when it validates the configuration document.

> [!NOTE]
> Defining the required security context in metadata is deprecated. DSC raises a warning when a
> configuration document defines this property. Use the `securityContext` directive in the
> document's [directives][03] property instead. If you define both, the values must match or DSC
> raises an error.

The valid security contexts are:

- `current`

  Indicates that the configuration document is usable under any security context. You can invoke
  DSC operations against the document when elevated as root or an administrator and as a normal
  user or account.
- `elevated`

  Indicates that the configuration document is usable only in an elevated security context. You can
  invoke DSC operations against the document when elevated as root or an administrator. When you
  invoke DSC operations against the document as a non-elevated user or account, DSC raises an error
  when it validates the configuration document.
- `restricted`

  Indicates that the configuration document is usable only in a non-elevated security context. You
  can invoke DSC operations against the document as a non-elevated user or account. When you invoke
  DSC operations against the document as root or an administrator, DSC raises an error when it
  validates the configuration document.

The default security context is `current`.

```yaml
Type:        string
Required:    false
Default:     current
ValidValues: [current, elevated, restricted]
```

<!-- Link reference definitions -->
[01]: resource.md#metadata-1
[02]: ../metadata/Microsoft.DSC/properties.md
[03]: document.md#securitycontext
