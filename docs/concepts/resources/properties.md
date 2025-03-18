---
description: >-
  Describes what DSC resource properties are, how they behave, and how to use
  them.
ms.date:     03/25/2025
ms.topic:    conceptual
title:       DSC resource properties
---

# DSC resource properties

Every DSC resource defines how you can manage an instance of the resource as a set of properties.
Resources define each property as part of their instance JSON schema.

When you define an instance of a resource to use for invoking an operation directly or in a
configuration document, DSC uses the instance schema to validate that data before invoking the
resource.

If you specify an invalid property name or an invalid value for a property, DSC raises an error
describing how the data is invalid.

Unless otherwise noted, for any given property:

- You can define the property for the desired state of a resource instance.
- The resource can retrieve the actual state of the property from the system.
- The resource can enforce the desired state of the property on the system.
- You can omit the property from the desired state if you don't want to manage it.

The preceding list describes a typical resource property. However, DSC recognizes different
attributes for a property that can change how you use the property:

- If the instance schema defines the `x-dsc-key` keyword for the property subschema as `true`, the
  property is a _key resource property_. Key properties uniquely identify a resource instance on
  the system to prevent conflicts in configuration documents.
  
  For more information, see the [Key properties](#key-resource-properties) section.
- If the instance schema defines the property name in the `required` keyword, the property is a
  _required resource property_. You _must_ define the property in the desired state for the
  instance. If you omit the property, DSC raises an error because the instance is invalid.
  
  For more information, see the [Required properties](#required-resource-properties) section.
- If the instance schema defines the `readOnly` keyword for the property subschema as `true`, the
  property is a _read-only resource property_. You can't define the property in the desired state
  for an instance. The resource can't set a read-only property. It can only return the actual state
  for that property on an instance.

  For more information, see the [Read-only properties](#read-only-resource-properties) section.
- If the instance schema defines the `writeOnly` keyword for the property subschema as `true`,
  property is a _write-only resource property_. The resource never returns a value for that
  property. You can use the property to control how the resource behaves, rather than directly map
  the value for the property to the instance state.

  For more information, see the [Write-only properties](#write-only-resource-properties) section.

Additionally, DSC defines a set of canonical properties that enable resources to participate in the
semantics of the DSC engine. Canonical resource properties are reusable subschemas that indicate
the resource adheres to certain contracts that you can rely on when using the resource.

## Key resource properties

DSC uses key resource properties to uniquely identify instances of the resource on a system. If you
specify two or more instances of the resource with the same key properties, you're attempting to
manage the same instance more than once.

Instances in a configuration document with the same values for their key properties are
_conflicting instances_. Never define conflicting instances in a configuration document. In a
future release, DSC will raise an error for a configuration document containing any conflicting
instances.

If you define different settings for conflicting instances, DSC invokes the resource for each
conflicting instance during every **Set** operation. In this case, later instances override any
settings defined by an earlier conflicting instance in the configuration document.

If the settings for conflicting instances are the same, the resource is still invoked for each
instance, which expends time and resources without benefit.

DSC determines whether a resource property is a key property by examining the property subschema
in the instance schema. If the subschema defines the `x-dsc-key` keyword with the value `true`,
the property is a key property.

## Required resource properties

When you're defining a resource instance, some properties might be required. An instance that
doesn't define every required property is invalid. DSC validates instance definitions before
invoking resource operations. When an instance is missing any required properties, DSC raises a
validation error and doesn't invoke the resource.

Properties can be _always_ required. DSC determines whether a resource property is a required
property by examining the `required` keyword in the instance schema. If the instance schema defines
the `required` keyword and the property name is included in the array of values for the keyword,
the property is always required.

Properties can be _conditionally_ required. Resources can conditionally require a property with the
`dependentRequires` keyword or other conditional keywords. For more information about conditionally
applied subschemas, see
[Conditional schema validation][01].

## Read-only resource properties

Resources can define read-only properties to describe information about an instance that the
resource can retrieve but not directly set. For example, file APIs don't generally allow a user to
set the property describing the last time the file was modified.

Generally, you shouldn't include read-only properties when defining the desired state for an
instance. Assertion resources that don't support the **Set** operation can include read-only
properties you can use for validating system state for conditional behavior.

DSC determines whether a resource property is a read-only property by examining the property
subschema in the instance schema. If the subschema defines the `readOnly` keyword with the value
`true`, the property is read-only.

## Write-only resource properties

Resources can define write-only properties that affect how the resource behaves but that the
resource can't retrieve for the actual state of an instance. For example, a resource might support
credentials for downloading a file, but won't return those credentials in the output.

DSC determines whether a resource property is a write-only property by examining the property
subschema in the instance schema. If the subschema defines the `writeOnly` keyword with the value
`true`, the property is write-only.

## Canonical resource properties

DSC defines a set of subschemas representing reusable properties with defined behaviors and
expectations. These reusable properties are called canonical resource properties.

Any resource that defines a canonical resource property in the instance schema must adhere to the
requirements and behaviors defined for the canonical property.

For more information about the available canonical properties, see
[DSC canonical resource properties][02].

## Related

- [DSC Resource Manifest schema reference][03]
- [JSON Schema reference][04]

<!-- Link reference definitions -->
[01]: https://json-schema.org/understanding-json-schema/reference/conditionals
[02]: ../../reference/schemas/resource/properties/overview.md
[03]: ../../reference/schemas/resource/manifest/root.md
[04]: https://json-schema.org/understanding-json-schema/reference
