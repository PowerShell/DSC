---
description: >-
  Describes what a DSC resource instance is and how to use them with DSC.
ms.date:     03/25/2025
ms.topic:    conceptual
title:       DSC resource instances
---

# DSC resource instances

DSC resources manage _instances_ of a configurable component. For example, the
`Microsoft.Windows/Registry` resource manages Windows Registry keys and values. Each registry key
and value is a different instance of the resource.

Every command resource defines a [resource instance schema][01] that describes how to validate and
manage an instance of the resource with a JSON Schema. [Adapter resources][02] implement the
[Validate operation][03] to enable validating adapted resource instances, which might not have JSON
Schemas to describe their properties.

If you specify an invalid definition for a resource instance, DSC raises an error before invoking
any operations on the instance.

## Nested resource instances

The resource instances declared in [adapter resources][04] and [group resources][05] or resolved by
[importer resources][06] are called _nested resource instances_.

For nested instances, a resource instance is _adjacent_ if:

- The instance is declared in the same group or adapter instance.
- The instance is resolved from the same importer instance.

A resource instance is _external_ to a nested instance if:

- The instance is declared outside of the group or adapter instance
- The instance is resolved from a different importer instance
- The instance is nested inside an adjacent group, adapter, or importer instance.

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

Nested resource instances have limitations for the [dependsOn][07] property and the
[reference()][08] configuration function.

1. You can only reference adjacent instances. You can't reference a nested instance from outside of
   the instance that declares or resolves it. You can't use a reference to a resource outside of
   the group, adapter, or importer resource for a nested instance.
1. You can only use the `dependsOn` property for adjacent instances. You must add a dependency on
   the group, adapter, or importer instance, not a nested instance. Nested instances can't depend
   on external instances.

The following examples show valid and invalid references and dependencies. The examples use the
`Microsoft.DSC/Group` resource, but the functionality is the same for adapter and import resources.

#### Example 1 - Valid references and dependencies

This example configuration defines several valid references and dependencies. It also defines two
instances of the `Microsoft.DSC/Group` resource, one nested inside the other.

The top level instance of the `Microsoft.DSC.Debug/Echo` resource references and depends on the
top-level instance of the `Microsoft/OSInfo` resource. The top-level instances of the
`Microsoft.DSC.Debug/Echo` and `Microsoft/OSInfo` resources both depend on the top-level instance
of the `Microsoft.DSC/Group` resource.

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
      )]
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
`Microsoft.DSC.Debug/Echo`, `Microsoft/OSInfo`, and `Microsoft.DSC/Group`. As at the top-level, the
`Microsoft.DSC.Debug/Echo` instance references and depends on the adjacent nested`Microsoft/OSInfo`
instance and that instance depends on the adjacent nested `Microsoft.DSC/Group` instance.

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
          )]
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
nested instance of `Microsoft.DSC.Debug/Echo` references and depends on the deeply nested instance
of `Microsoft/OSInfo`.

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
              )]
          dependsOn:
            - "[resourceId('Microsoft/OSInfo', 'Deeply nested OSInfo')]"
        - name: Deeply nested OSInfo
          type: Microsoft/OSInfo
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
      )]
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
    - name: Nested Group
      type: Microsoft.DSC/Group
      properties:
        $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
        resources:
        - name: Deeply nested OSInfo
          type: Microsoft/OSInfo
          properties: {}
        # The deeply nested echo references and depends on the adjacent
        # deeply nested OSInfo.
        - name: Deeply nested echo
          type: Microsoft.DSC.Debug/Echo
          properties:
            output:  >-
              [reference(
                resourceId('Microsoft/OSInfo', 'Deeply nested OSInfo')
              )]
          dependsOn:
            - "[resourceId('Microsoft/OSInfo', 'Deeply nested OSInfo')]"
    # The nested echo references and depends on the adjacent nested OSInfo.
    - name: Nested echo
      type: Microsoft.DSC.Debug/Echo
      properties:
        output:  >-
          [reference(
            resourceId('Microsoft/OSInfo', 'Nested OSInfo')
          )]
      dependsOn:
        - "[resourceId('Microsoft/OSInfo', 'Nested OSInfo')]"
    # The nested OSInfo depends on the adjacent nested Group.
    - name: Nested OSInfo
      type: Microsoft/OSInfo
      properties: {}
      dependsOn:
        - "[resourceId('Microsoft.DSC/Group', 'Nested Group')]"
    
```

#### Example 2 - Invalid reference and dependency on a nested instance

This example configuration is invalid, because the top-level instance of the
`Microsoft.DSC.Debug/Echo` resource references and depends on the nested `Microsoft/OSInfo`
instance. The nested instance is external to the top-level instance, not adjacent.

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
      )]
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

This example configuration is invalid, because the nested instance of the
`Microsoft.DSC.Debug/Echo` resource references and depends on the top-level `Microsoft/OSInfo`
instance. The top-level instance is external to the nested instance, not adjacent.

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
          )]
      dependsOn:
        - "[resourceId('Microsoft/OSInfo', 'Top level OSInfo')]"
```

## See also

- [DSC resource kinds][09]
- [DSC resource operations][10]
- [DSC configuration documents][11]

<!-- Link reference definitions -->
[01]: ../../reference/schemas/resource/manifest/root.md#schema
[02]: ./kinds.md#adapter-resources
[03]: ./operations.md#validate-operation
[04]: ./kinds.md#adapter-resources
[05]: ./kinds.md#group-resources
[06]: ./kinds.md#importer-resources
[07]: ../../reference/schemas/config/resource.md#dependson
[08]: ../../reference/schemas/config/functions/reference.md
[09]: ./kinds.md
[10]: ./operations.md
[11]: ../configuration-documents/overview.md
