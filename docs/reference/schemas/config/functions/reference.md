---
description: Reference for the 'reference' DSC configuration document function
ms.date:     02/28/2025
ms.topic:    reference
title:       reference
---

# reference

## Synopsis

Returns the output for a resource instance to use with another instance.

## Syntax

```Syntax
reference(resourceId('<resourceTypeName>', '<instanceName>'))
```

## Description

The `reference` function uses the output of the [resourceId()][01] function to return the operation
result for a resource instance. You can then access the result data to use for another resource
instance.

This function enables you to set properties for later resource instances on the output results from
earlier instances. The instances don't need to be of the same type.

> [!IMPORTANT]
> When you use the `reference()` function, always ensure that any referenced resource instance is
> also included in the [dependsOn][02] property for the instance using the references. Otherwise,
> there's no guarantee that the referenced instance will be resolved before the referencing
> instance. If DSC hasn't already operated on the referenced instance, DSC will throw an error when
> it encounters the reference.

## Examples

### Example 1 - Referencing a top-level instance

In this example configuration, the `Test/Echo` resource instance echoes the `bitness` property of
the `Microsoft/OSInfo` resource. It uses the `reference()` function to retrieve the actual state of
the resource and uses the dot-path notation to access the **bitness** property of that resource.

```yaml
# reference.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: System
  type: Microsoft/OSInfo
  properties: {}
- name: Echo bitness
  type: Test/Echo
  properties:
    output: "[reference(resourceId('Microsoft/OSInfo', 'System')).actualState.bitness]"
  dependsOn:
  - "[resourceId('Microsoft/OSInfo', 'System')]"
```

```bash
dsc config get --document reference.example.1.dsc.config.yaml config get
```

```yaml
results:
- name: System
  type: Microsoft/OSInfo
  result:
    actualState:
      $id: https://developer.microsoft.com/json-schemas/dsc/os_info/20230303/Microsoft.Dsc.OS_Info.schema.json
      family: Windows
      version: 10.0.22631
      edition: Windows 11 Enterprise
      bitness: '64'
- name: Echo bitness
  type: Test/Echo
  result:
    actualState:
      output: '64'
messages: []
hadErrors: false
```

## Parameters

### resourceId

The `reference()` function expects the resource ID of the instance to resolve as a reference. Use
the [resourceId][01] function to build the resource ID for the instance instead of constructing the
resource ID manually. The function verifies that the resource instance exists, while specifying
only the string manually may lead to a harder to diagnose error if the referenced instance name is
incorrect.

```yaml
Type:         string
Required:     true
MinimumCount: 1
MaximumCount: 1
```

## Output

The `reference()` function returns the result for the configuration operation on the referenced
instance.

When referencing a group or adapter instance, the output is an array of one of the following,
matching the current operation:

- [Full get result][03]
- [Full test result][04]
- [Full set result][05]

> [!NOTE]
> When you reference a group or adapter instance, the output is always an array of full get, test,
> or set results matching the current operation. DSC doesn't currently have functions for indexing
> into arrays, so the data for a specific nested instance can't be easily retrieved.
>
> You can't reference nested instances with the `resourceId()` from outside of the group or adapter
> instance they are nested in. If you need to reference a nested instance, make sure to reference
> the instance from an adjacent resource in the same group or adapter.

When referencing a resource instance, the output is one of the following, matching the current
operation:

- [Simple get response][06]
- [Simple test response][07]
- [Simple set response][08]

You can use dot-notation to access the properties of the referenced instance, as in
[Example 1](#example-1---referencing-a-top-level-instance).

> [!NOTE]
> You can't reuse references to `actualState` for `get` and `test` operations in the `set`
> operation. You'll need to reference `afterState` in a `set` operation to get the output state
> for an instance. This requires you to maintain two separate configuration documents when you
> want to use a reference for all three operations.

```yaml
Type: [Object, Array]
```

<!-- Link reference definitions -->
[01]: ./resourceId.md
[02]: ../resource.md#dependson
[03]: ../../outputs/resource/get.md#full-get-result
[04]: ../../outputs/resource/test.md#full-test-result
[05]: ../../outputs/resource/set.md#full-set-result
[06]: ../../outputs/resource/get.md#simple-get-response
[07]: ../../outputs/resource/test.md#simple-test-response
[08]: ../../outputs/resource/set.md#simple-set-response
