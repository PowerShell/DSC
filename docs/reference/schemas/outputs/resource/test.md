---
description: JSON schema reference for the data returned by the 'dsc resource test' command.
ms.date:     07/03/2025
ms.topic:    reference
title:       dsc resource test result schema reference
---

# dsc resource test result schema reference

## Synopsis

The result output from the `dsc resource test` command.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/outputs/resource/test.json
Type:          object
```

## Description

Describes the return data for a DSC Resource instance from the `dsc resource get` command. The
return data is either a single object that describes the tested state of a non-nested instance or
an array of objects that describe the tested state of the nested instances for a group or adapter
resource.

DSC returns a [simple test response](#simple-test-response) when the instance isn't a group
resource, adapter resource, or nested inside a group or adapter resource.

When the retrieved instance is for group resource, adapter resource, or nested inside a group or
adapter resource, DSC returns a [full test result](#full-test-result), which also includes the
resource type and instance name.

## Simple test response

### Required properties

The output always includes these properties:

- [desiredState](#desiredstate)

### Properties

#### desiredState

Represents the desired state of the resource instance. DSC validates this property's value against
the resource's instance schema.

```yaml
Type:     object
Required: true
```

#### actualState

Represents the actual state of the resource instance. DSC validates this property's value against
the resource's instance schema.

```yaml
Type:     object
Required: true
```

#### inDesiredState

Indicates whether the resource instance's properties are in the desired state. This value is `true`
if every property is in the desired state and otherwise `false`.

```yaml
Type:     boolean
Required: true
```

#### differingProperties

Defines the names of the properties that aren't in the desired state. If this value is an empty
array, the instance's properties are in the desired state.

```yaml
Type:      array
Required:  true
ItemsType: string
```

## Full test result

Describes the return data for the full result of the `test` operation for a resource instance. This
data is returned:

- For every instance in a configuration document when you use the `dsc config test` command.
- For nested instances of a group or adapter resource when you use the `dsc resource test` command.

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

The `result` property includes the validation state for the resource. This value is either:

- The [simple test response](#simple-test-response) for the instance
- An array of full get result objects for each nested instance, if the resource is a group or
  adapter resource.

```yaml
Type: [object, array]
Required: true
```

<!-- Link reference definitions -->
[01]: ../../metadata/Microsoft.DSC/properties.md#duration
[02]: https://datatracker.ietf.org/doc/html/rfc3339#appendix-A
[03]: ../../definitions/resourceType.md
