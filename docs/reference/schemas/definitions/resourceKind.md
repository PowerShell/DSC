---
description: JSON schema reference for resource kind
ms.date:     02/28/2025
ms.topic:    reference
title:       DSC Resource kind schema reference
---

# DSC Resource kind schema reference

## Synopsis

Identifies whether a resource is an adapter resource, a group resource, or a normal resource.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/definitions/resourceKind.json
Type:          string
ValidValues:  [resource, adapter, group, importer]
```

## Description

DSC supports three kinds of command-based DSC Resources:

- `resource` - Indicates that the manifest isn't for a group or adapter resource.
- `group` - Indicates that the manifest is for a [group resource](#group-resources).
- `adapter` - Indicates that the manifest is for an [adapter resource](#adapter-resources).
- `importer` - Indicates that the manifest is for an [importer resource](#importer-resources).
- `exporter` - Indicates that the manifest is for an [exporter resource](#exporter-resources).

When `kind` isn't defined in the resource manifest, DSC infers the value for the property. If the
`adapter` property is defined in the resource manifest, DSC infers the value of `kind` as
`adapter`. If the `adapter` property isn't defined, DSC infers the value of `kind` as `resource`.
DSC can't infer whether a manifest is for a group or importer resource.

When defining a group resource, always explicitly define the `kind` property in the manifest as
`group`. When defining an importer resource, always explicitly define the `kind` property in the
manifest as `importer`.

### Adapter resources

An adapter resource makes non-command resources available to DSC. They always have a `resources`
property that takes an array of nested resource instances. Adapters may provide additional control
over how the adapted resources are processed.

An adapter resource must always define the [adapter][01] and [validate][02] properties in the
resource manifest.

For example, the `Microsoft.DSC/PowerShell` adapter enables you to use PowerShell Desired State
Configuration (PSDSC) resources in DSC. PSDSC resources are published as components of PowerShell
modules. They don't define resource manifests.

### Group resources

Group resources always operate on nested DSC Resource instances. Group resources can change how the
nested instances are processed, like the `Microsoft.DSC/Assertion` group resource.

A group resource must always define the [kind][03] property in the resource manifest.

Group resources can also be used to bundle sets of resources together for processing, like the
`Microsoft.DSC/Group` resource. You can use the [dependsOn][04] property for a resource instance in
a configuration to point to a group resource instead of enumerating each resource in the list.

### Importer resources

Importer resources resolve an external source to a set of nested DSC Resource instances. The
properties of an importer resource define how to find and resolve the external source.

An importer resource must always define the [kind][03] and [resolve][05] properties in the resource
manifest.

For example, the `Microsoft.DSC/Import` importer resource resolves instances from an external
configuration document, enabling you to compose configurations from multiple files.

### Exporter resources

Exporter resources implement the **Export** operation to return full resource instances for DSC to
recursively export. This allows an exporter resource to help users quickly export the current
configuration of a system without having to know every available resource.

For example, an exporter resource might discover whether Apache is installed and then return an
instance for every supported resource to fully export the configuration for Apache.

An exporter resource must always define the [kind][03] and [export][06] properties in the resource
manifest.

### Nested resource instances

The resource instances declared in adapter and group resources or resolved by importer resources
are called _nested resource instances_.

For nested instances, a resource instance is _adjacent_ if:

- It's declared in the same group or adapter instance.
- It's resolved by the same importer instance.

A resource instance is _external_ to a nested instance if:

- It's declared outside of the group or adapter instance
- It's resolved by a different importer instance
- It's nested inside an adjacent group, adapter, or importer instance.

For top-level instances, other instances at the top-level are adjacent. All other instances are
external.

Consider the following configuration snippet. It defines seven resource instances:

- At the top-level, the configuration defines the `TopLevelEcho`, `TopLevelOSInfo`, and
  `TopLevelGroup` instances.
- The `TopLevelGroup` instance defines the nested instances `NestedEcho` and `NestedGroup`.
- The `NestedGroup` instance defines the nested instances `DeeplyNestedEcho` and
  `DeeplyNestedOSInfo`.

```yaml
resources:
- name: TopLevelEcho
  type: Microsoft.DSC.Debug/Echo
  properties: { output: 'top level instance' }
- name: TopLevelOSInfo
  type: Microsoft/OSInfo
  properties: { }
- name: TopLevelGroup
  type: Microsoft.DSC/Group
  properties:
    $schema:
    resources:
    - name: NestedEcho
      type: Microsoft.DSC.Debug/Echo
      properties: { output: 'nested instance' }
    - name: NestedGroup
      type: Microsoft.DSC/Group
      properties:
        $schema:
        resources:
        - name: DeeplyNestedEcho
          type: Microsoft.DSC.Debug/Echo
          properties: { output: 'deeply nested instance' }
        - name: DeeplyNestedOSInfo
          type: Microsoft/OSInfo
          properties: { }
