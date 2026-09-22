---
description: >-
  Considerations and guidance for defining metadata in a resource manifest.
ms.date: 08/15/2025
title: Defining DSC resource manifest metadata
---

# Defining DSC resource manifest metadata

DSC relies on metadata defined in the resource manifest to identify and describe each resource. The
following metadata fields are available for a resource manifest:

- `type` (required) - The fully qualified type name for the resource. DSC uses this field to
  uniquely identify a specific resource.
- `version` (required) - The semantic version of the resource. By default, when DSC finds more than
  one manifest for a resource with the same `type`, DSC uses the newest version of that resource.
- `description` (optional, recommended) - Defines a short human-readable description for the
  resource. This value is surfaced from the `dsc resource list` command and
  can be used as a filter when searching for resources.
- `tags` (optional, recommended) - Defines one or more strings that indicate usage for the
  resource. This value is surfaced from the `dsc resource list` command and can be used as a filter
  when searching for resources.

## Defining the resource type name

DSC distinguishes resources by their fully qualified type name. A resource's fully qualified type
name uses namespacing to distinguish between resources and uniquely identify them.

The syntax for a fully qualified type name is:

```Syntax
<owner>[.<group>][.<area>][.<subarea>]/<name>
```

- `<owner>` is a required component that indicates which person or organization is responsible for
  implementing and maintaining the resource.
- `<group>`, `<area>`, and `<subarea>` are optional values to create namespaces under the owner for
  organizing resources. These components of the type name are optional.
- `<name>` is a required component that indicates the resource's purpose and usage. The `<name>` is
  frequently used as shorthand for discussing a resource, like `OSInfo` instead of
  `Microsoft/OSInfo`.

The regular expression pattern DSC uses to validate type names is:

```regex
^\w+(\.\w+){0,3}\/\w+$
```

When you define the fully qualified type name for your resource, follow these guidelines:

1. Always define `<owner>` for the resource.
1. Consider defining at least one namespace segment for the resource to ensure that related
   resources can be grouped together and you can avoid name collisions for future resources.
1. When defining resources for a specific platform, indicate the platform as a namespace segment.
1. Define `<name>` as a noun. The noun should semantically align with the component that the
   resource manages, such as:

   - `Package` for a resource that installs, updates, and removes software packages.
   - `User` for a resource that creates, updates, and removes users from a system.
   - `Timezone` for a resource that sets the timezone for a system.

Example type names:

- `Microsoft.Linux.Apt/Package`
- `Microsoft.Python/Package`
- `Microsoft.Linux/User`
- `Microsoft.Windows/Timezone`
- `Microsoft.SqlServer/Database`
- `Microsoft.SqlServer.Database/Role`
- `TSToy.CLI/SettingsFile`

## Defining the resource version

The `version` field in a resource manifest defines the semantic version
([semver](https://semver.org/)) of the DSC resource. This version identifies the resource, not the
version of the application it manages.

Follow semantic versioning guidance for versioning your resource. Always increment the major
version of your resource for breaking changes.

By default, DSC uses the latest nonprerelease version of an available resource on a system.

## Defining the resource description

The `description` field in a resource manifest defines a short synopsis of the resource's purpose.

The description is surfaced from the [dsc resource list](../../../reference/cli/resource/list.md)
command. Users can specify the
[--description](../../../reference/cli/resource/list.md#--description) option to filter resources
by their description.

When you define the description for your resource, follow these guidelines:

- Keep the description short, no more than 80 characters.
- Don't use any newlines in the description.
- Write the description as a complete sentence in the imperative tense to indicate what you can use
  the resource to do.
- If the resource doesn't implement set, start the description with `Returns information`.
- If the description requires any markup, use Markdown syntax sparingly. Only use inline syntax.

Examples:

- Manage Windows registry keys and values.
- Return information about the operating system.
- Adapt PowerShell DSC (PSDSC) resources implemented as PowerShell classes.
- Manage software packages with `apt`.

## Defining the resource tags

The `tags` field defines a list of searchable terms for the resource. Each tag must be a string of
alphanumeric characters and underscores. No other characters are permitted.

There are a few conventional tags for resources with specific semantics:

- `Windows` - indicates that the resource is functional on Windows systems.
- `Linux` - indicates that the resource is functional on Linux systems.
- `macOS` - indicates that the resource is functional on macOS systems.

When you define the tags for your resource, follow these guidelines:

- Include any conventional tags that apply to your resource.
- Define a tag for every term you think users will frequently search for that should return your
  resource.

## Related content

- [Designing a DSC resource](./index.md)
- [Command-based DSC Resource manifest schema reference](../../../reference/schemas/resource/manifest/root.md)
