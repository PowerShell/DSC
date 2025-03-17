---
description: >-
  Learn how to use Microsoft's Desired State Configuration platform to manage the state of a
  machine as code.
ms.date: 03/18/2025
title: Get started with DSC
---

# Get started with DSC

Microsoft's Desired State Configuration (DSC) is a declarative configuration platform. You can use
DSC resources to query, audit, and set the state of a specific component on a system. You can use
DSC configuration documents to describe how a system should be configured using one or more
resources. This document describes how you can discover the resources available on a machine,
invoke those resources directly, and manage a configuration.

## Prerequisites

- [Install DSC][01] version `3.0.0` on a Windows machine.
- A terminal emulator, like [Windows Terminal][02].

## Discover resources

You can use the `dsc resource list` command to enumerate the available DSC resources on a machine.
DSC discovers resources by searching the folders in your `PATH` environment variable for files that
have any of the following suffixes:

- `.dsc.resource.json`
- `.dsc.resource.yaml`
- `.dsc.resource.yml`

Files with the `.dsc.resource.<extension>` suffix are DSC resource manifests. They describe both
the settings they enable you to manage and how DSC can invoke them as a command.

Open a terminal and run the following command:

```powershell
dsc resource list
```

DSC outputs a table showing the available resources and summary information about each resource.

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

Together, the columns describe each resource:

- The **Type** column defines the fully qualified type name for a resource. This identifies the
  resource on a system and in a configuration document.
- The **Kind** column indicates how you can use the resource.
  - `Adapter` indicates that the resource is an adapter resource and enables you to configure
    components that don't define a DSC resource manifest, like PowerShell DSC (PSDSC) resources.
  - `Group` indicates that the resource is a group resource and changes how DSC processes a list of
    resource instances. Group resources don't directly manage state on a system.
  - `Importer` indicates that the resource is an importer resource that retrieves a configuration
    document from an external source to insert into the current configuration document.
  - `Resource` indicates that the resource is typical - not an adapter, group, or importer resource.
- The **Version** column indicates the latest discovered version of the resource on the system.
- The **Capabilities** column indicates how the resource is implemented and what you can expect
  when using it. The table displays the capabilities as flags in the following order, using a `-`
  instead of the appropriate letter if the resource doesn't have a specific capability:

  - `g` indicates that the resource has the `get` capability.
  - `s` indicates that the resource has the `set` capability.
  - `x` indicates that the resource has the `setHandlesExist` capability.
  - `w` indicates that the resource has the `whatIf` capability.
  - `t` indicates that the resource has the `test` capability.
  - `d` indicates that the resource has the `delete` capability.
  - `e` indicates that the resource has the `export` capability.
  - `r` indicates that the resource has the `resolve` capability.

- The **RequireAdapter** column indicates which adapter resource, if any, the resource requires.
- The **Description** column defines a synopsis for the resource.

By default, the `dsc resource list` command only returns command resources, which always have a
resource manifest. You can use the `--adapter` option to return the resources for one or more
adapted resources.

Run the following command to return the available adapted resources instead of command resources:

```powershell
$adaptedResources = dsc resource list --adapter * | ConvertFrom-Json

$adaptedResources |
  Group-Object -NoElement -Property requireAdapter |
  Format-Table -AutoSize
```

```Output
Count Name
----- ----
   27 Microsoft.Windows/WindowsPowerShell
  961 Microsoft.Windows/WMI
```

The first command saves the output of `dsc resource list` to the `$resources` variable as an array
of PowerShell objects. WHen you first ran the `dsc resource list` command, the output in the
terminal was a table view. By default, when DSC detects that its output is being redirected to a
file, variable, or another command in the pipeline, it emits JSON representing the output.
Converting the JSON into a PowerShell object with the `ConvertFrom-Json` cmdlet enables you to
treat the output like any other PowerShell object.

The second command uses the `Group-Object` cmdlet to summarize the available adapted resources,
grouped by the adapter that they require. The specific count for adapted resources depends on the
machine you're using and whether you've installed any PowerShell modules that export any PSDSC
resources.

Run the following command to display only the resources available with the
`Microsoft.Windows/WindowsPowerShell` adapter.

```powershell
$adaptedResources |
  Where-Object -requireAdapter -EQ Microsoft.Windows/WindowsPowerShell |
  Format-Table -Property type, kind, version, capabilities, description
```

