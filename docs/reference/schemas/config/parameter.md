---
description: JSON schema reference for a parameter in a Desired State Configuration document.
ms.date:     01/17/2024
ms.topic:    reference
title:       DSC Configuration document parameter schema
---

# DSC Configuration document parameter schema

## Synopsis

Defines runtime options for a configuration.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/10/config/document.parameter.json
Type:          object
```

## Description

DSC Configuration documents can include parameters, which users can override at runtime. Parameters
enable separating secrets from configuration definitions and enable users to write configurations
that can apply to multiple contexts.

Parameters are defined as key-value pairs in the `parameters` property of a configuration document.
The key is the parameter's name, which is used to reference the parameter in the [resources][01]
property of the configuration document. The value is an object that defines the parameter.

Every parameter defines its data type. Parameters may also define a default value, validation
checks, a description of their purpose, and arbitrary metadata.

## Required Properties

- [type](#type)

## Properties

### description

Parameters may define a short explanation of their purpose and usage with the `description`
property. To define a longer explanation in YAML, use the folded block syntax or literal block
syntax.

```yaml
Type:     string
Required: false
```

### metadata

The `metadata` property defines a set of key-value pairs as annotations for the parameter. DSC
doesn't validate the metadata. A parameter can include any arbitrary information in this property.

```yaml
Type:     object
Required: false
```

### type

Every parameter must define the data type that it expects as the `type` property. DSC validates the
data type for every passed parameter before executing a configuration operation.

The `secure*` data types indicate that DSC and integrating tools shouldn't log or record the
values. If a secure data type parameter is used for a resource instance property that doesn't
expect a secure value, the resource may still log or record the value. If the resource has
independent logging or recording that isn't handled by DSC, the value may be stored insecurely.

Use secure strings for passwords and secrets.

For more information about data types, see
[DSC configuration parameter data type schema reference][02].

```yaml
Type:        string
Required:    true
ValidValues: [string, securestring, int, bool, object, secureobject, array]
```

### defaultValue

Parameters may define a default value with the `defaultValue` property. If the parameter isn't
passed at runtime, DSC uses the default value for the parameter. If the parameter isn't passed at
runtime and no default value is defined, DSC raises an error. The value must be valid for the
parameter's `type`.

```yaml
Required:       false
ValidJSONTypes: [string, integer, object, array, boolean]
```

### allowedValues

Parameters may limit the set of valid values for the parameter by defining the `allowedValues`
property. DSC validates parameters passed at runtime and defined as `defaultValue` against this
list of values. If any of the values is invalid, DSC raises an error.

This property is always an array. If this property is defined, it must include at least one item in
the list of values.

```yaml
Type:               array
Required:           false
ValidItemJSONTypes: [string, integer, object, array, boolean]
```

### minLength

The `minLength` property defines a validation option for array and string parameters. The length of
a string is its character count. The length of an array is its item count.

If the default value or runtime value for the parameter is shorter than this property, DSC raises
an error. If this property is defined for parameters whose `type` isn't `array`, `string`, or
`securestring`, DSC raises an error.

If this property is defined with the `maxLength` property, this property must be less than
`maxLength`. If it isn't, DSC raises an error.

```yaml
Type:         int
Required:     false
MinimumValue: 0
```

### maxLength

The `maxLength` property defines a validation option for array and string parameters. The length of
a string is its character count. The length of an array is its item count.

If the default value or runtime value for the parameter is longer than this property, DSC raises an
error. If this property is defined for parameters whose `type` isn't `array`, `string`, or
`securestring`, DSC raises an error.

If this property is defined with the `minLength` property, this property must be greater than
`minLength`. If it isn't, DSC raises an error.

```yaml
Type:         int
Required:     false
MinimumValue: 0
```

### minValue

The `minValue` property defines a validation option for integer parameters. If the default value or
runtime value for the parameter is less than this property, DSC raises an error. If this property
is defined for parameters whose `type` isn't `int`, DSC raises an error.

If this property is defined with the `maxValue` property, this property must be less than
`maxValue`. If it isn't, DSC raises an error.

```yaml
Type:     int
Required: false
```

### maxValue

The `maxValue` property defines a validation option for integer parameters. If the default value or
runtime value for the parameter is greater than this property, DSC raises an error. If this
property is defined for parameters whose `type` isn't `int`, DSC raises an error.

If this property is defined with the `minValue` property, this property must be greater than
`minValue`. If it isn't, DSC raises an error.

```yaml
Type:     int
Required: false
```

[01]: resource.md
[02]: ../definitions/parameters/dataTypes.md
