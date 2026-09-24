---
description: JSON schema reference for a Desired State Configuration document.
ms.date:     09/01/2026
ms.topic:    reference
title:       DSC Configuration document schema reference
---

# DSC Configuration document schema reference

## Synopsis

The YAML or JSON file that defines a DSC Configuration.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/config/document.json
Type:          object
```

## Description

DSC Configurations enable users to define state by combining different DSC Resources. A
configuration document uses parameters and variables to pass to a set of one or more resources that
define a desired state.

A configuration document can be defined as either YAML or JSON. For ease of authoring, Microsoft
recommends drafting configuration documents in YAML.

For DSC's authoring tools to recognize a file as a DSC Configuration document, the filename must
end with one of the following:

- `.dsc.config.json`
- `.dsc.config.yml`
- `.dsc.config.yaml`.
- `.dsc.json`
- `.dsc.yml`
- `.dsc.yaml`

You can use configuration document functions to dynamically determine values in the document at
runtime. For more information, see [DSC Configuration document functions reference][01]

<!-- For more information, see [DSC Configurations overview][01]. -->

The rest of this document describes the schema DSC uses to validation configuration documents.

## Examples

<!-- To-Do -->

## Required Properties

Every configuration document must include these properties:

- [$schema](#schema)
- [resources](#resources)

## Properties

### $schema

The `$schema` property indicates the URI that resolves to the version of this schema that the
document adheres to. DSC uses this property when validating and processing the configuration
document.

The JSON schemas for DSC are published in multiple versions and forms. This documentation is for
the latest version of the schema. As a convenience, you can specify either the full URI for the
schema hosted in GitHub or use the shorter `aka.ms` URI. You can specify the schema for a specific
semantic version, the latest schema for a minor version, or the latest schema for a major version
of DSC. For more information about schema URIs and versioning, see
[DSC JSON Schema URIs](../schema-uris.md).

For every version of the schema, there are three valid URLs:

- `.../config/document.json`

  The URL to the canonical non-bundled schema. When it's used for validation, the validating client
  needs to retrieve this schema and every schema it references.

- `.../bundled/config/document.json`

  The URL to the canonically bundled schema. When it's used for validation, the validating client
  only needs to retrieve this schema.

  This schema uses the bundling model introduced for JSON Schema 2020-12. While DSC can still
  validate the document when it uses this schema, other tools may error or behave in unexpected
  ways if they don't fully support the 2020-12 specification.

- `.../bundled/config/document.vscode.json`

  The URL to the enhanced authoring schema. This schema is much larger than the other schemas, as
  it includes additional definitions that provide contextual help and snippets that the others
  don't include.

  This schema uses keywords that are only recognized by VS Code. While DSC can still validate the
  document when it uses this schema, other tools may error or behave in unexpected ways.

```yaml
Type:        string
Required:    true
Format:      URI
ValidValues: [
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/bundled/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/bundled/config/document.vscode.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2/bundled/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2/bundled/config/document.vscode.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.3/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.3/bundled/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.3/bundled/config/document.vscode.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.2/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.2/bundled/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.2/bundled/config/document.vscode.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.1/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.1/bundled/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.1/bundled/config/document.vscode.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.0/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.0/bundled/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.2.0/bundled/config/document.vscode.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1/bundled/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1/bundled/config/document.vscode.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.3/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.3/bundled/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.3/bundled/config/document.vscode.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.2/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.2/bundled/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.2/bundled/config/document.vscode.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.1/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.1/bundled/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.1/bundled/config/document.vscode.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/bundled/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/bundled/config/document.vscode.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/bundled/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/bundled/config/document.vscode.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.2/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.2/bundled/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.2/bundled/config/document.vscode.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.1/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.1/bundled/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.1/bundled/config/document.vscode.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/bundled/config/document.json
               https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/bundled/config/document.vscode.json
               https://aka.ms/dsc/schemas/v3/config/document.json
               https://aka.ms/dsc/schemas/v3/bundled/config/document.json
               https://aka.ms/dsc/schemas/v3/bundled/config/document.vscode.json
               https://aka.ms/dsc/schemas/v3.2/config/document.json
               https://aka.ms/dsc/schemas/v3.2/bundled/config/document.json
               https://aka.ms/dsc/schemas/v3.2/bundled/config/document.vscode.json
               https://aka.ms/dsc/schemas/v3.2.3/config/document.json
               https://aka.ms/dsc/schemas/v3.2.3/bundled/config/document.json
               https://aka.ms/dsc/schemas/v3.2.3/bundled/config/document.vscode.json
               https://aka.ms/dsc/schemas/v3.2.2/config/document.json
               https://aka.ms/dsc/schemas/v3.2.2/bundled/config/document.json
               https://aka.ms/dsc/schemas/v3.2.2/bundled/config/document.vscode.json
               https://aka.ms/dsc/schemas/v3.2.1/config/document.json
               https://aka.ms/dsc/schemas/v3.2.1/bundled/config/document.json
               https://aka.ms/dsc/schemas/v3.2.1/bundled/config/document.vscode.json
               https://aka.ms/dsc/schemas/v3.2.0/config/document.json
               https://aka.ms/dsc/schemas/v3.2.0/bundled/config/document.json
               https://aka.ms/dsc/schemas/v3.2.0/bundled/config/document.vscode.json
               https://aka.ms/dsc/schemas/v3.1/config/document.json
               https://aka.ms/dsc/schemas/v3.1/bundled/config/document.json
               https://aka.ms/dsc/schemas/v3.1/bundled/config/document.vscode.json
               https://aka.ms/dsc/schemas/v3.1.3/config/document.json
               https://aka.ms/dsc/schemas/v3.1.3/bundled/config/document.json
               https://aka.ms/dsc/schemas/v3.1.3/bundled/config/document.vscode.json
               https://aka.ms/dsc/schemas/v3.1.2/config/document.json
               https://aka.ms/dsc/schemas/v3.1.2/bundled/config/document.json
               https://aka.ms/dsc/schemas/v3.1.2/bundled/config/document.vscode.json
               https://aka.ms/dsc/schemas/v3.1.1/config/document.json
               https://aka.ms/dsc/schemas/v3.1.1/bundled/config/document.json
               https://aka.ms/dsc/schemas/v3.1.1/bundled/config/document.vscode.json
               https://aka.ms/dsc/schemas/v3.1.0/config/document.json
               https://aka.ms/dsc/schemas/v3.1.0/bundled/config/document.json
               https://aka.ms/dsc/schemas/v3.1.0/bundled/config/document.vscode.json
               https://aka.ms/dsc/schemas/v3.0/config/document.json
               https://aka.ms/dsc/schemas/v3.0/bundled/config/document.json
               https://aka.ms/dsc/schemas/v3.0/bundled/config/document.vscode.json
               https://aka.ms/dsc/schemas/v3.0.2/config/document.json
               https://aka.ms/dsc/schemas/v3.0.2/bundled/config/document.json
               https://aka.ms/dsc/schemas/v3.0.2/bundled/config/document.vscode.json
               https://aka.ms/dsc/schemas/v3.0.1/config/document.json
               https://aka.ms/dsc/schemas/v3.0.1/bundled/config/document.json
               https://aka.ms/dsc/schemas/v3.0.1/bundled/config/document.vscode.json
               https://aka.ms/dsc/schemas/v3.0.0/config/document.json
               https://aka.ms/dsc/schemas/v3.0.0/bundled/config/document.json
               https://aka.ms/dsc/schemas/v3.0.0/bundled/config/document.vscode.json
             ]