```Output
type                                                  kind     version capabilities     description
----                                                  ----     ------- ------------     -----------
PSDesiredStateConfiguration/Archive                   resource 1.1     {get, set, test} 
PSDesiredStateConfiguration/Environment               resource 1.1     {get, set, test} 
PSDesiredStateConfiguration/File                      resource 1.0.0   {get, set, test} 
PSDesiredStateConfiguration/Group                     resource 1.1     {get, set, test} 
PSDesiredStateConfiguration/GroupSet                  resource 1.1     {get, set, test} 
PSDesiredStateConfiguration/Log                       resource 1.1     {get, set, test} 
PSDesiredStateConfiguration/Package                   resource 1.1     {get, set, test} 
PSDesiredStateConfiguration/ProcessSet                resource 1.1     {get, set, test} 
PSDesiredStateConfiguration/Registry                  resource 1.1     {get, set, test} 
PSDesiredStateConfiguration/Script                    resource 1.1     {get, set, test} 
PSDesiredStateConfiguration/Service                   resource 1.1     {get, set, test} 
PSDesiredStateConfiguration/ServiceSet                resource 1.1     {get, set, test} 
PSDesiredStateConfiguration/SignatureValidation       resource 1.0.0   {get, set, test} 
PSDesiredStateConfiguration/User                      resource 1.1     {get, set, test} 
PSDesiredStateConfiguration/WaitForAll                resource 1.1     {get, set, test} 
PSDesiredStateConfiguration/WaitForAny                resource 1.1     {get, set, test} 
PSDesiredStateConfiguration/WaitForSome               resource 1.1     {get, set, test} 
PSDesiredStateConfiguration/WindowsFeature            resource 1.1     {get, set, test} 
PSDesiredStateConfiguration/WindowsFeatureSet         resource 1.1     {get, set, test} 
PSDesiredStateConfiguration/WindowsOptionalFeature    resource 1.1     {get, set, test} 
PSDesiredStateConfiguration/WindowsOptionalFeatureSet resource 1.1     {get, set, test} 
PSDesiredStateConfiguration/WindowsPackageCab         resource 1.1     {get, set, test} 
PSDesiredStateConfiguration/WindowsProcess            resource 1.1     {get, set, test} 
PackageManagement/PackageManagement                   resource 1.4.8.1 {get, set, test} PackageManagement (a.k.a. OneGet) is a new way to discover and install software packa…
PackageManagement/PackageManagementSource             resource 1.4.8.1 {get, set, test} PackageManagement (a.k.a. OneGet) is a new way to discover and install software packa…
PowerShellGet/PSModule                                resource 2.2.5   {get, set, test} PowerShell module with commands for discovering, installing, updating and publishing …
PowerShellGet/PSRepository                            resource 2.2.5   {get, set, test} PowerShell module with commands for discovering, installing, updating and publishing …
```

For more information about the command, see [dsc resource list][03].

## Invoke a resource

You can use DSC to directly invoke a resource. When you invoke a resource, you are performing a
specific operation on an _instance_ of the resource. A resource instance is a specific item the
resource represents. The following examples use the `Microsoft.Windows/Registry` resource to invoke
DSC operations.

First, use the `dsc resource list` command to see what capabilities the `Registry` resource has.

```powershell
$resource = dsc resource list Microsoft.WIndows/Registry | ConvertFrom-Json
$resource.capabilities
```

```Output
get
set
whatIf
delete
```

Together, these capabilities tell us that you can use the `Registry` command resource to:

- The `get` capability indicates that you can invoke the **Get** operation to retrieve the current
  state of an instance.
- The `set` capability indicates that you can invoke the **Set** operation to modify the state of
  an instance.
- The `whatIf` capability indicates that you can invoke the **Set** operation in what-if mode to
  see how the resource would change the instance without actually modifying the system.
- The `delete` capability indicates that you can invoke the **Delete** operation to remove an
  instance from the system.

### Get the current state of a resource

You can use the `dsc resource get` command to retrieve the current state of an instance. Run the
following command to define the data that represents a registry value:

```powershell
$instance = @{
  keyPath   = 'HKLM\Software\Microsoft\Windows NT\CurrentVersion'
  valueName = 'SystemRoot'
} | ConvertTo-Json
```

For this example:

- The `keyPath` property to indicate the path to the registry key that should contain the value you
  want to manage. `keyPath` is a _required_ property for the `Registry` resource - you always need
  to specify it.
- The `valueName` property identifies which registry value to manage for the registry key.
- The `_exist` canonical property indicates whether the registry value should exist. In this example,
  it's set to `true`, indicating the instance should exist.

