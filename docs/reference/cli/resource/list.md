---
description: Command line reference for the 'dsc resource list' command
ms.date:     06/24/2024
ms.topic:    reference
title:       dsc resource list
---

# dsc resource list

## Synopsis

Returns the list of available DSC Resources with an optional filter.

## Syntax

```sh
dsc resource list [Options] <RESOURCE_NAME>
```

## Description

The `list` subcommand searches for available DSC Resources and returns their information. DSC
discovers resources by first searching the `PATH` or `DSC_RESOURCE_PATH` environment variable for
`.dsc.resource.json`, `.dsc.resource.yml`, and `dsc.resource.yaml` files. For more information
about the environment variables DSC uses, see [Environment variables][01]

If any of the discovered resources are resource adapters, DSC calls the `list` operation for those
adapters if the [--adapter](#-a---adapter) option specifies a matching filter. By default, DSC
doesn't return any adapted resources.

DSC returns the list of discovered resources with their implementation information and metadata. If
the command includes the `RESOURCE_NAME` argument, DSC filters the list of discovered resources
before returning them. The **description** and **tags** options filter the results by the
resource descriptions and tags. Filters are always applied after resource discovery.

## Examples

### Example 1 - List all non-adapted resources

Without any filters, the command returns every discovered DSC Resource, but doesn't call the `list`
operation for adapter resources to enumerate any adapted resources.

```sh
dsc resource list
```

```Output
Type                                        Kind      Version  Caps      RequireAdapter  Description
--------------------------------------------------------------------------------------------------------------------------------------------------------------------
Microsoft.DSC.Transitional/RunCommandOnSet  Resource  0.1.0    gs------                  Takes a single-command line to execute on DSC set operation
Microsoft.DSC/Assertion                     Group     0.1.0    gs--t---                  `test` will be invoked for all resources in the supplied configuration.    
Microsoft.DSC/Group                         Group     0.1.0    gs--t---                  All resources in the supplied configuration is treated as a group.
Microsoft.DSC/Include                       Import    0.1.0    -------r                  Allows including a configuration file contents into current configuration. 
Microsoft.DSC/Parallel                      Group     0.1.0    gs--t---                  All resources in the supplied configuration run concurrently.
Microsoft.DSC/PowerShell                    Adapter   0.1.0    gs--t-e-                  Resource adapter to classic DSC Powershell resources.
Microsoft.Windows/RebootPending             Resource  0.1.0    g-------                  Returns info about pending reboot.
Microsoft.Windows/Registry                  Resource  0.1.0    gs---d--                  Manage Windows Registry keys and values
Microsoft.Windows/WMI                       Adapter   0.1.0    g-------                  Resource adapter to WMI resources.
Microsoft.Windows/WindowsPowerShell         Adapter   0.1.0    gs--t---                  Resource adapter to classic DSC Powershell resources in Windows PowerShell.
Microsoft/OSInfo                            Resource  0.1.0    g-----e-                  Returns information about the operating system.
Microsoft/Process                           Resource  0.1.0    gs--t-e-                  Returns information about running processes.
```

### Example 2 - List a specific resource

When the `RESOURCE_NAME` argument doesn't include a wildcard, the command returns only the resource
with the specified type name.

```sh
dsc resource list Microsoft.DSC/Group
```

```Output
Type                 Kind   Version  Caps      RequireAdapter  Description
---------------------------------------------------------------------------------------------------------------------------------
Microsoft.DSC/Group  Group  0.1.0    gs--t---                  All resources in the supplied configuration is treated as a group.
```

### Example 3 - List resources with a matching type name

When the `RESOURCE_NAME` argument includes a wildcard, the command returns every resource with a
matching type name.

```sh
dsc resource list Microsoft.DSC/*
```

```Output
Type                      Kind     Version  Caps      RequireAdapter  Description
------------------------------------------------------------------------------------------------------------------------------------------------
Microsoft.DSC/Assertion   Group    0.1.0    gs--t---                  `test` will be invoked for all resources in the supplied configuration.
Microsoft.DSC/Group       Group    0.1.0    gs--t---                  All resources in the supplied configuration is treated as a group.
Microsoft.DSC/Include     Import   0.1.0    -------r                  Allows including a configuration file contents into current configuration.
Microsoft.DSC/Parallel    Group    0.1.0    gs--t---                  All resources in the supplied configuration run concurrently.
Microsoft.DSC/PowerShell  Adapter  0.1.0    gs--t-e-                  Resource adapter to classic DSC Powershell resources.
```

### Example 4 - List resources with a matching description

When the command includes the **description** option, the results include resources that have a
description containing the specified value.

```sh
dsc resource list --description 'supplied configuration'
```

```Output
Type                     Kind   Version  Caps      RequireAdapter  Description
------------------------------------------------------------------------------------------------------------------------------------------
Microsoft.DSC/Assertion  Group  0.1.0    gs--t---                  `test` will be invoked for all resources in the supplied configuration.
Microsoft.DSC/Group      Group  0.1.0    gs--t---                  All resources in the supplied configuration is treated as a group.
Microsoft.DSC/Parallel   Group  0.1.0    gs--t---                  All resources in the supplied configuration run concurrently.
```

### Example 5 - List resources with matching tags

When the command includes multiple instances of the **tags** option, the results include resources
that have any of the specified tags.

```sh
dsc resource list --tags Windows --tags Linux
```

```output
Type                        Kind      Version  Caps      RequireAdapter  Description
------------------------------------------------------------------------------------------------------------------------
Microsoft.Windows/Registry  Resource  0.1.0    gs---d--                  Manage Windows Registry keys and values
Microsoft/OSInfo            Resource  0.1.0    g-----e-                  Returns information about the operating system.
```

### Example 6 - List resources for a specific adapter

When the command includes the **adapter** option, DSC checks for any discovered resource adapters
with a matching name. If it discovers any, it then calls the `list` operation for the adapter and
adds the returned list of adapted resources to the discovered resource list. DSC applies any
further filters specified with the command after this enumeration.

```sh
dsc resource list --adapter Microsoft.Windows/WindowsPowerShell
```

```Output
Type                                                   Kind      Version   Caps      RequireAdapter                       Description
---------------------------------------------------------------------------------------------------------------------------------------------------------------------------
PSDesiredStateConfiguration/Archive                    Resource  1.1       gs--t---  Microsoft.Windows/WindowsPowerShell
PSDesiredStateConfiguration/Environment                Resource  1.1       gs--t---  Microsoft.Windows/WindowsPowerShell
PSDesiredStateConfiguration/File                       Resource  1.0.0     gs--t---  Microsoft.Windows/WindowsPowerShell
PSDesiredStateConfiguration/Group                      Resource  1.1       gs--t---  Microsoft.Windows/WindowsPowerShell
PSDesiredStateConfiguration/GroupSet                   Resource  1.1       gs--t---  Microsoft.Windows/WindowsPowerShell
PSDesiredStateConfiguration/Log                        Resource  1.1       gs--t---  Microsoft.Windows/WindowsPowerShell
PSDesiredStateConfiguration/Package                    Resource  1.1       gs--t---  Microsoft.Windows/WindowsPowerShell
PSDesiredStateConfiguration/ProcessSet                 Resource  1.1       gs--t---  Microsoft.Windows/WindowsPowerShell
PSDesiredStateConfiguration/Registry                   Resource  1.1       gs--t---  Microsoft.Windows/WindowsPowerShell
PSDesiredStateConfiguration/Script                     Resource  1.1       gs--t---  Microsoft.Windows/WindowsPowerShell
PSDesiredStateConfiguration/Service                    Resource  1.1       gs--t---  Microsoft.Windows/WindowsPowerShell
PSDesiredStateConfiguration/ServiceSet                 Resource  1.1       gs--t---  Microsoft.Windows/WindowsPowerShell
PSDesiredStateConfiguration/SignatureValidation        Resource  1.0.0     gs--t---  Microsoft.Windows/WindowsPowerShell
PSDesiredStateConfiguration/User                       Resource  1.1       gs--t---  Microsoft.Windows/WindowsPowerShell
PSDesiredStateConfiguration/WaitForAll                 Resource  1.1       gs--t---  Microsoft.Windows/WindowsPowerShell
PSDesiredStateConfiguration/WaitForAny                 Resource  1.1       gs--t---  Microsoft.Windows/WindowsPowerShell
PSDesiredStateConfiguration/WaitForSome                Resource  1.1       gs--t---  Microsoft.Windows/WindowsPowerShell
PSDesiredStateConfiguration/WindowsFeature             Resource  1.1       gs--t---  Microsoft.Windows/WindowsPowerShell
PSDesiredStateConfiguration/WindowsFeatureSet          Resource  1.1       gs--t---  Microsoft.Windows/WindowsPowerShell
PSDesiredStateConfiguration/WindowsOptionalFeature     Resource  1.1       gs--t---  Microsoft.Windows/WindowsPowerShell
PSDesiredStateConfiguration/WindowsOptionalFeatureSet  Resource  1.1       gs--t---  Microsoft.Windows/WindowsPowerShell
PSDesiredStateConfiguration/WindowsPackageCab          Resource  1.1       gs--t---  Microsoft.Windows/WindowsPowerShell
PSDesiredStateConfiguration/WindowsProcess             Resource  1.1       gs--t---  Microsoft.Windows/WindowsPowerShell
PSDscResources/Archive                                 Resource  2.12.0.0  gs--t---  Microsoft.Windows/WindowsPowerShell  This module contains the standard DSC resources.
PSDscResources/Environment                             Resource  2.12.0.0  gs--t---  Microsoft.Windows/WindowsPowerShell  This module contains the standard DSC resources.
PSDscResources/Group                                   Resource  2.12.0.0  gs--t---  Microsoft.Windows/WindowsPowerShell  This module contains the standard DSC resources.
PSDscResources/MsiPackage                              Resource  2.12.0.0  gs--t---  Microsoft.Windows/WindowsPowerShell  This module contains the standard DSC resources.
PSDscResources/Registry                                Resource  2.12.0.0  gs--t---  Microsoft.Windows/WindowsPowerShell  This module contains the standard DSC resources.
PSDscResources/Script                                  Resource  2.12.0.0  gs--t---  Microsoft.Windows/WindowsPowerShell  This module contains the standard DSC resources.
PSDscResources/Service                                 Resource  2.12.0.0  gs--t---  Microsoft.Windows/WindowsPowerShell  This module contains the standard DSC resources.
PSDscResources/User                                    Resource  2.12.0.0  gs--t---  Microsoft.Windows/WindowsPowerShell  This module contains the standard DSC resources.
PSDscResources/WindowsFeature                          Resource  2.12.0.0  gs--t---  Microsoft.Windows/WindowsPowerShell  This module contains the standard DSC resources.
PSDscResources/WindowsOptionalFeature                  Resource  2.12.0.0  gs--t---  Microsoft.Windows/WindowsPowerShell  This module contains the standard DSC resources.
PSDscResources/WindowsPackageCab                       Resource  2.12.0.0  gs--t---  Microsoft.Windows/WindowsPowerShell  This module contains the standard DSC resources.
PSDscResources/WindowsProcess                          Resource  2.12.0.0  gs--t---  Microsoft.Windows/WindowsPowerShell  This module contains the standard DSC resources.
```

This next command specifies the resource name filter `*Windows*`, limiting the list of returned
resources:

```sh
dsc resource list --adapter Microsoft.Windows/WindowsPowerShell *Windows*
```

```Output
Type                                                   Kind      Version   Caps      RequireAdapter                       Description
---------------------------------------------------------------------------------------------------------------------------------------------------------------------------
PSDesiredStateConfiguration/WindowsFeature             Resource  1.1       gs--t---  Microsoft.Windows/WindowsPowerShell
PSDesiredStateConfiguration/WindowsFeatureSet          Resource  1.1       gs--t---  Microsoft.Windows/WindowsPowerShell
PSDesiredStateConfiguration/WindowsOptionalFeature     Resource  1.1       gs--t---  Microsoft.Windows/WindowsPowerShell
PSDesiredStateConfiguration/WindowsOptionalFeatureSet  Resource  1.1       gs--t---  Microsoft.Windows/WindowsPowerShell
PSDesiredStateConfiguration/WindowsPackageCab          Resource  1.1       gs--t---  Microsoft.Windows/WindowsPowerShell
PSDesiredStateConfiguration/WindowsProcess             Resource  1.1       gs--t---  Microsoft.Windows/WindowsPowerShell
PSDscResources/WindowsFeature                          Resource  2.12.0.0  gs--t---  Microsoft.Windows/WindowsPowerShell  This module contains the standard DSC resources.
PSDscResources/WindowsOptionalFeature                  Resource  2.12.0.0  gs--t---  Microsoft.Windows/WindowsPowerShell  This module contains the standard DSC resources.
PSDscResources/WindowsPackageCab                       Resource  2.12.0.0  gs--t---  Microsoft.Windows/WindowsPowerShell  This module contains the standard DSC resources.
PSDscResources/WindowsProcess                          Resource  2.12.0.0  gs--t---  Microsoft.Windows/WindowsPowerShell  This module contains the standard DSC resources.
```

## Arguments

### RESOURCE_NAME

Specifies an optional filter to apply for the type names of discovered DSC Resources. The filter
may include wildcards (`*`). The filter isn't case-sensitive.

When this argument is specified, DSC filters the results to include only resources where the
resource type name matches the filter.

For example, specifying the filter `Microsoft.*` returns only the resources published by Microsoft.
Specifying the filter `*Sql*` returns any resource with the string `Sql` in its name, regardless of
the casing.

```yaml
Type:      String
Mandatory: false
```

## Options

### -a, --adapter

Specifies a filter to define which adapter resources to enumerate adapted resources for. By
default, the command doesn't call the `list` command for adapter resources. When you specify this
option, DSC looks for adapter resources with type names that match the filter. If it discovers any
adapters matching the filter, it calls the `list` command for those adapters and returns the
adapted resources. DSC retrieves adapted resources before applying any other filters for the
command.

If you specify this option with the filter `*`, DSC calls `list` for every adapter resource it
finds before applying the other filters.

```yaml
Type:      String
Mandatory: false
```

### -d, --description

Specifies a string to match in a resource's description. When this option is specified, DSC filters
the resources by their description strings. The filter is case-insensitive and matches the value
anywhere in the description string. Wildcards aren't permitted.

```yaml
Type:      String
Mandatory: false
```

### -t, --tags

Specifies a resource tag to filter on. When this option is specified, DSC filters the resources and
only includes those with a matching tag. The filter is case-insensitive. Wildcards aren't permitted.

You can specify this option more than once to filter on a set of tags. The results include
resources that have at least one of the tags specified with this option.

```yaml
Type:      String
Mandatory: false
```

### -f, --format

The `--format` option controls the console output format for the command. If the command output is
redirected or captured as a variable, the output is always a series of JSON Lines representing each
returned resource. When this option isn't specified, the output for the command shows a table
representing a summary of the returned resources. For more information, see [Output](#output).

```yaml
Type:         String
Mandatory:    false
DefaultValue: yaml
ValidValues:  [json, pretty-json, yaml]
```

### -h, --help

Displays the help for the current command or subcommand. When you specify this option, the
application ignores all options and arguments after this one.

```yaml
Type:      Boolean
Mandatory: false
```

## Output

This command returns a JSON object for each resource that includes the resource's type, version,
manifest settings, and other metadata. For more information, see
[dsc resource list result schema][02].

If the output of the command isn't captured or redirected, it displays in the console by default as
a summary table for the returned resources. The summary table includes the following columns,
displayed in the listed order:

- **Type** - The fully qualified type name of the resource.
- **Kind** - Whether the resource is an `Adapter`, `Group`, or typical `Resource`. For more
  information, see [DSC Resource kind schema reference][03].
- **Version** - The semantic version of the resource.
- **Caps** - A display of the resource's [capabilities][04] as flags. The capabilities are
  displayed in the following order, using a `-` instead of the appropriate letter if the resource
  doesn't have a specific capability:

  - `g` indicates that the resource has the [Get capability][05].
  - `s` indicates that the resource has the [Set capability][06]
  - `x` indicates that the resource has the [SetHandlesExist capability][07]
  - `w` indicates that the resource has the [WhatIf capability][08]
  - `t` indicates that the resource has the [Test capability][09]
  - `d` indicates that the resource has the [Delete capability][10]
  - `e` indicates that the resource has the [Export capability][11]
  - `r` indicates that the resource has the [Resolve capability][12]

  For example, the `Microsoft.Windows/Registry` resource has the following capabilities: `gs--d-`,
  indicating it has the `Get`, `Set`, and `Delete` capabilities.
- **RequireAdapter** - The fully qualified type name of the adapter resource that DSC uses to
  invoke the returned resource.
- **Description** - The short description of the resource's purpose and usage.

To display the output objects as either JSON or YAML objects in the console, use the
[--format](#-f---format) option.

<!-- Link reference definitions -->
[01]: ../dsc.md#environment-variables
[02]: ../../schemas/outputs/resource/list.md
[03]: ../../schemas/definitions/resourceKind.md
[04]: ../../schemas/outputs/resource/list.md#capabilities
[05]: ../../schemas/outputs/resource/list.md#capability-get
[06]: ../../schemas/outputs/resource/list.md#capability-set
[07]: ../../schemas/outputs/resource/list.md#capability-sethandlesexist
[08]: ../../schemas/outputs/resource/list.md#capability-whatif
[09]: ../../schemas/outputs/resource/list.md#capability-test
[10]: ../../schemas/outputs/resource/list.md#capability-delete
[11]: ../../schemas/outputs/resource/list.md#capability-export
[12]: ../../schemas/outputs/resource/list.md#capability-resolve