```

### contentVersion

The `contentVersion` property defines a version string for the configuration document. You can use
this property to track revisions of the document. DSC doesn't validate or use this value when
processing a configuration document. The document that the `dsc config export` command returns
always defines this property as `1.0.0`.

```yaml
Type:     string
Required: false
```

### directives

The `directives` property defines how DSC processes the configuration document as a whole. Every
directive is optional.

```yaml
Type:     object
Required: false
```

You can define the following directives for a configuration document:

#### resourceDiscovery

The `resourceDiscovery` directive controls when DSC raises an error for a resource that it can't
find. When you don't define this directive or set it to `preDeployment`, DSC discovers resources
and extensions before invoking any resource instance and raises an error if any instance in the
document uses a resource that DSC didn't discover.

Set this directive to `duringDeployment` to defer this check until DSC processes each instance.
This is useful when the configuration document itself installs a resource that a later instance
depends on. With this setting, DSC performs discovery again when it processes an instance whose
resource wasn't initially discovered and only raises an error if the resource isn't available at
that time.

```yaml
Type:        string
Required:    false
Default:     preDeployment
ValidValues: [preDeployment, duringDeployment]
```

#### securityContext

The `securityContext` directive defines the security context the configuration document requires.
Before invoking any resource instances, DSC validates that it's running in the required security
context and raises an error if it isn't:

- `current` - DSC can process the document in any security context. This is the default.
- `elevated` - DSC must be running as `root` (non-Windows) or in an elevated session with
  Administrator privileges (Windows).
- `restricted` - DSC must be running as a normal user or account in a non-elevated session.

This directive replaces the deprecated `Microsoft.DSC.securityContext` property in the document's
[metadata][02]. If you define both, the values must match or DSC raises an error. A resource
instance can override this directive with its own `directives.securityContext` setting.

```yaml
Type:        string
Required:    false
Default:     current
ValidValues: [current, elevated, restricted]
```

#### version

The `version` directive defines a semantic version requirement for DSC itself. When you define this
directive, DSC compares its own version to the requirement before invoking any resource instances
and raises an error if its version doesn't satisfy the requirement. This enables you to prevent a
configuration document from being processed by an incompatible version of DSC.

The value must be one or more comparators separated by commas. Each comparator is an operator
(`=`, `>`, `>=`, `<`, `<=`, `^`, or `~`) followed by a version, like `>=3.2.0, <4.0.0`. Build
metadata isn't allowed in the version. The syntax is the same as for the `requireVersion` property
of a resource instance. For more information, see [requireVersion][03].

```yaml
Type:     string
Required: false
```

### executionInformation

The `executionInformation` property describes the DSC operation that produced a configuration
document. DSC adds this property to the document returned by the `dsc config export` command. The
schema accepts this property in any configuration document, but DSC ignores it when it processes
the document.

The value is an object with the same properties as the [Microsoft.DSC metadata][04] object that DSC
returns in command output, plus an optional `whatIf` property that describes any what-if
operations DSC performed.

```yaml
Type:     object
Required: false
```

### functions

The `functions` property defines user-defined functions that you can call in configuration
expressions anywhere in the document. Each item in the list defines a namespace and the functions
that belong to it:

- `namespace` - Required. The name that groups the functions. You call a user-defined function as
  `<namespace>.<name>()`.
- `members` - Required. An object that maps each function name to its definition. Each definition
  is an object with the following properties:

  - `parameters` - Optional. A list of parameters for the function. Each parameter is an object
    that defines the `name` and `type` of the parameter. The `type` must be one of the
    [parameter data types][05]. When you call the function, DSC raises an error if the number of
    arguments or their types don't match the parameters.
  - `output` - Required. An object that defines the `type` of the value the function returns and
    the `value` as a string. DSC evaluates the `value` as a configuration expression and raises an
    error if the result doesn't match the declared `type`.

In the `value` expression, use the [parameters()][06] function to access the function's own
parameters. User-defined functions can't access the document's parameters or variables, can't use
the `reference()` function, and can't call other user-defined functions.

For example, this document defines the `contoso.greet()` function and calls it in a resource
instance:

```yaml
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

