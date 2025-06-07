---
title: "Glossary: Desired State Configuration"
description: >-
  A glossary of terms for Microsoft Desired State Configuration (DSC)
ms.topic: glossary
ms.date: 03/25/2025
---

# Glossary: Desired State Configuration

Microsoft Desired State Configuration (DSC) uses several terms that might have different
definitions elsewhere. This document lists the terms, their meanings, and shows how they're
formatted in the documentation.

<!-- markdownlint-disable MD028 MD036 MD024 -->

## Configuration terms

### DSC Configuration document

The JSON or YAML data that defines a list of resource instances and their desired state.

#### Guidelines

- **First mention:** DSC Configuration Document
- **Subsequent mentions:** configuration document or document

#### Examples

> A DSC Configuration Document can be formatted as JSON or YAML.

> Define the `scope` variable in the document as `machine`.

## Resource terms

### DSC Resource

The DSC interface for managing the settings of a component. DSC supports several kinds of
resources.

#### Guidelines

- **First mention:** DSC Resource
- **Subsequent mentions:** resource
- Format the names of specific resources as code.

#### Examples

> They both use the `Microsoft/OSInfo` DSC Resource.

> You can inspect a resource's definition with the `dsc resource list <resource_name>` command.

### DSC resource instance

A single item configured by a DSC resource. A resource can manage any number of instances. Each
instance uniquely represents an item, like a specific file or a software package.

A DSC configuration document defines the desired state for one or more instances.

#### Guidelines

- **First mention:** DSC resource instance
- **Subsequent mentions:** resource instance or instance

#### Examples

> Next, define the first DSC resource instance for your configuration document. Each resource
> instance configures a unique component on the machine.

### DSC command resource

A resource defined with a resource manifest is a _command_ resource. DSC uses the manifest to
determine how to invoke the resource and how to validate the resource instance properties.

#### Guidelines

- Use this term when distinguishing between command resources and adapted resources.

#### Examples

> `Microsoft.Windows/Registry` is a command resource. DSC uses the resource's resource manifest
> to determine how to invoke the `registry` executable for a DSC operation. Adapted resources,
> like those implemented in PowerShell, don't define a resource manifest. Instead, the adapter
> is responsible for discovering adapted resources, advertising their properties to DSC, and
> invoking the adapted resources for DSC.

### DSC group resource

A _group resource_ is a resource with a `resources` property that takes an array of resource
instances and processes them. Group resources can apply special handling to their nested resource
instances, like changing the user the resources run as.

#### Guidelines

- Always specify the term as group resource. Don't omit "group" from the term.

#### Examples

> To invoke resources in parallel, use the `Microsoft.DSC/ParallelGroup` group resource.

### Nested resource instance

A resource instance defined in the `resources` property of a group or adapter
resource instance.

#### Guidelines

- **First mention:** nested resource instance
- **Subsequent mentions:** nested instance
- If it's clear from context that the instance is a nested instance, you can omit the "nested"
  prefix.

#### Examples

> Add a nested resource instance to the `DSC/ParallelGroup` instance. Define the `type` of the
> nested instance as `Microsoft.Windows/Registry`.

### DSC adapter resource

An _adapter resource_ is a group resource that enables the use of noncommand resources with
DSC. Every nested resource instance must be a resource type the adapter supports.

#### Guidelines

- **First mention:** DSC adapter resource
- **Subsequent mentions:** adapter

#### Examples

> To use PowerShell DSC (PSDSC) Resources in your configuration document, add an instance of the
> `Microsoft.Dsc/PowerShell` adapter resource and define the PowerShell resource instances as
> nested instances.

### DSC PowerShell resources

A resource implemented in PowerShell is a PowerShell resource.

Any PowerShell resource that is compatible with PowerShell DSC is a PowerShell DSC resource (PSDSC
resource). The implementation of a PSDSC resource further distinguishes those resources:

