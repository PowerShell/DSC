---
description: >-
  DSC configuration documents are YAML or JSON data files that define the desired state of a system
  declaratively. They're used to retrieve, validate, and enforce the state of multiple resource
  instances.
ms.date: 03/25/2025
title: DSC configuration documents
---

# DSC configuration documents

<!-- markdownlint-disable MD013 -->

In Microsoft's Desired State Configuration (DSC) platform, DSC configuration documents declare the
desired state of a system as data files. Configuration documents define a collection of
[DSC resource][01] instances to describe what the desired state should be, not how to put the
system into that state. The DSC resources handle the _how_ for each instance.

DSC can process configuration documents to:

- Retrieve the current state of the defined resource instances with the `dsc config get` command.
- Validate whether the instances are in the desired state with the `dsc config test` command.
- Enforce the desired state for the instances with the `dsc config set` command.
- Export a new configuration document with every instance of a set of resources with the
  `dsc config export` command.

Configuration documents are YAML or JSON files that contain a single object. The object's
properties define how DSC processes the document. The top-level properties for a document are:

- `$schema` (required) - Defines the URI for the JSON Schema the document adheres to. DSC
  uses this URI to know how to validate and interpret the document.
- `resources` (required) - Defines the collection of resource instances the document manages.
- `metadata` (optional) - Defines an arbitrary set of annotations for the document. Except for
  metadata within the `Microsoft.DSC` property, DSC doesn't validate this data or use it directly.
  The annotations can include notes like who authored the document, the last time someone updated
  it, or any other information. DSC doesn't use the annotations. The metadata is for documentation
  or other tools to use.

  DSC applies validation to the `Microsoft.DSC` property. For more information, see the
  [DSC Configuration document metadata schema][02] reference.
- `parameters` (optional) - Defines a set of runtime options for the configuration. Resource
  instances can reference parameters to reduce duplicate definitions or enable dynamic values.
  Parameters can have default values and can be set on any configuration operation.
- `variables` (optional) - Defines a set of reusable values for the configuration. Resource
  instances can reference variables to reduce duplicate definitions.

## Defining a configuration document

Minimally, a configuration document defines the `$schema` and `resources` properties. The
`$schema` property must be a valid URI for the DSC configuration document schema. The `resources`
property must define at least one DSC Resource instance.

For example:

```yaml
# example.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: example key value
    type: Microsoft.Windows/Registry
    properties:
      keyPath:   HKCU\example\key
      valueName: Example
      valueData:
        String: This is an example.
```

The example document defines a single resource instance named `example key value`. The instance
uses the `Microsoft.Windows/Registry` resource to declare the following desired state:

- The `example\key` registry key should exist in the system's current user hive.
- The `example\key` registry key should have a value called `Example`.
- The `Example` value should be the string `This is an example`.

The example document is _declarative_. It describes the desired state, not how to put the system
into that state. It relies on the `Microsoft.Windows/Registry` resource to handle getting, testing,
and setting the state of the registry key and value.

For more information about the structure and validation of configuration documents, see
[DSC Configuration document schema reference][03].

### Defining resource instances

As shown in the prior example, configuration documents include a collection of resource instances.
Together, the instances describe the desired state of a system. A configuration document can
include any number of resource instances.

A resource instance declaration always includes:

- `name` - A short, human-readable name for the instance that's unique in the document. This name
  is used for logging and it helps describe the purpose of the instance.
- `type` - The [fully qualified type name][04] of the resource that DSC should use to manage the
  instance.
- `properties` - The desired state for the instance. DSC validates the values against the
  resource's instance schema.

Configuration documents can't include the same instance more than once. Declaring the same instance
with different properties leads to enforcement cycling, where each declaration enforces an
incompatible state for the instance on every run.

## Getting the current state of a configuration

The `dsc config get` command retrieves the current state of the resource instances defined in a
configuration document. When you use this command, DSC also:

- Collects any message emitted by the resources during the operation.
- Indicates whether any of the resources raised an error.
- Provides metadata about the operation as a whole and for each resource instance.

```sh
dsc config get --file ./example.config.dsc.yaml
```

```yaml
metadata:
  Microsoft.DSC:
    version: 3.0.0
    operation: get
    executionType: actual
    startDatetime: 2025-02-24T16:09:40.671454400-06:00
    endDatetime: 2025-02-24T16:09:41.850086300-06:00
    duration: PT1.1786319S
    securityContext: restricted
results:
- metadata:
    Microsoft.DSC:
      duration: PT0.2751153S
  name: example key value
  type: Microsoft.Windows/Registry
  result:
    actualState:
      keyPath: HKCU\example\key
      _exist: false
messages: []
hadErrors: false
```

## Testing a configuration

The `dsc config test` command compares the current state of the resource instances to their desired
state. The result for each instance includes:

- The desired state for the instance.
- The actual state of the instance.
- Whether the instance is in the desired state.
- The list of properties that are out of the desired state, if any.

When you use this command, DSC also:

- Collects any message emitted by the resources during the operation.
- Indicates whether any of the resources raised an error.
- Provides metadata about the operation as a whole and for each resource instance.

```sh
dsc config test --file /example.config.dsc.yaml
```

```yaml
metadata:
  Microsoft.DSC:
    version: 3.0.0
    operation: test
    executionType: actual
    startDatetime: 2025-02-24T16:11:42.798122500-06:00
    endDatetime: 2025-02-24T16:11:43.442216400-06:00
    duration: PT0.6440939S
    securityContext: restricted
results:
- metadata:
    Microsoft.DSC:
      duration: PT0.2234078S
  name: example key value
  type: Microsoft.Windows/Registry
  result:
    desiredState:
      keyPath: HKCU\example\key
      valueName: Example
      valueData:
        String: This is an example.
    actualState:
      keyPath: HKCU\example\key
      _exist: false
    inDesiredState: false
    differingProperties:
    - valueName
    - valueData
    - _exist
messages: []
hadErrors: false
```

## Enforcing a configuration

The `dsc config set` command enforces the resource instances defined in a configuration document to
their desired state. The result for each instance includes:

- The state of the instance before the operation.
- The state of the instance after the operation.
- Which properties the operation changed, if any.

When you use this command, DSC also:

- Collects any message emitted by the resources during the operation.
- Indicates whether any of the resources raised an error.
- Provides metadata about the operation as a whole and for each resource instance.

```sh
dsc config set --file ./example.config.dsc.yaml
```

```yaml
metadata:
  Microsoft.DSC:
    version: 3.0.0
    operation: set
    executionType: actual
    startDatetime: 2025-02-24T16:13:32.746742600-06:00
    endDatetime: 2025-02-24T16:13:33.606785-06:00
    duration: PT0.8600424S
    securityContext: restricted
results:
- metadata:
    Microsoft.DSC:
      duration: PT0.4070001S
  name: example key value
  type: Microsoft.Windows/Registry
  result:
    beforeState:
      keyPath: HKCU\example\key
      _exist: false
    afterState:
      keyPath: HKCU\example\key
      valueName: Example
      valueData:
        String: This is an example.
    changedProperties:
    - valueName
    - valueData
    - _exist
messages: []
hadErrors: false
```

## See also

- [DSC Resources][01] to learn about resources.
- [DSC Configuration document schema reference][05]
- [Command line reference for the 'dsc config' command][06]

[01]: ../resources/overview.md
[02]: ../../reference/schemas/config/metadata.md#microsoftdsc
[03]: ../../reference/schemas/config/document.md
[04]: ../resources/overview.md#resource-type-names
[05]: ../../reference/schemas/config/document.md
[06]: ../../reference/cli/config/index.md