```yaml
Type:      array
Required:  false
ItemsType: object
```

### metadata

The `metadata` property defines a set of key-value pairs as annotations for the configuration.
Except for the `Microsoft.DSC` property, DSC doesn't validate the metadata. A configuration can
include any arbitrary information in this property.

The `Microsoft.DSC` property is reserved for DSC. For more information, see
[DSC Configuration document metadata schema][02].

```yaml
Type:     object
Required: false
```

### outputs

The `outputs` property defines values that DSC evaluates after it processes every resource instance
in the document and returns in the `outputs` property of the command output. Each output is defined
as a key-value pair. The key is the name of the output. The value is an object with the following
properties:

- `type` - Required. The [data type][05] of the output value. DSC raises an error if the evaluated
  value doesn't match this type. DSC doesn't return outputs with the `secureString` or
  `secureObject` types. Instead, it raises a warning and skips them.
- `value` - Required. A string that DSC evaluates as a configuration expression. Use this property
  to return data from resource results, parameters, variables, and functions, like
  `"[reference(resourceId('Microsoft.DSC.Debug/Echo', 'echo')).output]"`.
- `condition` - Optional. A string that DSC evaluates as a configuration expression. DSC only
  returns the output when the condition evaluates to `true`. When it evaluates to any other value,
  DSC skips the output.

