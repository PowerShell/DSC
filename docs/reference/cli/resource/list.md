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

If any of the discovered resources are resource providers, DSC then calls the providers to list
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
Type                        Version  Methods                 Requires        Description
----------------------------------------------------------------------------------------------------------------------------------------------------
DSC.PackageManagement/Brew  0.1.0    get, set, export                        DSC resource to manage Homebrew packages
DSC/AssertionGroup          0.1.0    get, set, test                          `test` will be invoked for all resources in the supplied configuration.
DSC/Group                   0.1.0    get, set, test                          All resources in the supplied configuration is treated as a group.     
DSC/ParallelGroup           0.1.0    get, set, test                          All resources in the supplied configuration run concurrently.
DSC/PowerShellGroup         0.1.0    get, set, test, export                  Resource provider to classic DSC Powershell resources.
Microsoft.Windows/Registry  0.1.0    get, set, test                          Registry configuration provider for the Windows Registry
Microsoft/OSInfo            0.1.0    get, export                             Returns information about the operating system.
Microsoft/Process           0.1.0    get, set, test, export                  Returns information about running processes.
Test/Echo                   0.1.0    get, set, test
Test/Sleep                  0.1.0    get, set, test
Test/TestGroup              0.1.0    get
Test/TestResource1          1.0.0    get                     Test/TestGroup  This is a test resource.
Test/TestResource2          1.0.1    get                     Test/TestGroup  This is a test resource.
```

### Example 2 - List a specific resource

When the `RESOURCE_NAME` argument doesn't include a wildcard, the command returns only the resource
with the specified type name.

```sh
dsc resource list DSC/Group
```

```Output
Type       Version  Methods         Requires  Description
----------------------------------------------------------------------------------------------------------------
DSC/Group  0.1.0    get, set, test            All resources in the supplied configuration is treated as a group.
```

### Example 3 - List resources with a matching type name

When the `RESOURCE_NAME` argument includes a wildcard, the command returns every resource with a
matching type name.

```sh
dsc resource list DSC/*
```

```Output
Type                 Version  Methods                 Requires  Description
---------------------------------------------------------------------------------------------------------------------------------------
DSC/AssertionGroup   0.1.0    get, set, test                    `test` will be invoked for all resources in the supplied configuration.
DSC/Group            0.1.0    get, set, test                    All resources in the supplied configuration is treated as a group.
DSC/ParallelGroup    0.1.0    get, set, test                    All resources in the supplied configuration run concurrently.
DSC/PowerShellGroup  0.1.0    get, set, test, export            Resource provider to classic DSC Powershell resources.
```

### Example 4 - List resources with a matching description

When the command includes the **description** option, the results include resources that have a
description containing the specified value.

```sh
dsc resource list --description 'supplied configuration'
```

```Output
Type                Version  Methods         Requires  Description
------------------------------------------------------------------------------------------------------------------------------
DSC/AssertionGroup  0.1.0    get, set, test            `test` will be invoked for all resources in the supplied configuration.
DSC/Group           0.1.0    get, set, test            All resources in the supplied configuration is treated as a group.
DSC/ParallelGroup   0.1.0    get, set, test            All resources in the supplied configuration run concurrently.
```

### Example 5 - List resources with matching tags

When the command includes multiple instances of the **tags** option, the results include resources
that have any of the specified tags.

```sh
dsc resource list --tags Windows --tags Linux
```

```output
Type                        Version  Methods         Requires  Description
-----------------------------------------------------------------------------------------------------------------------
Microsoft.Windows/Registry  0.1.0    get, set, test            Registry configuration provider for the Windows Registry
Microsoft/OSInfo            0.1.0    get, export               Returns information about the operating system.
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
- **Version** - The semantic version of the resource.
- **Methods** - A comma-separated list of the implemented methods for the resource. Valid methods
  are `get`, `set`, `test`, and `export`. Resources that don't implement `test` rely on DSC's
  synthetic test functionality.
- **Requires** - The fully qualified type name of the provider resource that DSC uses to invoke the
  returned resource.
- **Description** - The short description of the resource's purpose and usage.

To display the output objects as either JSON or YAML objects in the console, use the
[--format](#-f---format) option.

[01]: ../dsc.md#environment-variables
[02]: ../../schemas/outputs/resource/list.md
