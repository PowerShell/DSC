# dsc resource list result schema reference

## Synopsis

The result output from the `dsc resource list` command.

## Metadata

```yaml
Schema Dialect : https://json-schema.org/draft/2020-12/schema
Schema ID      : https://schemas.microsoft.com/dsc/2023/07/results/resource/list.yaml
Type           : object
```

## Description

The output from the `dsc resource list` command includes a representation of discovered DSC
Resources as a series of [JSON Lines][01]. This schema describes the JSON object returned for each
resource.

## Required properties

Each resource in the output always includes these properties:

- [type](#type)
- [version](#version)
- [path](#path)
- [directory](#directory)
- [implementedAs](#implementedas)
- [author](#author)
- [properties](#properties)
- [requires](#requires)
- [manifest](#manifest)

## Properties

### type

Identifies the fully qualified type name of the resource. It's used to specify the resource in
configuration documents and as the value of the `--resource` flag when using the `dsc resource *`
commands. For more information about resource type names, see
[DSC Resource fully qualified type name schema reference][02].

```yaml
Type:     string
Required: true
Pattern:  ^\w+(\.\w+){0,2}\/\w+$
```

### version

Represents the current version of the resource as a valid semantic version (semver) string. The
version applies to the resource, not the software it manages.

```yaml
Type:     string
Required: true
Pattern:  ^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-((?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+([0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?$
```

### description

Defines a synopsis for the resource's purpose as a short string. If the resource doesn't have a
description, this property is `null`.

```yaml
Type:     [string, 'null']
Required: true
```

### path

Represents the path to the resource's manifest on the machine. For resources made available through
a provider, this property identifies the path to the file that defines the resource instead.

```yaml
Type:     string
Required: true
```

### directory

Represents the path to the folder containing the resource's manifest on the machine. For resources
made available through a provider, this property identifies the path to the folder containing the
file that defines the resource instead.

```yaml
Type:     string
Required: true
```

### implementedAs

Indicates how the DSC Resource was implemented. For command-based resources, this value is always
`Command`.

<!--
    Resources currently return this a null except for the test resources. Not
    sure how to document this.
-->

### author

Indicates the name of the person or organization that developed and maintains the DSC Resource. If
this property is `null`, the author is unknown.

<!--
    Resources currently return this a null except for the test resources. Is
    this only for provider resources, or something else?
-->

```yaml
Type:     [string, 'null']
Required: true
```

### properties

Defines the property names for resources made available through a provider resource. For other
resources, this property is an empty array.

<!--
    Resources currently return this a null except for the test resources.
    Should this value be populated for all resources made available through a
    provider?
-->

```yaml
Type:          array
Required:      true
Items Type:    string
Items Pattern: ^\w+$
```

### requires

Defines the fully qualified type name of the provider resource that this resource is made available
through.

```yaml
Type:     [string, 'null']
Required: true
```

### manifest

Represents the values defined in the resource's manifest. This value is `null` for resources that
aren't command-based. For more information on the value for this property, see
[Command-based DSC Resource manifest schema reference][03].

```yaml
Type:     [object, 'null']
Required: true
```

[01]: https://jsonlines.org/
[02]: ../../definitions/resourceType.md
[03]: ../../resource/manifest/root.md
