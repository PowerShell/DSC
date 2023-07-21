# DSC Configuration document resource schema

Defines instances of DSC Resources that compose a configuration.

## Metadata

```yaml
Schema Dialect : https://json-schema.org/draft/2020-12/schema
Schema ID      : https://schemas.microsoft.com/dsc/2023/07/config/document.resources.json
Type           : object
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

```yaml
Type:     string
Required: true
```

### type

The `type` property identifies the instance's DSC Resource. The value for this property must be the
valid fully-qualified type name for the resource. For more information about type names, see
[Anatomy of a command-based DSC Resource][01].

```yaml
Type:     string
Required: true
Pattern:  ^\w+(\.\w+){0,2}\/\w+$
```

### properties

The `properties` of a resource instance define its desired state. The value of this property must
be an object. For [assertion  resources][02], the value may be an empty object (`{}`). DSC uses the
DSC Resource's instance schema to validate the defined properties.

For more information about instance schemas in DSC, see [DSC Resource instance schemas][03].

```yaml
Type:     object
Required: true
```

### dependsOn

To declare that a resource instance is dependent on another instance in the configuration, define the `dependsOn` property.

This property must be an array of dependency declarations. Each dependency must use this
syntax:

```yaml
"[<resource-type-name>]<instance-name>"
```

In the dependency syntax, `<resource-type-name>` is the `type` property of the dependent resource
and `<instance-name>` is the dependency's `name` property.

Multiple instances can depend on the same instance, but every dependency for an instance must be
unique in that instance's `dependsOn` property.

For more information, see [Configuration resource dependencies][04].

```yaml
Type:                 array
Required:             false
Items Must be Unique: true
Valid Items Type:     string
Valid Items Pattern:  ^\[\w+(\.\w+){0,2}\/\w+\].+$
```

[01]: ../../../resources/concepts/anatomy.md
[02]: ../../../resources/concepts/assertion-resources.md
[03]: ../../../resources/concepts/schemas.md
[04]: ../../../configurations/concepts/dependencies.md
