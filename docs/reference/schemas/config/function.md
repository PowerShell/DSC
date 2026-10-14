---
description: JSON schema reference for a user-defined function in a Desired State Configuration document.
ms.date:     10/13/2026
ms.topic:    reference
title:       DSC Configuration document user-defined function schema
---

# DSC Configuration document user-defined function schema

## Synopsis

Defines a namespace of user-defined functions for a configuration document.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.0/config/document.function.json
Type:          object
```

## Description

The `functions` property of a DSC Configuration document defines a list of user-defined functions.
Each item in the list defines a namespace and the functions that belong to it. You can call a
user-defined function in a configuration expression anywhere in the document with the same syntax
as a built-in [configuration function][01], as `<namespace>.<name>()`.

User-defined functions enable you to give a descriptive name to an expression and reuse it in
several places in the document.

A user-defined function can only use the values of its own parameters, which it accesses with the
[parameters()][02] function. User-defined functions can't access the document's parameters or
variables, can't use the [reference()][03] function, and can't call other user-defined functions.

When you call a user-defined function, DSC validates the call before it evaluates the function:

- DSC raises an error if the document doesn't define the function.
- DSC raises an error if the number of arguments doesn't match the number of parameters.
- DSC raises an error if the data type of an argument doesn't match the data type of its parameter.

After DSC evaluates the function, it raises an error if the result doesn't match the data type that
the function's `output` property declares.

## Required Properties

Every item in the `functions` list must be an object that defines these properties:

- [namespace](#namespace)
- [members](#members)

## Properties

### namespace

The `namespace` property defines the name that groups the functions. The namespace is the prefix
you use when you call the functions, like `contoso` in `contoso.greet()`.

```yaml
Type:     string
Required: true
```

### members

The `members` property defines the functions in the namespace as key-value pairs. The key is the
name of the function. The value is an object that defines the function's parameters and output.

```yaml
Type:     object
Required: true
```

#### parameters

The `parameters` property defines the list of parameters that the function accepts, in the order
you pass them when you call the function. Each parameter is an object with the following
properties:

- `name` - Required. The name of the parameter. Use this name with the [parameters()][02] function
  in the `value` expression of the function's `output` to access the argument passed to the
  function.
- `type` - Required. The [data type][04] of the parameter.

When a function doesn't define any parameters, call it without arguments, like
`contoso.greetWorld()`.

```yaml
Type:      array
Required:  false
ItemsType: object
```

#### output

The `output` property defines the value that the function returns. The value is an object with the
following properties:

- `type` - Required. The [data type][04] of the value the function returns. DSC raises an error if
  the evaluated value doesn't match this type.
- `value` - Required. A string that DSC evaluates as a configuration expression to get the value
  the function returns.

```yaml
Type:     object
Required: true
```

## Examples

### Example 1 - Define and call a function

The following configuration document defines the `contoso.greet()` function. The function accepts
one string parameter and returns a greeting. The `Greeting` instance calls the function to set its
`output` property.

```yaml
# function.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
functions:
  - namespace: contoso
    members:
      greet:
        parameters:
          - name: name
            type: string
        output:
          type: string
          value: "[format('Hello, {0}!', parameters('name'))]"
resources:
  - name: Greeting
    type: Microsoft.DSC.Debug/Echo
    properties:
      output: "[contoso.greet('World')]"
```

```sh
dsc config get --file function.example.1.dsc.config.yaml
```

```yaml
executionInformation:
  duration: PT0.0552689S
  endDatetime: 2026-10-13T22:05:06.808177700+02:00
  executionType: actual
  operation: get
  securityContext: restricted
  startDatetime: 2026-10-13T22:05:06.752908800+02:00
  version: 3.4.0-preview.2
metadata:
  Microsoft.DSC:
    duration: PT0.0552581S
    endDatetime: 2026-10-13T22:05:06.808166900+02:00
    executionType: actual
    operation: get
    securityContext: restricted
    startDatetime: 2026-10-13T22:05:06.752908800+02:00
    version: 3.4.0-preview.2
results:
- executionInformation:
    duration: PT0.0374836S
  metadata:
    Microsoft.DSC:
      duration: PT0.0374836S
  name: Greeting
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: Hello, World!
messages: []
hadErrors: false
```

<!-- Link reference definitions -->
[01]: functions/overview.md
[02]: functions/parameters.md
[03]: functions/reference.md
[04]: ../definitions/parameters/dataTypes.md
