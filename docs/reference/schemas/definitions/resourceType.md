---
description: JSON schema reference for a resource instance type name
ms.date:     09/01/2026
ms.topic:    reference
title:       DSC Resource fully qualified type name schema reference
---

# DSC Resource fully qualified type name schema reference

## Synopsis

Identifies a DSC Resource.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/definitions/resourceType.json
Type:          string
Pattern:       ^\w+(\.\w+)*\/\w+$
```

## Description

DSC Resources are identified by their fully qualified type name. Values of this type are used to
specify a resource in configuration documents and as the value of the `--resource` flag when
using the `dsc resource *` commands. DSC extensions use the same syntax for their type names.

The fully qualified type name of a resource uses the following syntax:

```text
<owner>[.<namespace>]*/<name>
```

The portion of the type name before the forward slash (`/`) is the namespace. It consists of one
or more segments separated by a single period (`.`). The first segment is always the owner. Any
following segments organize related resources into groups and areas. The portion after the `/` is
the name.

Each segment must be a string of alphanumeric characters and underscores. No other characters are
permitted. Periods can't appear at the start or end of the namespace, and two periods can't appear
consecutively. Every resource must define an `owner` and a `name`. DSC doesn't limit the number of
segments in the namespace. For example:

- `Microsoft/OSInfo`
- `Microsoft.SqlServer/Database`
- `Microsoft.SqlServer.Database/Role`
- `Microsoft.SqlServer.Database/User`
- `Microsoft.SqlServer/Endpoint`
- `Microsoft.SqlServer.Endpoint/Permission`
- `Microsoft.SqlServer/Login`
- `Microsoft.SqlServer/MaxDop`

DSC compares fully qualified type names without regard to case. For example, DSC treats
`Microsoft/OSInfo` and `microsoft/osinfo` as the same type name.

Earlier versions of the schema limited the number of namespace segments. The schemas published for
DSC 3.0 allow up to three segments and the schemas published for DSC 3.1 allow up to four. Starting
with DSC 3.2.0, the number of segments is unlimited.

## Type name segments

### Owner

The owner segment of the type name is mandatory. It's always the first segment of the namespace.
It identifies the person or organization that owns, develops, and maintains the resource.

### Group and area

The segments after the owner are optional. Use them to organize resources into related namespaces.
By convention, the first segment after the owner is the _group_. It defines a logical grouping for
a collection of resources. For example, resources that manage SQL Server might use the `SqlServer`
group in their type name.

The next segment is the _area_. It defines a grouping for a collection of resources by purpose or
higher-level component. For example, resources that manage components of a SQL Server database
might use the `Database` area in their type name.

You can add further segments after the area when you need to subdivide a namespace further.

### Name

The name segment of the type name is mandatory. It identifies the component that the resource
manages. This segment should be a singular noun unless the resource always manages a list of
components in a single resource instance. In that case, the resource name should be the plural form
of the noun it manages or the singular form of the noun followed by the word `List`, like
`JeaRoleCapabilities` or `JeaRoleCapabilityList`.
