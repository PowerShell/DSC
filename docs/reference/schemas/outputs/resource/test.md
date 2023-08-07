---
description: JSON schema reference for the data returned by the 'dsc resource test' command.
ms.date:     08/04/2023
ms.topic:    reference
title:       dsc resource test result schema reference
---

# dsc resource test result schema reference

## Synopsis

The result output from the `dsc resource test` command.

## Metadata

```yaml
Schema Dialect : https://json-schema.org/draft/2020-12/schema
Schema ID      : https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/08/outputs/resource/test.json
Type           : object
```

## Description

The output from the `dsc resource test` command includes the actual state for the specified
resource instance.

## Required properties

The output always includes these properties:

- [desiredState](#desiredstate)

## Properties

### desiredState

Represents the desired state of the resource instance. DSC validates this property's value against
the resource's instance schema.

```yaml
Type:     object
Required: true
```

### actualState

Represents the actual state of the resource instance. DSC validates this property's value against
the resource's instance schema.

```yaml
Type:     object
Required: true
```

### inDesiredState

Indicates whether the resource instance's properties are in the desired state. This value is `true`
if every property is in the desired state and otherwise `false`.

```yaml
Type:     boolean
Required: true
```

### differingProperties

Defines the names of the properties that aren't in the desired state. If this value is an empty
array, the instance's properties are in the desired state.

```yaml
Type:       array
Required:   true
Items Type: string
```
