# DSC Resource fully qualified type name schema reference

## Synopsis

Identifies a DSC Resource.

## Metadata

```yaml
Schema Dialect : https://json-schema.org/draft/2020-12/schema
Schema ID      : https://schemas.microsoft.com/dsc/2023/07/definitions/resourceType.yaml
Type           : string
Pattern        : ^\w+(\.\w+){0,2}\/\w+$
```

## Description

DSC Resources are identified by their fully qualified type name. Values of this type are used to
specify a resource in configuration documents and as the value of the `--resource` flag when
using the `dsc resource *` commands.

The fully qualified type name of a resource uses the following syntax:

```text
`<owner>[.<group>][.<area>]/<name>`
```

Each segment must be string of alphanumeric characters and underscores. No other characters are
permitted. Every resource must define an `owner` and a `name`. Use the `group` and `area`
components to organize resources into related namespaces. For example:

- `Microsoft.SqlServer/Database`
- `Microsoft.SqlServer.Database/Role`
- `Microsoft.SqlServer.Database/User`
- `Microsoft.SqlServer/Endpoint`
- `Microsoft.SqlServer.Endpoint/Permission`
- `Microsoft.SqlServer/Login`
- `Microsoft.SqlServer/MaxDop`

## Type name segments

### Owner

The owner segment of the type name is mandatory. It identifies the person or organization that
owns, develops and maintains the resource.

### Group

The group segment of the type name is optional. It defines a logical grouping for a collection of
resources. For example, resources that manage SQL Server might use the `SqlServer` group in their
type name.

### Area

The area segment of the type name is optional. It defines a grouping for a collection of resources
by purpose or higher-level component. For example, resources that manage components of a SQL Server
database might use the `Database` area in their type name.

### Name

The name segment of the type name is mandatory. It identifies the component that the resource
manages. This segment should be a singular noun unless the resource always manages a list of
components in a single resource instance. In that case, the resource name should be the plural form
of the noun it manages or the singular form of the noun followed by the word `List`, like
`JeaRoleCapabilities` or `JeaRoleCapabilityList`.