Run the following command to get the current state of the registry key:

```powershell
dsc resource get --resource $resource.type --input $instance
```

```yaml
actualState:
  keyPath: HKLM\Software\Microsoft\Windows NT\CurrentVersion
  valueName: SystemRoot
  valueData:
    String: C:\WINDOWS
```

The output shows that the value is set to the string `C:\WINDOWS`.

### Test whether a resource is in the desired state

Retrieving the current state of a resource is useful, but in practice you more often want to know
whether an instance is in the desired state. The `dsc resource test` command not only tells you
whether an instance is out of state, but how it's out state.

> {!NOTE}
> Remember that the `Registry` resource doesn't have the `test` capability. Fortunately, that
> doesn't mean that you can't use the **Test** operation for a resource. When a resource doesn't
> have the `test` capability, it is indicating that the resource depends on DSC's synthetic test
> capabilities. DSC itself calls the **Get** operation for the resource and compares it to the
> desired state.

Run the following commands to define the desired state of a registry key that doesn't exist and test
it.

```powershell
$desired = @{
  keyPath   = 'HKCU\dsc\example\key'
  _exist    = $true
} | ConvertTo-Json
dsc resource test --resource $resource.type --input $desired
```

```yaml
desiredState:
  keyPath: HKCU\dsc\example\key
  _exist: true
actualState:
  keyPath: HKCU\dsc\example\key
  _exist: false
inDesiredState: false
differingProperties:
- _exist
```

The output for the command shows you the desired state, actual state, and how they differ. In this
case, the registry key doesn't exist - the `_exist` property is `false` when the desired state is
`true`.

### Set the desired state of a resource

You can also directly invoke a resource to set the state for an instance if the resource has the
`set` capability.

Run the following command to create the registry key:

```powershell

dsc resource set --resource $resource.type --input $desired
```

```yaml
beforeState:
  keyPath: HKCU\dsc\example\key
  _exist: false
afterState:
  keyPath: HKCU\dsc\example\key
changedProperties:
- _exist
```

The output indicates that the resource created the missing instance.

## Manage a configuration

Invoking resources directly is useful, but tedious for defining the desired state of a system. You
can define a DSC configuration document to describe a set of resource instances together.

Copy the following codeblock and save it in a file named `example.dsc.config.yaml`.

```yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Example registry key 
  type: Microsoft.Windows/Registry
  properties:
    keyPath: HKCU\dsc\example\key
    _exist:  true
- name: PSDSC resources
  type: Microsoft.Windows/WindowsPowerShell
  properties:
    resources:
    - name: DSC_EXAMPLE env variable
      type: PSDesiredStateConfiguration/Environment
      properties:
        Name: DSC_EXAMPLE
        Ensure: Present
        Value: Set by DSC
```

The configuration document specifies two resource instances at the top level:

- The `Microsoft.Windows/Registry` resource to ensure that a registry key exists.
- The `Microsoft.Windows/WindowsPowerShell` adapter resource to define PowerShell DSC (PSDSC)
  resources.

The `WindowsPowerShell` adapter instance defines a single property, `resources`, as an array of
resource instances. In this configuration, it defines two instances:

- The first instance uses the `PSRepository` PSDSC resource from the **PowerShellGet** module to
  make sure the PowerShell Gallery is available for use as a repository.
- The second instance uses the `PSModule` PSDSC resource from the same module to make sure that the
  **Microsoft.WinGet.Client** module is installed.

Open a terminal with elevated permissions. PSDSC resources in Windows PowerShell need to run as
administrator. In the elevated terminal, change directory to the folder where you saved the
configuration document as `example.dsc.config.yaml`. Then run the following command.

```powershell
dsc config test --file ./example.dsc.config.yaml
```

```yaml
metadata:
  Microsoft.DSC:
    version: 3.0.0
    operation: test
    executionType: actual
    startDatetime: 2025-03-03T17:11:25.726475600-06:00
    endDatetime: 2025-03-03T17:11:32.567311800-06:00
    duration: PT6.8408362S
    securityContext: elevated
results:
- metadata:
    Microsoft.DSC:
      duration: PT0.1818183S
  name: Example registry key
  type: Microsoft.Windows/Registry
  result:
    desiredState:
      keyPath: HKCU\dsc\example\key
      _exist: true
    actualState:
      keyPath: HKCU\dsc\example\key
    inDesiredState: true
    differingProperties: []
- metadata:
    Microsoft.DSC:
      duration: PT3.0461988S
  name: PSDSC resources
  type: Microsoft.Windows/WindowsPowerShell
  result:
    desiredState:
      resources:
      - name: DSC_EXAMPLE env variable
        type: PSDesiredStateConfiguration/Environment
        properties:
          Name: DSC_EXAMPLE
          Ensure: Present
          Value: Set by DSC
      metadata:
        Microsoft.DSC:
          context: configuration
    actualState:
      result:
      - name: DSC_EXAMPLE env variable
        type: PSDesiredStateConfiguration/Environment
        properties:
          InDesiredState: false
    inDesiredState: false
    differingProperties:
    - resources
    - metadata
messages: []
hadErrors: false
```

