---
description: JSON schema reference for the '_purge' canonical DSC Resource property.
ms.date:     07/03/2025
ms.topic:    reference
title:       DSC Resource _purge property schema
---

# DSC Resource _purge property schema

## Synopsis

Indicates that the resource should treat non-defined entries in a list as invalid.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/resource/properties/purge.json
Type:          [boolean, 'null']
WriteOnly:     true
```

## Description

DSC Resources that need to distinguish between whether unmanaged entries in a list are valid or
must be removed can define the `_purge` property.

When a resource's instance schema defines this property, it indicates that the resource changes its
behavior based on the property's value in an instance's desired state:

- When `_purge` is `true`, the resource removes unmanaged entries. The resource treats any entries
  not listed in the instance's desired state as invalid.
- When `_purge` is `false` or not specified, the resource ignores unmanaged entries.

When a resource defines this property, it should always document which property or properties
`_purge` affects. A resource may define `_purge` as a subproperty for a complex property.

This property is write-only. A resource that uses the `_purge` property should never return
`_purge` in the instance's output state. A resource must not define `_purge` as a mandatory
property.

To add this property to a resource's instance schema, define the property with the following
snippet:

```json
"_inDesiredState": {
  "$ref": "https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/resource/properties/purge.json"
}
```

<!-- TODO: Enumerate the other available URIs and describe which to select and why -->
