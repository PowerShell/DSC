---
description: Microsoft.Filesystem.File/Content resource reference documentation
ms.date:     10/19/2026
ms.topic:    reference
title:       Microsoft.Filesystem.File/Content
---

# Microsoft.Filesystem.File/Content

## Synopsis

Manage the UTF-8 content of a local file.

## Metadata

```yaml
Version    : 0.1.0
Kind       : resource
Tags       : [file, linux, macos, windows]
Author     : Microsoft
```

## Instance definition syntax

```yaml
resources:
  - name: <instance name>
    type: Microsoft.Filesystem.File/Content
    properties:
      # Required properties
      path: <string>
      # Instance properties
      content:
      sha256:
      sha512:
      _exist:
```

## Description

The `Microsoft.Filesystem.File/Content` resource manages the UTF-8 content of a local file. The
resource returns the file content and its SHA-256 and SHA-512 hashes when the file exists. It
creates missing parent directories when setting content. Set `content` to create or replace the
file; set `_exist` to `false` to remove it. A **Set** operation requires `content` unless
`_exist` is `false`.

The **Test** operation compares the desired content and any specified hashes with the file's
actual content. If no content or hash is specified, **Test** only checks whether the file exists.
Hash comparisons are case-insensitive. The resource implements **Test** directly; it doesn't use
DSC synthetic testing. It doesn't cache file state.

> [!NOTE]
> This resource is installed with DSC itself on all platforms.
>
> You can update this resource by updating DSC. When you update DSC, the updated version of this
> resource is automatically available.

## Capabilities

The resource has the following capabilities:

- `get` - You can use the resource to retrieve the actual state of an instance.
- `set` - You can use the resource to enforce the desired state for an instance.
- `test` - You can use the resource to test whether an instance is in the desired state.

This resource doesn't have the `export` capability. You can't use it to discover and export every
file instance.

For more information about resource capabilities, see [DSC resource capabilities][01].

## Examples

1. [Manage a local file's content][02] - Read an existing file, create or update a file with
   desired UTF-8 content, and remove the file when it is no longer needed.

## Properties

The following list describes the properties for the resource.

- **Required properties:** <a id="required-properties"></a> The following properties are always
  required when defining an instance of the resource. An instance that doesn't define each of these
  properties is invalid.

  - [path](#path) - The path to the local file.

- **Key properties:** <a id="key-properties"></a> This resource doesn't have any key properties.

- **Instance properties:** <a id="instance-properties"></a> The following properties are optional.
  They define the desired state for an instance of the resource.

  - [content](#content) - The desired UTF-8 content of the file.
  - [sha256](#sha256) - The expected SHA-256 hash of the file content.
  - [sha512](#sha512) - The expected SHA-512 hash of the file content.
  - [_exist](#_exist) - Defines whether the file should exist.

- **Read-only properties:** <a id="read-only-properties"></a> The resource returns the following
  properties, but they aren't configurable. For more information about read-only properties, see
  the "Read-only resource properties" section in [DSC resource properties][03].

  - [_inDesiredState](#_indesiredstate) - Indicates whether the file is in the desired state.

### path

<details><summary>Expand for <code>path</code> property metadata</summary>

```yaml
Type             : string
IsRequired       : true
IsKey            : false
IsReadOnly       : false
IsWriteOnly      : false
MinimumLength    : 1
```

</details>

Defines the path to the local file. The path must not be empty.

### content

<details><summary>Expand for <code>content</code> property metadata</summary>

```yaml
Type             : string
IsRequired       : false
IsKey            : false
IsReadOnly       : false
IsWriteOnly      : false
```

</details>

Defines the desired UTF-8 content of the file. A **Set** operation requires this property unless
`_exist` is `false`. When specified with `sha256` or `sha512`, the supplied hash must match the
hash of this content.

### sha256

<details><summary>Expand for <code>sha256</code> property metadata</summary>

```yaml
Type             : string
IsRequired       : false
IsKey            : false
IsReadOnly       : false
IsWriteOnly      : false
Pattern          : ^[A-Fa-f0-9]{64}$
```

</details>

Defines the expected SHA-256 hash of the file content. The value must contain exactly 64
hexadecimal characters. During **Test**, the resource compares this value with the actual
content hash. During **Set**, if `content` is specified, the resource verifies that this hash
matches the supplied content.

### sha512

<details><summary>Expand for <code>sha512</code> property metadata</summary>

```yaml
Type             : string
IsRequired       : false
IsKey            : false
IsReadOnly       : false
IsWriteOnly      : false
Pattern          : ^[A-Fa-f0-9]{128}$
```

</details>

Defines the expected SHA-512 hash of the file content. The value must contain exactly 128
hexadecimal characters. During **Test**, the resource compares this value with the actual content
hash. During **Set**, if `content` is specified, the resource verifies that this hash matches the
supplied content.

### _exist

<details><summary>Expand for <code>_exist</code> property metadata</summary>

```yaml
Type             : boolean
IsRequired       : false
IsKey            : false
IsReadOnly       : false
IsWriteOnly      : false
```

</details>

Defines whether the file should exist. Omit this property or set it to `true` to create or update
the file. Set it to `false` to remove the file; removing a file that doesn't exist succeeds.

### _inDesiredState

<details><summary>Expand for <code>_inDesiredState</code> property metadata</summary>

```yaml
Type             : boolean
IsRequired       : false
IsKey            : false
IsReadOnly       : true
IsWriteOnly      : false
```

</details>

Indicates whether the file is in the desired state. The resource returns this property from
**Test** operations.

## Instance validating schema

The following snippet contains the JSON Schema that validates an instance of the resource.

```json
{
  "type": "object",
  "required": [
    "path"
  ],
  "properties": {
    "content": {
      "type": "string"
    },
    "sha256": {
      "type": "string",
      "pattern": "^[A-Fa-f0-9]{64}$"
    },
    "sha512": {
      "type": "string",
      "pattern": "^[A-Fa-f0-9]{128}$"
    },
    "path": {
      "type": "string",
      "minLength": 1
    },
    "_exist": {
      "type": "boolean"
    },
    "_inDesiredState": {
      "type": "boolean",
      "readOnly": true
    }
  },
  "additionalProperties": false
}
```

## Exit codes

The resource returns the following exit codes from operations:

- [0](#exit-code-0) - Success.
- [1](#exit-code-1) - Invalid arguments.
- [2](#exit-code-2) - Invalid input.
- [3](#exit-code-3) - File operation error.

### Exit code 0

Indicates the resource operation completed without errors.

### Exit code 1

Indicates the operation or required input argument is missing or invalid. The resource emits a
JSON error message to standard error.

### Exit code 2

Indicates the input couldn't be parsed as the expected resource input. The resource emits a JSON
error message to standard error.

### Exit code 3

Indicates the file operation failed, such as when the file can't be read, written, or removed, or
when the supplied content doesn't match a specified hash. The resource emits a JSON error message
to standard error.

## See also

- [DSC resource capabilities][01]
- [DSC resource properties][03]

<!-- Link reference definitions -->
[01]: ../../../../../concepts/resources/capabilities.md
[02]: ./examples/manage-a-local-files-content.md
[03]: ../../../../../concepts/resources/properties.md
