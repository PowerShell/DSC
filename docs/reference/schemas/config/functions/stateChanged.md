---
description: Reference for the 'stateChanged' DSC configuration document function
ms.date:     07/11/2026
ms.topic:    reference
title:       stateChanged
---

# stateChanged

## Synopsis

Returns whether a resource instance changed state during the current `set` operation.

## Syntax

```Syntax
stateChanged(resourceId('<resourceTypeName>', '<instanceName>'))
```

## Description

The `stateChanged()` function returns `true` when the referenced resource instance changed one or
more properties during the current `set` operation. Otherwise, it returns `false`.

The function can only evaluate an instance after DSC has executed it. Use [resourceId()][01] to
identify the instance and add that instance to the calling resource's [dependsOn][02] property.
When DSC evaluates `stateChanged()` for an instance that has not run or does not exist, it returns
an error.

## Examples

### Example 1 - Report whether a resource changed state

The configuration sets the **Install OpenSSH Client** instance with the
[Microsoft.Windows/FeatureOnDemandList][03] resource and then runs the **Report change** instance.
The second instance uses `stateChanged()` to report whether the feature resource changed state. Its
`dependsOn` property ensures the feature instance has completed before DSC evaluates the function.

> [!NOTE]
> This example requires Windows, an elevated session, and access to a Windows Update or WSUS
> source. The output is `true` only when installing the feature changes state.

```yaml
# stateChanged.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Install OpenSSH Client
  type: Microsoft.Windows/FeatureOnDemandList
  properties:
    capabilities:
    - identity: OpenSSH.Client~~~~0.0.1.0
      state: Installed
- name: Report change
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[stateChanged(resourceId('Microsoft.Windows/FeatureOnDemandList', 'Install OpenSSH Client'))]"
  dependsOn:
  - "[resourceId('Microsoft.Windows/FeatureOnDemandList', 'Install OpenSSH Client')]"
```

```bash
dsc config set --file stateChanged.example.1.dsc.config.yaml
```

```yaml
results:
- name: Install OpenSSH Client
  type: Microsoft.Windows/FeatureOnDemandList
  result:
    changedProperties:
    - capabilities
- name: Report change
  type: Microsoft.DSC.Debug/Echo
  result:
    afterState:
      output: true
messages: []
hadErrors: false
```

## Parameters

### resourceId

The resource ID of the instance whose state change information you want to retrieve. Use the
[resourceId()][01] function to create this value.

```yaml
Type:         string
Required:     true
MinimumCount: 1
MaximumCount: 1
```

## Output

The `stateChanged()` function returns `true` if the resource changed one or more properties in the
current `set` operation. Otherwise, it returns `false`.

```yaml
Type: bool
```

<!-- Link reference definitions -->
[01]: ./resourceId.md
[02]: ../resource.md#dependson
[03]: ../../../resources/Microsoft/Windows/FeatureOnDemandList/index.md
