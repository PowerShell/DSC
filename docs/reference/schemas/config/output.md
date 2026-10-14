---
description: JSON schema reference for an output in a Desired State Configuration document.
ms.date:     10/13/2026
ms.topic:    reference
title:       DSC Configuration document output schema
---

# DSC Configuration document output schema

## Synopsis

Defines a value that DSC returns after it processes a configuration document.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.0/config/document.output.json
Type:          object
```

## Description

The `outputs` property of a DSC Configuration document defines values that DSC evaluates after it
processes every resource instance in the document. DSC returns the evaluated values in the
`outputs` property of the command output for the `dsc config get`, `dsc config set`,
`dsc config test`, and `dsc config export` commands. For more information about the command output,
see [dsc config get result schema reference][01].

Outputs are defined as key-value pairs in the `outputs` property of the configuration document. The
key is the name of the output. DSC uses the name as the key for the value in the command output.
The value is an object that defines the data type of the output and the expression that DSC
evaluates to get the value.

Use outputs to return data from resource results, parameters, variables, and functions without
inspecting the result for each resource instance. For example, an output can return a property
that a resource reported in its actual state with the [reference()][02] function, or report
whether any resource requires a restart with the [restartRequired()][03] function.

DSC evaluates outputs after it processes every resource instance in the document. DSC doesn't
evaluate outputs when you invoke the `dsc config set` command with the `--what-if` option.

## Required Properties

Every output must be an object that defines these properties:

- [type](#type)
- [value](#value)

## Properties

### condition

The `condition` property defines an expression that DSC evaluates before it evaluates the output.
DSC only returns the output when the condition evaluates to `true`. When the condition evaluates to
any other value, DSC skips the output and doesn't include it in the command output.

```yaml
Type:     string
Required: false
```

### type

The `type` property defines the [data type][04] of the output value. After DSC evaluates the
`value` expression, it validates that the result matches this type and raises an error if it
doesn't.

DSC doesn't return outputs with the `secureString` or `secureObject` types. When an output defines
one of those types, DSC raises a warning and skips the output.

```yaml
Type:        string
Required:    true
ValidValues: [string, secureString, int, bool, object, secureObject, array]
```

### value

The `value` property defines the expression that DSC evaluates to get the value of the output. DSC
evaluates the expression after it processes every resource instance in the document, so the
expression can use the results of any instance.

```yaml
Type:     string
Required: true
```

### copy

The schema accepts a `copy` object in place of the `value` property. The object has the same
properties as the [copy][05] property of a resource instance. DSC doesn't currently support copy
loops for outputs. When an output defines `copy` instead of `value`, DSC raises a warning and skips
the output.

```yaml
Type:     object
Required: false
```

## Examples

### Example 1 - Return a value from a resource result

The following configuration document defines the `echoOutput` output. The output returns the value
of the `output` property that the `echo` instance reports in its actual state.

```yaml
# output.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: echo
    type: Microsoft.DSC.Debug/Echo
    properties:
      output: Hello World
outputs:
  echoOutput:
    type: string
    value: "[reference(resourceId('Microsoft.DSC.Debug/Echo', 'echo')).output]"
```

```sh
dsc config get --file output.example.1.dsc.config.yaml
```

DSC returns the evaluated output in the `outputs` property of the command output:

```yaml
executionInformation:
  duration: PT0.3540948S
  endDatetime: 2026-10-13T22:05:06.210996600+02:00
  executionType: actual
  operation: get
  securityContext: restricted
  startDatetime: 2026-10-13T22:05:05.856901800+02:00
  version: 3.4.0-preview.2
metadata:
  Microsoft.DSC:
    duration: PT0.3540803S
    endDatetime: 2026-10-13T22:05:06.210982100+02:00
    executionType: actual
    operation: get
    securityContext: restricted
    startDatetime: 2026-10-13T22:05:05.856901800+02:00
    version: 3.4.0-preview.2
results:
- executionInformation:
    duration: PT0.316169S
  metadata:
    Microsoft.DSC:
      duration: PT0.316169S
  name: echo
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: Hello World
messages: []
hadErrors: false
outputs:
  echoOutput: Hello World
```

### Example 2 - Return an output conditionally

The following configuration document defines two outputs. The `hostName` output always returns the
`hostName` property that the `echo` instance reports. The `details` output returns the entire
`output` property of the instance, but only when the `includeDetails` parameter is `true`.

```yaml
# output.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  includeDetails:
    type: bool
    defaultValue: false
resources:
  - name: echo
    type: Microsoft.DSC.Debug/Echo
    properties:
      output:
        hostName: webserver01
        port: 8080
outputs:
  hostName:
    type: string
    value: "[reference(resourceId('Microsoft.DSC.Debug/Echo', 'echo')).output.hostName]"
  details:
    type: object
    condition: "[parameters('includeDetails')]"
    value: "[reference(resourceId('Microsoft.DSC.Debug/Echo', 'echo')).output]"
```

When you invoke DSC without overriding the parameter, the condition for the `details` output
evaluates to `false` and DSC only returns the `hostName` output:

```sh
dsc config get --file output.example.2.dsc.config.yaml
```

```yaml
executionInformation:
  duration: PT0.0530961S
  endDatetime: 2026-10-13T22:05:06.416749900+02:00
  executionType: actual
  operation: get
  securityContext: restricted
  startDatetime: 2026-10-13T22:05:06.363653800+02:00
  version: 3.4.0-preview.2
metadata:
  Microsoft.DSC:
    duration: PT0.0530842S
    endDatetime: 2026-10-13T22:05:06.416738+02:00
    executionType: actual
    operation: get
    securityContext: restricted
    startDatetime: 2026-10-13T22:05:06.363653800+02:00
    version: 3.4.0-preview.2
results:
- executionInformation:
    duration: PT0.0399163S
  metadata:
    Microsoft.DSC:
      duration: PT0.0399163S
  name: echo
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        hostName: webserver01
        port: 8080
messages: []
hadErrors: false
outputs:
  hostName: webserver01
```

When you set the `includeDetails` parameter to `true`, DSC returns both outputs:

```sh
dsc config --parameters '{ "parameters": { "includeDetails": true } }' get --file output.example.2.dsc.config.yaml
```

```yaml
executionInformation:
  duration: PT0.0547291S
  endDatetime: 2026-10-13T22:05:06.613340600+02:00
  executionType: actual
  operation: get
  securityContext: restricted
  startDatetime: 2026-10-13T22:05:06.558611500+02:00
  version: 3.4.0-preview.2
metadata:
  Microsoft.DSC:
    duration: PT0.0547168S
    endDatetime: 2026-10-13T22:05:06.613328300+02:00
    executionType: actual
    operation: get
    securityContext: restricted
    startDatetime: 2026-10-13T22:05:06.558611500+02:00
    version: 3.4.0-preview.2
results:
- executionInformation:
    duration: PT0.0408S
  metadata:
    Microsoft.DSC:
      duration: PT0.0408S
  name: echo
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        hostName: webserver01
        port: 8080
messages: []
hadErrors: false
outputs:
  details:
    hostName: webserver01
    port: 8080
  hostName: webserver01
```

<!-- Link reference definitions -->
[01]: ../outputs/config/get.md#outputs
[02]: functions/reference.md
[03]: functions/restartRequired.md
[04]: ../definitions/parameters/dataTypes.md
[05]: resource.md#copy
