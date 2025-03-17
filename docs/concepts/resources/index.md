---
description: >-
  DSC Resources provide a standardized interface for idempotently managing the settings of a
  system. They use a declarative syntax to define what the desired state should be.
ms.date: 03/18/2025
title: DSC Resources
---

# DSC Resources

<!-- markdownlint-disable MD013 -->

In Microsoft's Desired State Configuration (DSC) platform, DSC Resources represent a standardized
interface for managing the settings of a system. Resources can model components as generic as a
file or as specific as an IIS server setting. Resources use a declarative syntax rather than
imperative. Instead of specifying _how_ to set a system to the desired state, with DSC you specify
_what_ the desired state is. Resources handle the "how" for you.

Resources manage _instances_ of a configurable component. For example, the
`PSDscResources/Environment` resource manages environment variables. Each environment variable is a
different instance of the resource. Every resource defines a schema that describes how to validate
and manage an instance of the resource.

DSC supports several kinds of resources:

- A resource defined with a resource manifest is a _command_ resource. DSC uses the manifest
  to determine how to invoke the resource and how to validate the resource instance properties.
- A _group resource_ is a command resource with a `resources` property that takes an array of
  resource instances and processes them. Group resources may apply special handling to their nested
  resource instances, like changing the user the resources run as.
- An _adapter resource_ is a group resource that enables the use of non-command resources with DSC.
  For example, the `Microsoft.DSC/PowerShell` and `Microsoft.Windows/WIndowsPowerShell` adapter
  resources enable the use of PowerShell DSC (PSDSC) resources in DSC, invoking the resources in
  PowerShell and Windows PowerShell respectively.

## Resource type names

Resources are identified by their fully qualified type name. The type name is used to specify a
resource in configuration documents and as the value of the `--resource` flag when using the
`dsc resource *` commands.

The fully qualified type name of a resource uses the following syntax:

```Syntax
<owner>[.<group>][.<area>]/<name>
```

Every resource must define an `owner` and a `name`. The `group` and `area` components enable
organizing resources into related namespaces, like `Microsoft.SqlServer/Database` and
`Microsoft.SqlServer.Database/Role`.

For more information about type names and how DSC validates them, see
[DSC Resource fully qualified type name schema reference][01].

## Resource properties

The properties of a resource are the settings and options a user can declare for managing an
instance. Resources always have at least one property. Resources define their properties in their
instance schema.

Properties are optional by default. Resources can be invoked directly or declared in a
configuration with only the properties that are relevant to the current task or purpose. You don't
need to declare every property for an instance. Properties may have default values for their
desired state.

Most properties are one of the basic types:

- String properties require the property value to be a set of characters, like `machine`.
- Integer properties require the property value to be a number without a fractional part, like `5`.
- Boolean properties require the property value to be either `true` or `false`.
- Array properties require the property value to be a list of items. Usually, array properties
  specify that the values must be of a particular type, like a list of exit code integers or a list
  of file paths.

Complex properties require the property value to be an object with defined subproperties. The
subproperties can be basic or complex, but they're usually a basic type.

Resources may define their properties as read-only or write-only:

- A _read-only resource property_ defines metadata about an instance that the resource can retrieve
  but that a user can't directly set. You can't specify read-only properties in the desired state
  for an instance. Examples of read-only properties include the last time a file was modified or
  the author of an installed software package.
- A _write-only resource property_ defines a value that the resource uses during a resource
  operation but which can't be returned for the current state of an instance. Examples of
  write-only properties include credentials used to authenticate during a resource operation and
  the temporary directory to use when retrieving and unpacking a remote archive.

DSC defines a set of _canonical resource properties_ which indicate that a resource participates in
shared semantics the DSC engine provides. For example, any resource that includes the `_exist`
canonical property in its instance schema indicates that the resource manages instances that can be
created and deleted. If a resource has the `_exist` canonical property and the `delete` capability,
DSC can handle invoking the **Delete** operation instead of **Set** when the desired state
indicates the instance shouldn't exist. For more information about the available canonical
properties, see [DSC canonical properties](../reference/schemas/resource/properties/overview.md).

## Listing resources

You can use DSC to list the available resources with the `dsc resource list` command. DSC searches
the `PATH` for command-based resources and invokes available resource providers to list their
resources.

By default, the command returns every discovered DSC Resource.

```sh
dsc resource list
```

```Output
Type                                        Kind      Version  Capabilities  RequireAdapter  Description
------------------------------------------------------------------------------------------------------------------------------------------------------------------------
Microsoft.DSC.Debug/Echo                    Resource  1.0.0    gs--t---
Microsoft.DSC.Transitional/RunCommandOnSet  Resource  0.1.0    gs------                      Takes a single-command line to execute on DSC set operation
Microsoft.DSC/Assertion                     Group     0.1.0    gs--t---                      `test` will be invoked for all resources in the supplied configuration.
Microsoft.DSC/Group                         Group     0.1.0    gs--t---                      All resources in the supplied configuration is treated as a group.
Microsoft.DSC/Include                       Importer  0.1.0    gs--t---                      Allows including a configuration file with optional parameter file.
Microsoft.DSC/PowerShell                    Adapter   0.1.0    gs--t-e-                      Resource adapter to classic DSC Powershell resources.
Microsoft.Windows/RebootPending             Resource  0.1.0    g-------                      Returns info about pending reboot.
Microsoft.Windows/Registry                  Resource  0.1.0    gs-w-d--                      Manage Windows Registry keys and values
Microsoft.Windows/WMI                       Adapter   0.1.0    g-------                      Resource adapter to WMI resources.
Microsoft.Windows/WindowsPowerShell         Adapter   0.1.0    gs--t---                      Resource adapter to classic DSC Powershell resources in Windows PowerShell.
Microsoft/OSInfo                            Resource  0.1.0    g-----e-                      Returns information about the operating system.
Microsoft/Process                           Resource  0.1.0    gs--t-e-                      Returns information about running processes.
```