- Class-Based - A PSDSC resource defined as a PowerShell class in a PowerShell module is a
  _class-based_ PSDSC resource.

  The class's members define a class-based resource's schema. A class-based PSDSC resource must:

  1. Have the `[DscResource()]` attribute.
  1. Define at least one property with the `[DscProperty()]` attribute.
  1. Define the `Get()`, `Set()`, and `Test()` methods.

- MOF-based - A PSDSC resource defined with a MOF file (`.mof`), a script module file (`.psm1`),
  and optional module manifest file (`.psd1`) is a _MOF-based_ PSDSC resource. MOF-based resources
  are only supported through Windows PowerShell and the `Microsoft.Windows/WindowsPowerShell`
  adapter.

  The MOF file is the schema for the resource and defines the resource's properties. The script
  module file defines the resource's functions: `Get-TargetResource`, `Set-TargetResource`, and
  `Test-TargetResource`. These functions map to the **Get**, **Set**, and **Test** operations.

#### Guidelines

<!-- vale alex.Condescending = NO -->

- **First mention:** PowerShell DSC resources
- **Subsequent mentions:** PSDSC resources.
- When discussing a specific type of PowerShell resource, always specify the type prefix, like
  _class-based PSDSC resources_.
- The PSDSC prefix can be omitted when the context is clearly or only about PowerShell DSC
  resources, like a tutorial for authoring a class-based resource.

<!-- vale alex.Condescending = YES -->

#### Examples

> To use PowerShell DSC Resources in your configuration document, add an instance of the
> `Microsoft.DSC/PowerShell` adapter resource and define the PSDSC resource instances as nested
> instances.

> When developing PowerShell resources for cross-platform software, create class-based resources.
> MOF-based resources are only supported through Windows PowerShell.

### DSC Resource manifest

The data file that defines the metadata and implementation of a command-based resource. A resource
manifest can be authored in either JSON or YAML.

#### Guidelines

- **First mention:** DSC resource manifest
- **Subsequent mentions:** manifest

#### Examples

> Every command resource must define a DSC resource manifest. The manifest's filename must end with
> `.dsc.resource.json`.

### DSC Resource type name

The identifying name of a resource. The fully qualified resource type name uses the following
syntax:

```text
`<owner>[.<group>][.<area>]/<name>`
```

#### Guidelines

- **First mention:** When discussing type names conceptually, use the phrase _DSC resource type
  name_. When referencing a specific resource by name, always use the fully qualified resource type
  name formatted as code.
- **Subsequent mentions:** When discussing type names conceptually, use the phrase _resource type_
  or _type name_. When referencing a specific resource by name, you can specify it as `<name>` for
  brevity.
- When discussing the syntax of a resource type name, always specify the term as
  _fully qualified resource type name_.

#### Examples

> DSC Resources are uniquely identified by their fully qualified resource type name.

> The `Microsoft.DSC/PowerShell` adapter resource enables you to use PowerShell DSC (PSDSC)
> resources with DSC. The `PowerShell` adapter handles discovering and invoking PSDSC resources.

### Operations

The actions a resource can take for the component it manages.

- **Get** - Retrieves the current state of an instance of the resource.
- **Set** - Enforces the desired state of an instance of the resource.
- **Test** - Compares the desired state of an instance of the resource against its current state.
- **Export** - Retrieves the current state of every instance of the resource.
- **Delete** - Removes a specific instance of the resource.

#### Guidelines

- Capitalize the operations.
- When referring to the operation specifically, format it as **bold**.
- When referring to the operation's method as implemented in a PowerShell class, format the method
  as `code` with an empty set of parentheses (`()`) after the name.
- When referring to the operation as the DSC command, format the method as `code` for the
  appropriate command.

#### Examples

> The implementation of the `Set()` method in a class-based PowerShell resource can't use any
> `return` statements.

> DSC is constructed around the primary operations **Get**, **Test**, and **Set**. When you use the
> `get` subcommand for `dsc resource`, it returns the current state of the resource instance.

### Resource properties