```

The following matrix defines the relations of each instance in the configuration:

|                        | TopLevelEcho   | TopLevelOSInfo | TopLevelGroup | NestedEcho | NestedGroup | DeeplyNestedEcho | DeeplyNestedOSInfo |
|------------------------|----------------|----------------|---------------|------------|-------------|------------------|--------------------|
| **TopLevelEcho**       | Self           | Adjacent       | Adjacent      | External   | External    | External         | External           |
| **TopLevelOSInfo**     | Adjacent       | Self           | Adjacent      | External   | External    | External         | External           |
| **TopLevelGroup**      | Adjacent       | Adjacent       | Self          | External   | External    | External         | External           |
| **NestedEcho**         | External       | External       | External      | Self       | Adjacent    | External         | External           |
| **NestedGroup**        | External       | External       | External      | Adjacent   | Self        | External         | External           |
| **DeeplyNestedEcho**   | External       | External       | External      | External   | External    | Self             | Adjacent           |
| **DeeplyNestedOSInfo** | External       | External       | External      | External   | External    | Adjacent         | Self               |

### Referencing nested instances

Nested resource instances have limitations for the [dependsOn][04] property and the
[reference()][07] configuration function.

1. You can only reference adjacent instances. You can't reference a nested instance from outside of
   the instance that declares or resolves it. You can't use a reference to a resource outside of the
   group, adapter, or importer resource for a nested instance.
1. You can only use the `dependsOn` property for adjacent instances. You must add a dependency on
   the group, adapter, or importer instance, not a nested instance. Nested instances can't depend
   on external instances.

The following examples show valid and invalid references and dependencies. The examples use the
`Microsoft.DSC/Group` resource, but the functionality is the same for adapter and import resources.

#### Example 1 - Valid references and dependencies

This example configuration defines several valid references and dependencies. It also defines two
instances of the `Microsoft.DSC/Group` resource, one nested inside the other.

The top level instance of the `Microsoft.DSC.Debug/Echo` resource references and depends on the top-level instance
of the `Microsoft/OSInfo` resource. The top-level instances of the `Microsoft.DSC.Debug/Echo` and
`Microsoft/OSInfo` resources both depend on the top-level instance of the `Microsoft.DSC/Group`
resource.

```yaml
# yaml-language-server: $schema=https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.vscode.json
resources:
# The top level echo references and depends on the top-level OSInfo.
# It also depends on the top-level Group.
- name: Top level echo
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:  >-
      [reference(
        resourceId('Microsoft/OSInfo', 'Top level OSInfo')
      ).actualState]
  dependsOn:
    - "[resourceId('Microsoft/OSInfo', 'Top level OSInfo')]"
    - "[resourceId('Microsoft.DSC/Group', 'Top level group')]"
# The top level OSInfo depends on the top-level Group.
- name: Top level OSInfo
  type: Microsoft/OSInfo
  properties: {}
  dependsOn:
    - "[resourceId('Microsoft.DSC/Group', 'Top level group')]"
- name: Top level group
  type: Microsoft.DSC/Group
  properties: # snipped for brevity
```

The top-level instance of `Microsoft.DSC/Group` defines three nested resource instances:
`Microsoft.DSC.Debug/Echo`, `Microsoft/OSInfo`, and `Microsoft.DSC/Group`. As at the top-level, the `Microsoft.DSC.Debug/Echo`
instance references and depends on the adjacent nested`Microsoft/OSInfo` instance and that instance
depends on the adjacent nested `Microsoft.DSC/Group` instance.

```yaml
# Other top-level instances snipped for brevity
- name: Top level group
  type: Microsoft.DSC/Group
  properties:
    $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
    resources:
    # The nested echo references and depends on the adjacent nested OSInfo.
    - name: Nested echo
      type: Microsoft.DSC.Debug/Echo
      properties:
        output:  >-
          [reference(
            resourceId('Microsoft/OSInfo', 'Nested OSInfo')
          ).actualState]
      dependsOn:
        - "[resourceId('Microsoft/OSInfo', 'Nested OSInfo')]"
    # The nested OSInfo depends on the adjacent nested Group.
    - name: Nested OSInfo
      type: Microsoft/OSInfo
      properties: {}
    - name: Nested Group
      type: Microsoft.DSC/Group
      properties: # snipped for brevity
