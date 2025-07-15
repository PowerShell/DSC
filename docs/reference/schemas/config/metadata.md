---
description: JSON schema reference for metadata in a Desired State Configuration document.
ms.date:     07/03/2025
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

## Microsoft.DSC

The `Microsoft.DSC` metadata property contains directives and information that DSC itself uses when
processing a configuration document. Unlike other metadata key-value pairs, DSC validates these
properties. This property is reserved and shouldn't contain any custom user-defined metadata.

### Properties

#### securityContext

This property defines the security context a configuration requires. If you invoke a DSC operation
against the configuration document in a security context that conflicts with this metadata, DSC
raises an error when it validates the configuration document.

The valid security contexts are:

- `Current`

  Indicates that the configuration document is usable under any security context. You can invoke
  DSC operations against the document when elevated as root or an administrator and as a normal
  user or account.
- `Elevated`

  Indicates that the configuration document is usable only in an elevated security context. You can
  invoke DSC operations against the document when elevated as root or an administrator. When you
  invoke DSC operations against the document as a non-elevated user or account, DSC raises an error
  when it validates the configuration document.
- `Restricted`

  Indicates that the configuration document is usable only in a non-elevated security context. You
  can invoke DSC operations against the document as a non-elevated user or account. When you invoke
  DSC operations against the document as root or an administrator, DSC raises an error when it
  validates the configuration document.

The default security context is `Current`.

```yaml
Type:         object
Required:     false
Default:      Current
ValidValues: [Current, Elevated, Restricted]
```