A setting that a resource can manage for a component. Resources always have at least one property.
Resources describe their properties with their [instance schema](#resource-instance-schema) in the
`properties` keyword.

Some properties are [canonical](#canonical-resource-properties), [key](#key-resource-properties),
[read-only](#read-only-resource-properties), or [write-only](#write-only-resource-properties)
properties.

#### Guidelines

- Format property names as bold.
- Format property values as code.
- Use the correct casing for the property based on the resource instance schema.

#### Examples

> The value of the **format** property in this example is `JSON`.

### Key resource properties

The key properties of a resource uniquely identify an instance of the resource. No two instances of
a resource in a configuration can have identical key properties.

If two instances have the same key properties but different values for the other properties, the
configuration will never be in the desired state and DSC will reconfigure the instance during every
**Set** operation. In a future release, specifying two or more instances of a resource with the
same key properties will raise a validation error.

#### Guidelines

- **First mention:** When discussing a specific property, specify the suffix _key property_ after
  the formatted property name. When discussing key resource properties conceptually, specify the
  phrase as _key resource properties_.
- **Subsequent mentions:** When discussing a specific property, you can omit the word _key_. When
  discussing key properties conceptually, you can omit the word _resource_ and use the phrase _key
  properties_ instead. If it's clear from context, you can omit the word _key_.
- Follow the same formatting for general resource properties.

#### Examples

> The `Microsoft.Windows/Registry` resource has two key properties:
>
> - `keyPath` uniquely identifies the registry key to manage. This key property is required.
> - `valueName` uniquely identifies the registry value to manage. This key property is optional
>   unless you specify a value for the `valueData` property.

> Specify the `settingsScope` key property defines whether the settings file should be updated for
> the machine or current user.

### Canonical resource properties

DSC defines a set of common purpose properties for use in resource instance schemas. These
properties indicate that the resource is participating in specific semantics that enables DSC to
handle certain behaviors on behalf of the resource. For more information about canonical
properties, see [DSC canonical properties][01]

#### Guidelines

- **First mention:** When discussing a specific property, specify the suffix _canonical property_
  after the formatted property name. When discussing canonical resource properties conceptually,
  specify the phrase as _canonical resource properties_.
- **Subsequent mentions:** When discussing a specific property, you can omit the word _canonical_.
  When discussing canonical properties conceptually, you can omit the word _resource_ and use the
  phrase _canonical properties_ instead. If it's clear from context, you can omit the word
  _canonical_.
- Follow the same formatting for general resource properties. Canonical property names always have
  an underscore (`_`) prefix.

#### Examples

> The `_exist` canonical property defines whether an instance should exist.

> When defining your resource, consider whether you can use any of the semantics DSC defines for
> canonical resource properties. If your resource manages instances that can be created, updated,
> and deleted, consider using the `_exist_ canonical property. Implementing your resource to adhere
> to canonical properties makes it easier for your users to understand how your resource behaves
> and reduces the code you need to write by letting DSC handle some behaviors for your resource.

### Read-only resource properties

Read-only resource properties of a resource describe nonconfigurable information about an instance
that the resource returns. Examples include metadata, such as the last time a file was modified, or
the latest available version of a package.

Resources indicate which properties are read-only in their instance schema by defining the
`readOnly` keyword as `true`.

#### Guidelines

- **First mention"** When discussing a specific property, specify the suffix "read-only property"
  after the formatted property name. When discussing read-only resource properties conceptually,
  specify the phrase as "read-only resource properties.
- **Subsequent mentions:** When discussing a specific property, you can omit the phrase
  "read-only." When discussing read-only properties conceptually, you can omit the word "resource"
  and use the phrase "read-only properties" instead.
- Follow the same formatting for general resource properties.

#### Examples

> The `lastWriteTime` read-only resource property indicates when the file was last updated. The
> `creationTime` read-only property indicates when the file was created.

> When defining a resource, consider whether your resource should return any non-configurable
> metadata for users as read-only resource properties. Defining read-only properties can enable
> your users to more effectively and quickly investigate and audit their systems by providing
> information they need about an instance but can't directly configure.

### Write-only resource properties

Write-only properties of a resource instance describe options that influence how the resource
behaves, but can't be returned from the machine. Examples of write-only properties include
credentials required for configuring the resource and the
[_purge][02] canonical property.

Resources indicate which properties are write-only in their instance schema by defining the
`writeOnly` keyword as `true`.

#### Guidelines

- **First mention"** When discussing a specific property, specify the suffix "write-only property"
  after the formatted property name. When discussing write-only resource properties conceptually,
  specify the phrase as "write-only resource properties.
- **Subsequent mentions:** When discussing a specific property, you can omit the phrase
  "write-only." When discussing write-only properties conceptually, you can omit the word "resource"
  and use the phrase "write-only properties" instead.
- Follow the same formatting for general resource properties.

#### Examples

> The `token` write-only resource property defines the access token the resource must use to access
> the remote datastore. The `connectionTimeout` write-only property defines how many seconds the
> resource should allow for retrieving the data. The `connectionTimeout` property defaults to sixty
> seconds.

> When defining a resource, consider whether your resource needs any options that change how the
> resource behaves but can't be retrieved from the system as write-only resource properties. If
> your resource requires or can use credentials, credential properties should always be write-only
> properties.

### Resource instance schema

The JSON schema that describes and validates the properties of a resource instance. All command
resources define a resource instance schema. Adapter resources provide the instance schema for
their adapted resources.

#### Guidelines

- **First mention:** resource instance schema
- **Subsequent mentions:** resource schema or schema.
- Specify the full term when discussing multiple kinds of schema.

## General terms

### Desired State Configuration

Microsoft's Desired State Configuration (DSC) is a declarative configuration platform. With DSC,
you describe the state of a machine in a format that anyone can read and understand, not just
subject matter experts.

#### Guidelines

- **First mention:** Microsoft's Desired State Configuration (DSC) platform
- **Subsequent mentions:** DSC or DSC platform
- Specify the platform suffix when referencing the platform specifically in contexts where the
  term could be confused with PowerShell DSC or the `dsc` command.

#### Examples

> In Microsoft's Desired State Configuration (DSC) platform, DSC resources represent a standardized
> interface for managing the settings of a system.

> You can use DSC to list the available resources with the `dsc resource list` command.

> For resources that don't implement the **Test** operation, DSC can validate an instance's state
> with a synthetic test.

### `dsc`

The DSC command line tool that invokes resources and manages configuration documents.

#### Guidelines

- Specify the term as DSC when discussing the command line tool in general.
- Use code formatting when discussing running the command, a specific subcommand, or to distinguish
  the command line tool from the conceptual platform.

#### Examples

> Use the `dsc resource test` command to invoke the operation. DSC returns data that includes:
>
> - The desired state for the instance.
> - The actual state of the instance.
> - Whether the instance is in the desired state.
> - The list of properties that aren't in the desired state.

### PowerShell Desired State Configuration (PSDSC)

The Desired State Configuration feature of PowerShell. Previously, this term included the
PowerShell DSC platform, the Local Configuration Manager, and the **PSDesiredStateConfiguration**
PowerShell module.

For DSC, this term applies to defining and using resources that are implemented in PowerShell and
compatible with PSDSC. DSC users manage PSDSC resource instances with the
`Microsoft.DSC/PowerShell` or `Microsoft.Windows/WindowsPowerSHell` adapter resources.

#### Guidelines

- **First mention:** PowerShell Desired State Configuration (PSDSC)
- **Subsequent mentions:** PowerShell DSC or PSDSC
- Always distinguish PowerShell DSC from the DSC platform and the `dsc` command.
- Always specify the **PSDesiredStateConfiguration** module by name and strongly emphasized when
  discussing the PowerShell module itself.

#### Examples

> You can use PowerShell DSC (PSDSC) resources with DSC as adapted resources.

> Get started authoring a class-based PowerShell DSC resource to manage a configuration file.
> Completing this tutorial gives you a functional class-based PSDSC Resource in a module you can
> use for further learning and customization.

<!-- Link reference definitions -->
[01]: ./reference/schemas/resource/properties/overview.md
[02]: ./reference/schemas/resource/properties/ensure.md
