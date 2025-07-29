---
description: JSON schema reference for a resource instance in a Desired State Configuration document.
ms.date:     07/03/2025
ms.topic:    reference
title:       DSC Configuration document resource instance schema
---

# DSC Configuration document resource instance schema

## Synopsis

Defines a DSC Resource instance in a configuration document.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/config/document.resource.json
Type:          object
```

## Description

The `resources` property of a DSC Configuration document always includes at least one DSC Resource
instance. Together, the instances in a configuration define the desired state that DSC can get,
test, and set on a machine.

This document describes the JSON schema for a valid DSC Resource instance in a configuration
document.

For more information about DSC Resources, see [Anatomy of a command-based DSC Resource][01].

## Required Properties

Every resource instance must be an object that defines these properties:

- [name](#name)
- [type](#type)

## Properties

### name

The `name` property of a resource instance defines the short, human-readable name for a DSC
Resource instance. This property must be unique within a DSC Configuration document. If any
resource instances share the same name, DSC raises an error.

The instance name must be a non-empty string containing only letters, numbers, and spaces.

```yaml
Type:          string
Required:      true
MinimumLength: 1
Pattern:       ^[a-zA-Z0-9 ]+$
```

### type

The `type` property identifies the instance's DSC Resource. The value for this property must be the
valid fully qualified type name for the resource. For more information about type names, see
[DSC Resource fully qualified type name schema reference][01].

```yaml
Type:     string
Required: true
Pattern:  ^\w+(\.\w+){0,2}\/\w+$
```

### properties

The `properties` of a resource instance define its desired state. The value of this property must
be an object. For assertion  resources, the value may be an empty object (`{}`). DSC uses the
DSC Resource's instance schema to validate the defined properties.

<!-- For more information about instance schemas in DSC, see [DSC Resource instance schemas][aa]. -->

```yaml
Type:     object
Required: true
```

### dependsOn

To declare that a resource instance is dependent on another instance in the configuration, define
the `dependsOn` property.

This property defines a list of DSC Resource instances that DSC must successfully process before
processing this instance. Each value for this property must be the [resourceID() function][02]
lookup for another instance in the configuration. Multiple instances can depend on the same
instance, but every dependency for an instance must be unique in that instance's `dependsOn`
property.

The `resourceID()` function uses this syntax:

```yaml
"[resourceId('<resource-type-name>', '<instance-name>']"
```

The `<resource-type-name>` value is the `type` property of the dependent resource and
`<instance-name>` is the dependency's `name` property. When adding a dependency in a YAML-format
configuration document, always wrap the `resourceID()` lookup in double quotes (`"`).

For example, this instance depends on an instance of the `Microsoft.Windows/Registry`
resource named `Tailspin Key`:

```yaml
- name: Tailspin Key
  type: Microsoft.Windows/Registry
  properties:
    keyPath: HKCU\tailspin
    _ensure: Present
- name: Update Tailspin Automatically
  type: Microsoft.Windows/Registry
  properties:
    keyPath:   HKCU\tailspin\updates
    valueName: automatic
    valueData:
      String: enable
  dependsOn:
    - "[resourceId('Microsoft.Windows/Registry', 'Tailspin Key')]"
```

> [!NOTE]
> When defining dependencies for [nested resource instances][03], instances can only reference
> dependencies in the same resource provider or group instance. They can't use the `resourceId()`
> function to lookup instances at the top-level of the configuration document or inside another
> provider or group instance.
>
> If a top-level instance depends on a nested instance, use the `resourceId()` function to lookup
> the instance of the provider or group containing the dependency instance instead.

For more information about using functions in configuration documents, see
[DSC Configuration document functions reference][04]. For more information about the `resourceId()`
function, see [resourceId][02].

<!-- For more information, see [Configuration resource dependencies][ab]. -->

```yaml
Type:              array
Required:          false
ItemsMustBeUnique: true
ItemsType:         string
ItemsPattern:      ^\[resourceId\(\s*'\w+(\.\w+){0,2}\/\w+'\s*,\s*'[a-zA-Z0-9 ]+'\s*\)\]$
```

[01]: ../definitions/resourceType.md
[02]: functions/resourceId.md
[03]: /powershell/dsc/glossary#nested-resource-instance
[04]: functions/overview.md
<!-- [aa]: ../../../resources/concepts/schemas.md -->
<!-- [ab]: ../../../configurations/concepts/dependencies.md -->
