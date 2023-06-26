# Command-based DSC Resource manifest schema reference

Every command-based DSC Resource must have a DSC Resource manifest. The manifest must:

1. Be discoverable in the `PATH` environment variable.
1. Follow the naming convention `<name>.resource.json`.
1. Be valid for the schema described in this document.

The rest of this document describes the DSC Resource manifest schema.

## Metadata

| Metadata Key | Metadata Value                                      |
|:------------:|:----------------------------------------------------|
|  `$schema`   | `https://json-schema.org/draft/2020-12/schema`      |
|    `$id`     | `https://aka.ms/dsc/schemas/resource/manifest.yaml` |
|    `type`    | `object`                                            |

## Required Properties

The manifest must include these properties:

- [manifestVersion](#manifestversion)
- [type](#type)
- [version](#version)
- [get](#get)

## Properties

### manifestVersion

The `manifestVersion` property indicates the semantic version (semver) of the DSC Resource manifest
schema that the DSC Resource manifest validates against. This property is mandatory. DSC uses this
value to validate the manifest against the correct JSON schema.

```yaml
Type:     string
Required: true
Valid Values:
  - '1.0'
```

### type

The `type` property represents the fully qualified the name of the DSC Resource in its namespace.
It's used to specify the DSC Resource in configuration documents and as the value of the
`--resource` flag when using the `dsc resource *` commands. This value must use the following
syntax:

```text
`<owner>[.<group>][.<area>]/<name>`
```

Each component must be string of alphanumeric characters and underscores. No other characters are
permitted. Every DSC Resource must define an `owner` and a `name`. Use the `group` and `area`
components to organize DSC Resources into related namespaces. For example:

- `Microsoft.SqlServer/Database`
- `Microsoft.SqlServer.Database/Role`
- `Microsoft.SqlServer.Database/User`
- `Microsoft.SqlServer/Endpoint`
- `Microsoft.SqlServer.Endpoint/Permission`
- `Microsoft.SqlServer/Login`
- `Microsoft.SqlServer/MaxDop`

```yaml
Type:     string
Required: true
Pattern:  ^\w+(\.\w+){0,2}\/\w+$
```

### version

The `version` property must be the current version of the DSC Resource as a valid semantic
version string. The version applies to the DSC Resource, not the software it manages.

```yaml
Type:     string
Required: true
```

### description

The `description` property defines a synopsis for the DSC Resource's purpose. The value for this
property must be a short string.

```yaml
Type:     string
Required: false
```

### tags

The `tags` property defines a list of searchable terms for the DSC Resource. The value of this
property must be an array of strings. Each tag must contain only alphanumeric characters and
underscores. No other characters are permitted.

```yaml
Type:     array
Required: false
Valid Items:
  Type:    string
  Pattern: ^\w+$
```

### get

The `get` property defines how to call the DSC Resource to get the current state of an instance.
This property is mandatory for all DSC Resources.

The value of this property must be an object. The object's `executable` property, defining the name
of the command to call, is mandatory. The `args` and `input` properties are optional. For more
information, see [DSC Resource manifest get property schema reference][01].

```yaml
Type:     object
Required: true
```

### set

The `set` property defines how to call the DSC Resource to set the desired state of an instance. It
also defines how to process the output from the DSC Resource for this method. When this property
isn't defined, the DSC can't manage instances of the DSC Resource. It can only get their current
state and test whether the instance is in the desired state.

The value of this property must be an object. The `executable` property, defining the name of the
command to call, is mandatory. The `args` `input`, `preTest`, and `returns` properties are
optional. For more information, see [DSC Resource manifest set property schema reference][02].

### test

The `test` property defines how to call the DSC Resource to test whether an instance is in the
desired state. It also defines how to process the output from the DSC Resource for this method.
When this property isn't defined, DSC performs a basic synthetic test for instances of the DSC
Resource.

The value of this property must be an object. The object's `executable` property, defining the name
of the command to call, is mandatory. The `args` `input`, and `returns` properties are optional.
For more information, see [DSC Resource manifest test property schema reference][03].

### validate

The `validate` property defines how to call a DSC Group Resource to validate its instances. This
property is mandatory for DSC Group Resources. DSC ignores this property for all other DSC
Resources.

The value of this property must be an object. The object's `executable` property, defining the name
of the command to call, is mandatory. The `args` property is optional. For more information, see
[DSC Resource manifest validate property schema reference][04].

### provider

When specified, the `provider` property defines the DSC Resource as a DSC Resource Provider.

For more information, see the [DSC Resource manifest provider property schema reference][05].

### exitCodes

The `exitCodes` property defines a set of valid exit codes for the DSC Resource and their meaning.
Define this property as a set of key-value pairs where:

- The key is an integer that maps to a known exit code for the DSC Resource.
- The value is a string describing the semantic meaning of that exit code for a human reader.

DSC always interprets exit code `0` as a successful operation and any other exit code as an error.

```yaml
Type:     object
Required: false
Valid Properties:
  Name Pattern: ^[0-9]+#
  Value Type:   string
```

### schema

The `schema` property defines how DSC should get the JSON schema to validate an instance of the DSC
Resource. This property must always be an object that defines one of the following properties:

- `command` - When you specify the `command` property, DSC calls the defined command to get the
  JSON schema.
- `embedded` - When you specify the `embedded` property, DSC uses the defined value as the JSON
  schema.

For more information, see [DSC Resource manifest schema property reference][06].

```yaml
Type:     object
Required: true
```

[01]: ./methods/get.md
[02]: ./methods/set.md
[03]: ./methods/test.md
[04]: ./methods/validate.md
[05]: ./provider-property.md
[06]: ./schema-property.md