The output shows that:

- The `Registry` instance is in the desired state. This is because the key was created earlier in
  this article when invoking the **Set** operation directly.

- The adapted `Environment` PSDSC resource reports that the `DSC_EXAMPLE` environment variable is
  _not_ in the desired state.

Run the following command to enforce the desired state on the system.

```powershell
dsc config set --file ./example.dsc.config.yaml
```

```yaml
metadata:
  Microsoft.DSC:
    version: 3.0.0
    operation: set
    executionType: actual
    startDatetime: 2025-03-03T17:14:15.841393700-06:00
    endDatetime: 2025-03-03T17:14:29.136469500-06:00
    duration: PT13.2950758S
    securityContext: elevated
results:
- metadata:
    Microsoft.DSC:
      duration: PT0.2633556S
  name: Example registry key
  type: Microsoft.Windows/Registry
  result:
    beforeState:
      keyPath: HKCU\dsc\example\key
      _exist: true
    afterState:
      keyPath: HKCU\dsc\example\key
    changedProperties: null
- metadata:
    Microsoft.DSC:
      duration: PT8.6601181S
  name: PSDSC resources
  type: Microsoft.Windows/WindowsPowerShell
  result:
    beforeState:
      result:
      - name: DSC_EXAMPLE env variable
        type: PSDesiredStateConfiguration/Environment
        properties:
          ResourceId: null
          PsDscRunAsCredential: null
          PSComputerName: localhost
          ModuleVersion: '1.1'
          Value: null
          Path: null
          ConfigurationName: null
          Name: DSC_EXAMPLE
          ModuleName: PSDesiredStateConfiguration
          SourceInfo: null
          DependsOn: null
          Ensure: Absent
    afterState:
      result:
      - name: DSC_EXAMPLE env variable
        type: PSDesiredStateConfiguration/Environment
        properties:
          RebootRequired: false
    changedProperties:
    - result
messages: []
hadErrors: false
```

To review the actual state of the system, run the `dsc config get` command:

```powershell
dsc config get --file ./example.dsc.config.yaml
```

```yaml
metadata:
  Microsoft.DSC:
    version: 3.0.0
    operation: get
    executionType: actual
    startDatetime: 2025-03-03T17:16:39.507848200-06:00
    endDatetime: 2025-03-03T17:16:47.734256600-06:00
    duration: PT8.2264084S
    securityContext: elevated
results:
- metadata:
    Microsoft.DSC:
      duration: PT0.1739569S
  name: Example registry key
  type: Microsoft.Windows/Registry
  result:
    actualState:
      keyPath: HKCU\dsc\example\key
- metadata:
    Microsoft.DSC:
      duration: PT3.9958946S
  name: PSDSC resources
  type: Microsoft.Windows/WindowsPowerShell
  result:
    actualState:
      result:
      - name: DSC_EXAMPLE env variable
        type: PSDesiredStateConfiguration/Environment
        properties:
          ResourceId: null
          PsDscRunAsCredential: null
          PSComputerName: localhost
          ModuleVersion: '1.1'
          Value: Set by DSC
          Path: null
          ConfigurationName: null
          Name: DSC_EXAMPLE
          ModuleName: PSDesiredStateConfiguration
          SourceInfo: null
          DependsOn: null
          Ensure: Present
messages: []
hadErrors: false
```

## Related content

- [Glossary: Desired State Configuration][04]
- [DSC configuration documents][05]
- [DSC Resources][06]
- [DSC command reference][07]

<!-- Link reference definitions -->

[01]: ../overview.md#installation
[02]: https://learn.microsoft.com/windows/terminal/
[03]: ../reference/cli/resource/list.md
[04]: ../glossary.md
[05]: ../concepts/configurations.md
[06]: ../concepts/resources.md
[07]: ../reference/cli/dsc.md
