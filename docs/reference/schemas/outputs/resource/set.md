---
description: JSON schema reference for the data returned by the 'dsc resource set' command.
ms.date:     02/28/2025
ms.topic:    reference
title:       dsc resource set result schema reference
---

# dsc resource set result schema reference

## Synopsis

The result output from the `dsc resource set` command.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/outputs/resource/set.json
Type:          object
```

## Description

Describes the return data for a DSC Resource instance from the `dsc resource set` command. The
return data is either a single object that describes the enforced state of a non-nested instance or
an array of objects that describe the enforced state of the nested instances for a group or adapter
resource.

DSC returns a [simple set response](#simple-set-response) when the instance isn't a group resource,
adapter resource, or nested inside a group or adapter resource.

When the retrieved instance is for group resource, adapter resource, or nested inside a group or
adapter resource, DSC returns a [full set result](#full-set-result), which also includes the
resource type and instance name.

## Simple set response

Describes the return data for a DSC Resource instance from the `dsc resource set` command. The
return data is either a single object that describes the enforced state of a non-nested instance or
an array of objects that describe the enforced state of the nested instances for a group or adapter
resource.

### Required properties

The output always includes these properties:

- [beforeState](#beforestate)
- [afterState](#afterstate)
- [changedProperties](#changedproperties)

### Properties

#### beforeState

Represents the state of the instance returned before the set operation. DSC validates this
property's value against the resource's instance schema.

```yaml
Type:     object
Required: true
```

#### afterState

Represents the state of the instance returned after the set operation. DSC validates this
property's value against the resource's instance schema.

```yaml
Type:     object
Required: true
```

#### changedProperties

Defines the names of the properties the set operation enforced. If this value is an empty array,
the resource made no changes during the set operation.

```yaml
Type:      array
Required:  true
ItemsType: string
```

## Full set result

Describes the return data for the full result of the `set` operation for a resource instance. This
data is returned:

- For every instance in a configuration document when you use the `dsc config set` command.
- For nested instances of a group or adapter resource when you use the `dsc resource set` command.

### Required properties

- [metadata](#metadata-1)
- [name](#name)
- [type](#type)
- [result](#result)

### Properties

#### metadata

Defines metadata DSC returns for a configuration operation. The properties under the
`Microsoft.DSC` property describe the context of the operation.

- [duration][01] defines the duration of a DSC operation against a configuration document or
  resource instance as a string following the format defined in [ISO8601 ABNF for `duration`][02].
  For example, `PT0.611216S` represents a duration of about `0.61` seconds.

```yaml
Type:     object
Required: true
```

#### type

The `type` property identifies the instance's DSC Resource by its fully qualified type name.
For more information about type names, see
[DSC Resource fully qualified type name schema reference][03].

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

The `result` property includes the enforced state for the resource. This value is either:

- The [simple set response](#simple-set-response) for the instance
- An array of full set result objects for each nested instance, if the resource is a group or
  adapter resource.

```yaml
Type: [object, array]
Required: true
```

<!-- Link reference definitions -->
[01]: ../../metadata/Microsoft.DSC/properties.md#duration
[02]: https://datatracker.ietf.org/doc/html/rfc3339#appendix-A
[03]: ../../definitions/resourceType.md
