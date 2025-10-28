---
description: Reference for the 'copy' DSC configuration document resource loop
ms.date:     02/28/2025
ms.topic:    reference
title:       copy
---

# copy

## Synopsis

Defines a loop to create multiple instances of a resource.

## Syntax

```Syntax
copy:
  name: <loopName>
  count: <numberOfIterations>
```

## Description

The `copy` property enables you to create multiple instances of a resource in a
DSC configuration. This is the equivalent implementation of the copy
functionality from Azure Resource Manager (ARM) templates, but without support
for variables and properties, which will be added in future releases.

When you use `copy` on a resource, DSC creates multiple instances of that
resource based on the specified count. You can use the [`copyIndex()`][01]
function within the resource definition to access the current iteration index
and create unique names or property values for each instance.

> [!NOTE]
> The `mode` and `batchSize` properties are not currently supported and will
> result in an error if used.

## Examples

### Example 1 - Create multiple Echo resources

This example demonstrates the basic usage of `copy` to create three instances
of a Debug Echo resource.

```yaml
# copy.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: "[format('Echo-{0}', copyIndex())]"
  copy:
    name: echoLoop
    count: 3
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "Hello DSC"
```

```bash
dsc config get --file copy.example.1.dsc.config.yaml
```

```yaml
results:
- name: Echo-0
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: Hello DSC
- name: Echo-1
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: Hello DSC
- name: Echo-2
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: Hello DSC
messages: []
hadErrors: false
```

### Example 2 - Using copyIndex with offset

This example demonstrates using [`copyIndex()`][01] with an offset to start
numbering from a different value.

```yaml
# copy.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: "[format('Service-{0}', copyIndex(100))]"
  copy:
    name: serviceLoop
    count: 3
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "Service instance"
```

```bash
dsc config get --file copy.example.2.dsc.config.yaml
```

```yaml
results:
- name: Service-100
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: Service instance
- name: Service-101
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: Service instance
- name: Service-102
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: Service instance
messages: []
hadErrors: false
```

### Example 3 - Using named loop references

This example shows how to reference a specific loop by name when using
[`copyIndex()`][01].

```yaml
# copy.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: "[format('Resource-{0}', copyIndex('mainLoop'))]"
  copy:
    name: mainLoop
    count: 2
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "From main loop"
```

```bash
dsc config get --file copy.example.3.dsc.config.yaml
```

```yaml
results:
- name: Resource-0
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: From main loop
- name: Resource-1
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: From main loop
messages: []
hadErrors: false
```

### Example 4 - Using expressions for count with parameters

This example demonstrates using an expression for the `count` property, which
allows you to dynamically determine the number of resource instances to create
based on a parameter value. This is commonly used to make configurations
flexible and reusable.

```yaml
# copy.example.4.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  instanceCount:
    type: int
    defaultValue: 2
resources:
- name: "[format('Dynamic-{0}', copyIndex())]"
  copy:
    name: dynamicLoop
    count: "[parameters('instanceCount')]"
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[format('Instance {0} of {1}', copyIndex(), parameters('instanceCount'))]"
```

```bash
dsc config get --file copy.example.4.dsc.config.yaml --parameters '{"instanceCount": 4}'
```

```yaml
results:
- metadata:
    Microsoft.DSC:
      duration: PT0.2173106S
  name: Dynamic-0
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: Instance 0 of 2
- metadata:
    Microsoft.DSC:
      duration: PT0.0161486S
  name: Dynamic-1
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: Instance 1 of 2
messages: []
hadErrors: false
```

## Properties

### name

The name of the copy loop. This name can be used with the [`copyIndex()`][01]
function to reference the current iteration index of this specific loop.

```yaml
Type:     string
Required: true
```

### count

The number of iterations to perform. Must be a non-negative integer. If set to
0, no instances of the resource are created.

The `count` property accepts both literal integer values and expressions that
evaluate to an integer, such as parameter references using the
[`parameters()`][02] function. This allows you to dynamically control the
number of resource instances based on configuration parameters.

```yaml
Type:     integer
Required: true
Minimum:  0
```

### mode

> [!WARNING]
> The `mode` property is not currently supported and will result in an error
> if used.

This property is reserved for future implementation to specify whether resources
should be created serially or in parallel.

### batchSize

> [!WARNING]
> The `batchSize` property is not currently supported and will result in an
> error if used.

This property is reserved for future implementation to specify how many
resources to create in each batch when using parallel mode.

## Limitations

The current implementation has the following limitations:

- **Variables and properties**: Copy loops for variables and properties are not
  yet supported.
- **Mode control**: The `mode` property (serial/parallel) is not implemented.
- **Batch processing**: The `batchSize` property is not implemented.
- **Name expressions**: The resource name expression must evaluate to a string.

## Related Functions

- [`copyIndex()`][01] - Returns the current iteration index of a copy loop.
- [`parameters()`][02] - Returns the value of a configuration parameter.

<!-- Link reference definitions -->
[01]: ./copyIndex.md
[02]: ./parameters.md