```

Finally, the nested instance of `Microsoft.DSC/Group` defines two nested instances. The deeply
nested instance of `Microsoft.DSC.Debug/Echo` references and depends on the deeply nested instance of
`Microsoft/OSInfo`.

```yaml
- name: Top level group
  type: Microsoft.DSC/Group
  properties:
    $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
    resources:
    # Snipped the Microsoft.DSC.Debug/Echo and Microsoft/OSInfo instances for brevity
    - name: Nested Group
      type: Microsoft.DSC/Group
      properties:
        $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
        resources:
        # The deeply nested echo references and depends on the adjacent
        # deeply nested OSInfo.
        - name: Deeply nested echo
          type: Microsoft.DSC.Debug/Echo
          properties:
            output:  >-
              [reference(
                resourceId('Microsoft/OSInfo', 'Deeply nested OSInfo')
              ).actualState]
          dependsOn:
            - "[resourceId('Microsoft/OSInfo', 'Deeply nested OSInfo')]"
        - name: Deeply nested OSInfo
          type: Microsoft.OSInfo
          properties: {}
```

In every case, the references and dependencies are to adjacent instances in the configuration.
Instances at the top level only depend on or reference other instances at the top level. Instances
nested in the top-level group only depend on or reference other nested instances in the same group.
The deeply nested instances defined in the nested group only depend on or reference other deeply
nested instances in the same group.

Putting the configuration together, you get this full document:

```yaml
# yaml-language-server: $schema=https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.vscode.json
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
# The top level echo references and depends on the top-level OSInfo.
- name: Top level echo
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:  >-
      [reference(
        resourceId('Microsoft/OSInfo', 'Top level OSInfo')
      ).actualState]
  dependsOn:
    - "[resourceId('Microsoft/OSInfo', 'Top level OSInfo')]"
# The top level OSInfo depends on the top-level Group.
- name: Top level OSInfo
  type: Microsoft/OSInfo
  properties: {}
  dependsOn:
    - "[resourceId('Microsoft.DSC/Group', 'Top level group')]"
- name: Top level group
  type: Microsoft.DSC/Group
  properties:
    $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
    resources:
    # The nested echo references and depends on the adjacent nested OSInfo.
    - name: Nested echo
      type: Microsoft.DSC.Debug/Echo
      properties:
        output:  >-
          [reference(
            resourceId('Microsoft/OSInfo', 'Nested OSInfo')
          ).actualState]
      dependsOn:
        - "[resourceId('Microsoft/OSInfo', 'Nested OSInfo')]"
    # The nested OSInfo depends on the adjacent nested Group.
    - name: Nested OSInfo
      type: Microsoft/OSInfo
      properties: {}
    - name: Nested Group
      type: Microsoft.DSC/Group
      properties:
        $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
        resources:
        # The deeply nested echo references and depends on the adjacent
        # deeply nested OSInfo.
        - name: Deeply nested echo
          type: Microsoft.DSC.Debug/Echo
          properties:
            output:  >-
              [reference(
                resourceId('Microsoft/OSInfo', 'Deeply nested OSInfo')
              ).actualState]
          dependsOn:
            - "[resourceId('Microsoft/OSInfo', 'Deeply nested OSInfo')]"
        - name: Deeply nested OSInfo
          type: Microsoft.OSInfo
          properties: {}
```

#### Example 2 - Invalid reference and dependency on a nested instance

This example configuration is invalid, because the top-level instance of the `Microsoft.DSC.Debug/Echo` resource
references and depends on the nested `Microsoft/OSInfo` instance. The nested instance is external
to the top-level instance, not adjacent.

```yaml
# yaml-language-server: $schema=https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.vscode.json
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Top level echo
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:  >-
      [reference(
        resourceId('Microsoft/OSInfo', 'Nested OSInfo')
      ).actualState]
  dependsOn:
    - "[resourceId('Microsoft/OSInfo', 'Nested OSInfo')]"
- name: Top level group
  type: Microsoft.DSC/Group
  properties:
    $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
    resources:
    - name: Nested OSInfo
      type: Microsoft/OSInfo
      properties: {}
```

#### Example 3 - Invalid reference and dependency on an external instance

This example configuration is invalid, because the nested instance of the `Microsoft.DSC.Debug/Echo` resource
references and depends on the top-level `Microsoft/OSInfo` instance. The top-level instance is
external to the nested instance, not adjacent.

```yaml
# yaml-language-server: $schema=https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.vscode.json
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Top level OSInfo
  type: Microsoft/OSInfo
  properties: {}
- name: Top level group
  type: Microsoft.DSC/Group
  properties:
    $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
    resources:
    - name: Nested echo
      type: Microsoft.DSC.Debug/Echo
      properties:
        output:  >-
          [reference(
            resourceId('Microsoft/OSInfo', 'Top level OSInfo')
          ).actualState]
      dependsOn:
        - "[resourceId('Microsoft/OSInfo', 'Top level OSInfo')]"
```

[01]: ../resource/manifest/adapter.md
[02]: ../resource/manifest/validate.md
[03]: ../resource/manifest/root.md#kind
[04]: ../config/resource.md#dependson
[05]: ../resource/manifest/resolve.md
[06]: ../resource/manifest/export.md
[07]: ../config/functions/reference.md
