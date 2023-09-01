---
description: JSON schema reference for the data returned by the 'dsc resource set' command.
ms.date:     08/04/2023
ms.topic:    reference
title:       dsc resource set result schema reference
---

# dsc resource set result schema reference

## Synopsis

The result output from the `dsc resource set` command.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/08/outputs/resource/set.json
Type:          object
```

## Description

The output from the `dsc resource set` command includes the state of the resource instance before
and after the set operation, and the list of properties the operation changed.

## Required properties

The output always includes these properties:

- [beforeState](#beforestate)
- [afterState](#afterstate)
- [changedProperties](#changedproperties)

## Properties

### beforeState

Represents the state of the instance returned before the set operation. DSC validates this
property's value against the resource's instance schema.

```yaml
Type:     object
Required: true
```

### afterState

Represents the state of the instance returned after the set operation. DSC validates this
property's value against the resource's instance schema.

```yaml
Type:     object
Required: true
```

### changedProperties

Defines the names of the properties the set operation enforced. If this value is an empty array,
the resource made no changes during the set operation.

```yaml
Type:      array
Required:  true
ItemsType: string
```