The schema also accepts a `copy` object in place of `value`, with the same shape as the `copy`
property of a resource instance. DSC doesn't currently support copy loops for outputs. When an
output defines `copy` instead of `value`, DSC raises a warning and skips the output.

DSC doesn't evaluate outputs when you invoke the `dsc config set` command with the `--what-if`
option.

For example, this document returns the value that the `echo` instance reported:

```yaml
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

```yaml
Type:     object
Required: false
```

### parameters

The `parameters` property defines a set of runtime options for the configuration. Each parameter is
defined as key-value pair. The key for each pair defines the name of the parameter. The value for
each pair must be an object that defines the `type` keyword to indicate how DSC should process the
parameter.

Parameters may be overridden at runtime, enabling re-use of the same configuration document for
different contexts.

For more information about defining parameters in a configuration, see
[DSC Configuration document parameter schema][07].

<!-- For more information about using parameters in a configuration, see
[DSC Configuration parameters][08] -->

```yaml
Type:                object
Required:            false
ValidPropertySchema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/config/document.parameter.json
```

### resources

The `resources` property defines a list of DSC Resource instances that the configuration manages.
Instances may share the same DSC Resource type, but every instance must have a unique combination
of `type` and `name`. If two instances share the same type and name, DSC raises an error.

For more information about defining a valid resource instance in a configuration, see
[DSC Configuration document resource schema][09].

<!-- For more information about how DSC uses resources in a configuration, see
[DSC Configuration resources][10] and [DSC Configuration resource groups][11]. -->

```yaml
Type:            array
Required:        true
ValidItemSchema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/config/document.resource.json
```

### variables

The `variables` property defines a set of reusable values for the resources in the document as
key-value pairs. The key for each pair defines the name of the variable. Resources that reference
the variable by name can access the variable's value.

This can help reduce the amount of copied values and options for resources in the configuration,
which makes the document easier to read and maintain. Unlike parameters, variables can only be
defined in the configuration and can't be overridden at runtime.

<!-- For more information about using variables in a configuration, see
[DSC Configuration variables][12]. -->

```yaml
Type:     object
Required: false
```

<!-- Link reference definitions -->
[01]: functions/overview.md
<!-- [01]: ../../../configurations/overview.md -->
[02]: metadata.md
[03]: resource.md#requireversion
[04]: ../metadata/Microsoft.DSC/properties.md
[05]: ../definitions/parameters/dataTypes.md
[06]: functions/parameters.md
[07]: parameter.md
<!-- [08]: ../../../configurations/parameters.md -->
[09]: resource.md
<!-- [10]: ../../../configurations/resources.md -->
<!-- [11]: ../../../configurations/resource-groups.md -->
<!-- [12]: ../../../configurations/variables.md -->
