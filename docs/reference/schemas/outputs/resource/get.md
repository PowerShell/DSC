---
description: JSON schema reference for the data returned by the 'dsc resource get' command.
ms.date:     01/17/2024
ms.topic:    reference
title:       dsc resource get result schema reference
---

# dsc resource get result schema reference

## Synopsis

The result output from the `dsc resource get` command.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/10/outputs/resource/get.json
Type:          object
```

## Description

Describes the return data for a DSC Resource instance from the `dsc resource get` command. The
return data is either a single object that describes the actual state of a non-nested instance or
an array of objects that describe the actual state of the nested instances for a group or adapter
resource.

DSC returns a [simple get response](#simple-get-response) when the instance isn't a group resource,
adapter resource, or nested inside a group or adapter resource.

When the retrieved instance is for group resource, adapter resource, or nested inside a group or
adapter resource, DSC returns a [full get result](#full-get-result), which also includes the
resource type and instance name.

## Simple get response

Describes the return data for a single DSC Resource instance from the `dsc resource get` command.
This data is returned for instances that aren't group resources, adapter resources, or nested
inside a group or adapter resource.

### Required properties

The output always includes these properties:

- [actualState](#actualstate)

### Properties

#### actualState

The `actualState` property always includes the state of the instance returned when DSC invokes the
resource's get operation. DSC validates this property's value against the resource's instance
schema.

```yaml
Type:     object
Required: true
```

## Full get result

Describes the return data for the full result of the `get` operation for a resource instance. This
data is returned:

- For every instance in a configuration document when you use the `dsc config get` command.
- For nested instances of a group or adapter resource when you use the `dsc resource get` command.

### Required properties

- [name](#name)
- [type](#type)
- [result](#result)

### Properties

#### type

The `type` property identifies the instance's DSC Resource by its fully qualified type name.
For more information about type names, see
[DSC Resource fully qualified type name schema reference][01].

```yaml
Type:     string
Required: true
Pattern:  ^\w+(\.\w+){0,2}\/\w+$
```

#### name

The `name` property identifies the instance by its short, unique, human-readable name.

```yaml
Type:     string
Required: true
```

#### result

The `result` property includes the actual state for the resource. This value is either:

- The [simple get response](#simple-get-response) for the instance
- An array of full get result objects for each nested instance, if the resource is a group or
  adapter resource.

```yaml
Type: [object, array]
Required: true
```

[01]: ../../definitions/resourceType.md
