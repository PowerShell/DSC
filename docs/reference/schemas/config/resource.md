---
description: JSON schema reference for a resource instance in a Desired State Configuration document.
ms.date:     09/01/2026
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

The `resources` property of a DSC Configuration document defines the DSC Resource instances for the
configuration. Together, the instances in a configuration define the desired state that DSC can
get, test, and set on a machine.

This document describes the JSON schema for a valid DSC Resource instance in a configuration
document.

For more information about DSC Resources, see [Anatomy of a command-based DSC Resource][01].

## Required Properties

Every resource instance must be an object that defines these properties:

- [type](#type)

## Properties

### condition

The `condition` property defines an expression that DSC evaluates before invoking the instance. If
the expression evaluates to `true`, DSC invokes the instance as normal. If it evaluates to any other
value, DSC skips the instance and doesn't include it in the results for the operation.

For example, DSC only invokes this instance when the `enableFeature` parameter is `true`:

```yaml
parameters:
  enableFeature:
    type: bool
    defaultValue: false
resources:
  - name: Feature
    type: Microsoft.DSC.Debug/Echo
    condition: "[parameters('enableFeature')]"
    properties:
      output: Feature enabled
```

```yaml
Type:     string
Required: false
```

### type

The `type` property identifies the instance's DSC Resource. The value for this property must be the
valid fully qualified type name for the resource. For more information about type names, see
[DSC Resource fully qualified type name schema reference][01].

```yaml
Type:     string
Required: true
Pattern:  ^\w+(\.\w+)*\/\w+$
```

### requireVersion

The `requireVersion` property pins the instance to a specific version or a range of versions of the
resource. DSC only invokes a discovered resource whose version satisfies the requirement and raises
an error if it can't find one. When this property isn't defined, DSC doesn't constrain the version
of the resource.

Define the value as a semantic version requirement: one or more comparators separated by commas.
Each comparator is an operator (`=`, `>`, `>=`, `<`, `<=`, `^`, or `~`) followed by a version.
Build metadata isn't allowed in the version. For example:

- `=1.2.3` - exactly version `1.2.3`.
- `>=1.2.3, <2.0.0` - any version from `1.2.3` up to, but not including, `2.0.0`.
- `^1.2` - any version from `1.2.0` up to, but not including, `2.0.0`.
- `~2.3` - any version from `2.3.0` up to, but not including, `2.4.0`.

For compatibility with resources that use date versions, the value can also be a date version like
`2026-02-03` or `2026-11-27-preview`. A date version requirement only matches a resource with
exactly the same date version. Date versions are deprecated. Use semantic versions instead.

This property has the alias `apiVersion`. You can define the requirement with either property name,
but not both.

```yaml
Type:     string
Required: false
```

### name

The `name` property defines the short, human-readable name for the instance. DSC uses the name
together with the `type` to identify the instance in results and messages, and for the
[resourceId()][02] function. The combination of `type` and `name` must be unique within a
configuration document. If two instances share the same type and name, DSC raises an error.

The value can be a configuration expression, like `"[format('Server-{0}', copyIndex())]"` for an
instance in a copy loop. DSC evaluates the expression to a string before invoking the instance.

Although the schema doesn't require this property, always define a unique name for every instance.
When the property isn't defined, DSC uses an empty string as the name.

```yaml
Type:     string
Required: false
Default:  ""
```

### directives

The `directives` property of a resource instance defines per-instance overrides for how DSC should
process the resource. This property was added in DSC version 3.2.

```yaml
Type:     object
Required: false
```

You can define the following directives for a resource instance:

#### requireAdapter

The `requireAdapter` directive indicates that DSC should use the specified adapter to invoke the
adapted resource instance. The value for this directive must be the fully qualified type name of
the adapter resource, like `Microsoft.Adapter/PowerShell`.

When this directive isn't specified, DSC invokes the adapted resource through the first discovered
adapter that indicates it can invoke the resource. This directive has no effect on nonadapted
resource instances.

```yaml
Type:     string
Required: false
Pattern:  ^\w+(\.\w+)*\/\w+$
```

#### securityContext

The `securityContext` directive indicates that DSC should validate the current security context
against this directive before invoking the resource. This value overrides the
`directives.securityContext` setting for the top level of the configuration document. This enables
you to selectively require or forbid elevated security contexts for a specific resource instance.

```yaml
Type:        string
Required:    false
ValidValues: [current, elevated, restricted]
```

### executionInformation

The `executionInformation` property describes the DSC operation that produced the instance. DSC
adds this property to every instance in the configuration document returned by the
`dsc config export` command. The schema accepts this property for any instance, but DSC ignores it
when it processes a configuration document.

The value is an object with the same properties as the [Microsoft.DSC metadata][03] object that DSC
returns in command output, plus an optional `whatIf` property that describes any what-if
operations DSC performed.

```yaml
Type:     object
Required: false
```

### dependsOn

To declare that a resource instance is dependent on another instance in the configuration, define
the `dependsOn` property.

This property defines a list of DSC Resource instances that DSC must successfully process before
processing this instance. Each value for this property must be an expression that uses the
[resourceId() function][02] to look up another instance in the configuration. Multiple instances
can depend on the same instance.

The `resourceId()` function uses this syntax:

```yaml
"[resourceId('<resource-type-name>', '<instance-name>')]"
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

For an instance in a copy loop, you can use the [copyIndex()][04] function in the expression to
depend on the matching iteration of another copy loop, like
`"[resourceId('Microsoft.DSC.Debug/Echo', format('Policy-{0}', copyIndex()))]"`.

> [!NOTE]
> When defining dependencies for [nested resource instances][05], instances can only reference
> dependencies in the same resource provider or group instance. They can't use the `resourceId()`
> function to lookup instances at the top-level of the configuration document or inside another
> provider or group instance.
>
> If a top-level instance depends on a nested instance, use the `resourceId()` function to lookup
> the instance of the provider or group containing the dependency instance instead.

For more information about using functions in configuration documents, see
[DSC Configuration document functions reference][06]. For more information about the `resourceId()`
function, see [resourceId][02].

<!-- For more information, see [Configuration resource dependencies][ab]. -->

```yaml
Type:      array
Required:  false
ItemsType: string
```

### copy

The `copy` property defines a copy loop that expands the instance into multiple instances that
share the same definition. Before invoking any resources, DSC creates one instance for each
iteration of the loop. Use the [copyIndex()][04] function in the instance's `name`, `properties`,
and `dependsOn` expressions to make each expanded instance unique. The `name` of an instance in a
copy loop must be an expression that evaluates to a different string for every iteration.

> [!IMPORTANT]
> Copy loops are deprecated. DSC raises a warning when a configuration document uses the `copy`
> property. The functionality remains available for compatibility but will be removed in DSC
> version 4.0.0. For more information, see [DSC issue #1429][07].

The value for this property is an object with the following properties:

- `name` - Required. The name of the copy loop. Pass this name to `copyIndex()` to get the current
  iteration index for a specific loop.
- `count` - Required. The number of iterations, as an integer or an expression that evaluates to an
  integer, like `"[parameters('serverCount')]"`. A count of `0` expands to no instances.
- `mode` - Optional. Reserved for future use. The schema accepts the values `serial` and
  `parallel`, but DSC raises an error if you define this property.
- `batchSize` - Optional. Reserved for future use. The schema accepts an integer or an expression,
  but DSC raises an error if you define this property.

For example, this instance expands into the `Server-0` and `Server-1` instances:

```yaml
resources:
  - name: "[format('Server-{0}', copyIndex())]"
    type: Microsoft.DSC.Debug/Echo
    copy:
      name: serverLoop
      count: 2
    properties:
      output: "[format('Instance-{0}', copyIndex())]"
```

```yaml
Type:     object
Required: false
```

### properties

The `properties` of a resource instance define its desired state. The value of this property must
be an object. For assertion resources, the value can be an empty object (`{}`). DSC uses the
DSC Resource's instance schema to validate the defined properties.

<!-- For more information about instance schemas, see [DSC Resource instance schemas][aa]. -->

```yaml
Type:     object
Required: false
```

### resources

The `resources` property defines a list of nested resource instances that use the same schema as a
top-level instance. This property mirrors the shape of a resource in an Azure Resource Manager
(ARM) template. The schema accepts this property, but DSC doesn't currently process instances
defined in it. To manage nested instances, use a group resource like `Microsoft.DSC/Group` and
define the nested instances in the group's `properties`.

```yaml
Type:      array
Required:  false
ItemsType: object
```

### metadata

The `metadata` property defines a set of key-value pairs as annotations for the resource instance.
Except for the `Microsoft.DSC` property, DSC doesn't validate the metadata. When DSC invokes the
resource, it passes the metadata to the resource as the `_metadata` property of the instance if the
resource's instance schema allows it. For adapters that accept the full configuration as input, DSC
passes the metadata as the `metadata` property instead.

The `Microsoft.DSC` property is reserved for DSC. DSC honors the deprecated `securityContext`
setting in this property for the instance, and adds the `copyLoops` property to instances it
expands from a copy loop. For more information, see
[DSC Configuration document metadata schema][08].

```yaml
Type:     object
Required: false
```

<!-- Link reference definitions -->
[01]: ../definitions/resourceType.md
[02]: functions/resourceId.md
[03]: ../metadata/Microsoft.DSC/properties.md
[04]: functions/copyIndex.md
[05]: ../../../glossary.md#nested-resource-instance
[06]: functions/overview.md
[07]: https://github.com/PowerShell/DSC/issues/1429
[08]: metadata.md
<!-- [aa]: ../../../resources/concepts/schemas.md -->
<!-- [ab]: ../../../configurations/concepts/dependencies.md -->
