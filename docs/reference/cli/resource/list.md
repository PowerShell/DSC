---
description: Command line reference for the 'dsc resource list' command
ms.date:     08/04/2023
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

If any of the discovered resources are resource adapters, DSC then calls the adapters to list
their resources, too.

DSC returns the list of discovered resources with their implementation information and metadata. If
the command includes the `RESOURCE_NAME` argument, DSC filters the list of discovered resources
before returning them. The **description** and **tags** options filter the results by the
resource descriptions and tags.

## Examples

### Example 1 - List all resources

Without any filters, the command returns every discovered DSC Resource.

```sh
dsc resource list
```

```Output
type                       version tags                        description
----                       ------- ----                        -----------
Test/TestGroup             0.1.0
Microsoft/OSInfo           0.1.0   {os, linux, windows, macos} Returns information about the operating system.
Microsoft.Windows/Registry 0.1.0   {Windows, NT}               Registry configuration adapter for the Windows Registry
                                                               This is a test resource.
DSC/PowerShellGroup        0.1.0   {PowerShell}                Resource adapter to classic DSC Powershell resources.
DSC/AssertionGroup         0.1.0                               `test` will be invoked for all resources in the supplied configuration.
DSC/ParallelGroup          0.1.0                               All resources in the supplied configuration run concurrently.
                                                               This is a test resource.
DSC/Group                  0.1.0                               All resources in the supplied configuration is treated as a group.
```

### Example 2 - List a specific resource

When the `RESOURCE_NAME` argument doesn't include a wildcard, the command returns only the resource
with the specified type name.

```sh
dsc resource list DSC/Group
```

```Output
Type       Version  Requires  Description
------------------------------------------------------------------------------------------------
DSC/Group  0.1.0              All resources in the supplied configuration is treated as a group.
```

### Example 3 - List resources with a matching type name

When the `RESOURCE_NAME` argument includes a wildcard, the command returns every resource with a
matching type name.

```sh
dsc resource list DSC/*
```

```Output
Type                 Version  Requires  Description
---------------------------------------------------------------------------------------------------------------
DSC/Group            0.1.0              All resources in the supplied configuration is treated as a group.
DSC/ParallelGroup    0.1.0              All resources in the supplied configuration run concurrently.
DSC/PowerShellGroup  0.1.0              Resource adapter to classic DSC Powershell resources.
DSC/AssertionGroup   0.1.0              `test` will be invoked for all resources in the supplied configuration.
```

### Example 4 - List resources with a matching description

When the command includes the **description** option, the results include resources that have a
description containing the specified value.

```sh
dsc resource list --description 'supplied configuration'
```

```Output
Type                Version  Requires  Description
--------------------------------------------------------------------------------------------------------------
DSC/ParallelGroup   0.1.0              All resources in the supplied configuration run concurrently.
DSC/AssertionGroup  0.1.0              `test` will be invoked for all resources in the supplied configuration.
DSC/Group           0.1.0              All resources in the supplied configuration is treated as a group.
```

### Example 5 - List resources with matching tags

When the command includes multiple instances of the **tags** option, the results include resources
that have any of the specified tags.

```sh
dsc resource list --tags Windows --tags Linux
```

```output
Type                        Version  Requires  Description
-------------------------------------------------------------------------------------------------------
Microsoft.Windows/Registry  0.1.0              Registry configuration adapter for the Windows Registry
Microsoft/OSInfo            0.1.0              Returns information about the operating system.
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
redirected or captured as a variable, the output is always JSON.

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

[01]: ../dsc.md#environment-variables
[02]: ../../schemas/outputs/resource/list.md
