---
description: JSON schema reference for the '_exist' canonical DSC Resource property.
ms.date:     02/28/2025
ms.topic:    reference
title:       DSC Resource _exist property schema
---

# DSC Resource _exist property schema

## Synopsis

Indicates whether an instance should exist.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/resource/properties/exist.json
Type:          boolean
DefaultValue:  true
```

## Description

The `_exist` canonical property indicates that the resource can enforce whether instances exist,
handling whether an instance should be added, updated, or removed during a set operation. This
property provides shared semantics for DSC Resources and integrating tools. Resources that define
this property indicate to DSC that they adhere to the contract for the canonical property.

Resources should only define this property when their implementation adheres to the following
behavior contract:

1. When the desired state for `_exist` is `true`, the resource expects the instance to exist. If it
   doesn't exist, the resource creates or adds the instance during the set operation.
1. When the desired state for `_exist` is `false`, the resource expects the instance to not exist.
   If it does exist, the resource deletes or removes the instance during the set operation.
1. When the get operation queries for an instance that doesn't exist, the returned JSON always
   defines the `_exist` property as `false`.

   The resource _may_ omit the `_exist` property from the result JSON when the instance exists.

To add this property to a resource's instance schema, define the property with the following
snippet:

```json
"_exist": {
  "$ref": "https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/resource/properties/exist.json"
}
```

<!-- TODO: Enumerate the other available URIs and describe which to select and why -->