You can filter the results by a resource's type name, description, and tags. For more information,
see [dsc resource list][02]

## Invoking resources

You can invoke resources directly with the `dsc resource *` commands to manage a single instance
through the three primary DSC operations: **Get**, **Test**, **Set**. If the resource has the
capability, you can also invoke the **Export** or **Delete** operations.

### Get operations

Every resource implements the **Get** operation, which retrieves the actual state of a resource
instance. Use the `dsc resource get` command to invoke the operation.

For example, you can use the `Microsoft.Windows/Registry` resource to get the actual state for a
registry key value:

```powershell
dsc resource get --resource Microsoft.Windows/Registry --input '{
    "keyPath": "HKLM\\Software\\Microsoft\\Windows NT\\CurrentVersion",
    "valueName": "SystemRoot"
}'
```

```yaml
actualState:
  keyPath: HKLM\Software\Microsoft\Windows NT\CurrentVersion
  valueName: SystemRoot
  valueData:
    String: C:\WINDOWS
```

### Test operations

Some resources implement the **Test** operation. For resources that don't implement the **Test**
operation, DSC can validate an instance's state with a synthetic test. The synthetic test is a
strict case-insensitive comparison of the desired and actual values for the instance's properties.
Only resources that have advanced or complex validation requirements need to implement the **Test**
operation themselves.

Use the `dsc resource test` command to invoke the operation. DSC returns data that includes:

- The desired state for the instance.
- The actual state of the instance.
- Whether the instance is in the desired state.
- The list of properties that aren't in the desired state.

For example, you can test whether a specific registry key exists:

```powershell
dsc resource test --resource Microsoft.Windows/Registry --input '{
    "keyPath": "HKCU\\key\\that\\does\\not\\exist",
    "exist": true
}'
```

```yaml
desiredState:
  keyPath: HKCU\key\that\does\not\exist
  _exist: true
actualState:
  keyPath: HKCU\key\that\does\not\exist
  _exist: false
inDesiredState: false
differingProperties:
- _exist
```

### Set operations

<!-- vale Microsoft.Adverbs = NO -->

Most resources implement the **Set** operation, which enforces the desired state for an
instance. When used with DSC, the **Set** operation is _idempotent_, which means that the
resource only invokes the operation when an instance isn't in the desired state. Because the
operation is idempotent, invoking it repeatedly is the same as invoking it once. The idempotent
model prevents side effects from unnecessarily executing code.

<!-- vale Microsoft.Adverbs = YES -->

Resources that don't implement the **Set** operation are _assertion_ resources. You can use
assertion resources to retrieve and validate the state of an instance, but you can't use them to
enforce a desired state.

Use the `dsc resource set` command to invoke the operation. DSC returns data that includes:

- The state of the instance before the operation.
- The state of the instance after the operation.
- The list of properties the operation changed.

For example, you can create a registry key by setting the desired state for a key that doesn't
exist.

```powershell
dsc resource set --resource Microsoft.Windows/Registry --input '{
    "keyPath":   "HKCU\\example\\key",
    "valueName": "Example",
    "valueData": { "String": "This is an example." }
}'
```

```yaml
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
```

### Delete operations

Some resources implement the **Delete** operation for convenience. This enables you to invoke the
resource to remove an instance from the system.

Use the `dsc resource delete` command to invoke the operation. When you invoke the **Delete**
operation, DSC returns no output unless there's an error.

For example, you can delete the registry created in the **Set** operation example:

```powershell
dsc resource delete --resource Microsoft.Windows/Registry --input '{
    "keyPath":   "HKCU\\example\\key"
}'
```

### Export operations

Some resources implement the **Export** operation, which returns every instance of the resource on
the system. This can help you discover how a machine is currently configured.

Use the `dsc resource export` command to invoke the operation. When you invoke the **Export**
operation, DSC returns an array of resources instance definitions you can copy into a configuration
document.

## Declaring resource instances

DSC configuration documents enable managing more than one resource or resource instance at a time.
Configuration documents declare a collection of resource instances and their desired state. This
makes it possible to model complex desired states by composing different resources and instances
together, like defining a security baseline for compliance or the settings for a web farm.

A resource instance declaration always includes:

- `name` - A short, human-readable name for the instance that's unique in the document. This name
  is used for logging and it helps to document an instance's purpose in the document.
- `type` - The fully qualified type name for the resource to identify the resource DSC should use
  to manage the instance.
- `properties` - The desired state for the instance. DSC validates the values against the
  resource's instance schema.

This example configuration document snippet declares an instance of the
`Microsoft.Windows/Registry` resource.

```yaml
$schema: https://schemas.microsoft.com/dsc/2023/08/configuration.schema.json
resources:
  - name: example key value
    type: Microsoft.Windows/Registry
    properties:
      keyPath: HKCU\example\key
      valueName: Example
      valueData:
        String: This is an example.
```

## See also

- [Anatomy of a DSC command resource][03] to learn about authoring resources in your language
  of choice.
- [Configuration Documents][04] to learn about using resources in a configuration document.
- [Command line reference for the 'dsc resource' command][05]

[01]: ../../reference/schemas/definitions/resourceType.md
[02]: ../../reference/cli/resource/list.md
[03]: anatomy.md
[04]: ../configuration-documents/index.md
[05]: ../../reference/cli/resource/command.md
