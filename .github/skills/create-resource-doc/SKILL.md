---
name: create-resource-doc
description: |
    Create complete, accurate reference documentation for a DSC resource in this repository, following the conventions and structure used in docs/reference/resources/.
---

# Create DSC resource reference documentation

You are a documentation agent. Your task is to create complete, accurate reference documentation
for a DSC resource in this repository.

## Overview

Each resource has a reference documentation page located at:

```
docs/reference/resources/<Namespace>/<ResourceName>/index.md
```

Example files live at:

```
docs/reference/resources/<Namespace>/<ResourceName>/examples/<example-name>.md
docs/reference/resources/<Namespace>/<ResourceName>/examples/<descriptor>.config.dsc.yaml
```

Use the resource manifest (`*.dsc.resource.json`), schema definitions, and source code to gather
accurate information before writing. Use the conventions defined in this prompt. If a convention
is not covered here, follow the patterns in the reference examples listed below:

- `docs/reference/resources/Microsoft/OSInfo/index.md` — read-only resource with no configurable
  properties
- `docs/reference/resources/Microsoft/Windows/Registry/index.md` — configurable resource with
  required, key, and optional properties
- `docs/reference/resources/Microsoft/Windows/RebootPending/index.md` — Windows-only read-only
  resource
- `docs/reference/resources/Microsoft/Adapter/PowerShell/index.md` — adapter resource

---

## index.md — Structure and Conventions

### Front matter

```yaml
---
description: <TypeName> resource reference documentation
ms.date:     MM/DD/YYYY
ms.topic:    reference
title:       <TypeName>
---
```

- Set `ms.date` to the current date in `MM/DD/YYYY` format. Check the conversation context for
  today's date; if not available, use the date from the nearest existing resource document in the
  same directory.
- The `title` and H1 heading must be the fully qualified resource type name (e.g.,
  `Microsoft.Windows/Registry`).

---

### Section order

Write sections in this order. Omit sections that don't apply (e.g., omit **Requirements** if the
resource has none).

1. H1 heading
2. Synopsis
3. Proof-of-concept callout *(optional)*
4. Metadata
5. Instance definition syntax
6. Description
7. Requirements *(optional)*
8. Capabilities
9. Examples
10. Properties
11. Instance validating schema
12. Exit codes
13. See also
14. Link reference definitions

---

### 1. H1 heading

```markdown
# <TypeName>
```

Use the fully qualified resource type name.

---

### 2. Synopsis

```markdown
## Synopsis

<One-sentence description of what the resource does.>
```

---

### 3. Proof-of-concept callout (optional)

If the resource is labelled as a proof-of-concept or example, add this callout immediately after
the synopsis:

```markdown
> [!IMPORTANT]
> The `<executable>` command and `<TypeName>` resource are a proof-of-concept example for use with
> DSC. Don't use it in production.
```

---

### 4. Metadata

Use a YAML fenced code block aligned with spaces:

````markdown
## Metadata

```yaml
Version    : <semver>
Kind       : resource | adapter
Tags       : [<tag1>, <tag2>]
Author     : Microsoft
```
````

For adapter resources, add additional fields:

```yaml
Version           : <semver>
Kind              : adapter
Tags              : [<tag1>, <tag2>]
Executable        : <executable name>
MinimumDSCVersion : <semver>
```

Derive values from the resource manifest (`*.dsc.resource.json`).

---

### 5. Instance definition syntax

Show how to define an instance inside a configuration document. Use YAML, and clearly comment
which properties are required vs optional.

**For standard resources:**

````markdown
## Instance definition syntax

```yaml
resources:
  - name: <instance name>
    type: <TypeName>
    properties:
      # Required properties
      <requiredProp>: <type>
      # Instance properties
      <optionalProp>:
```
````

**For adapter resources**, use two subsections:

````markdown
## Adapted resource instance definition syntax

### Implicitly required adapter syntax

```yaml
- name: <instance name>
  type: <module name>/<resource name>
  properties:
    <property name>: <property value>
```

### Explicitly required adapter syntax

```yaml
- name: <instance name>
  type: <module name>/<resource name>
  properties:
    <property name>: <property value>
  directives:
    requireAdapter: <TypeName>
```
````

---

### 6. Description

```markdown
## Description

<Paragraph describing what the resource does, how it works, and any important behavior.>
```

Include:
- What operations the resource supports and what it does.
- Whether it relies on synthetic testing instead of implementing `test` itself.
- Whether it uses caching, what triggers cache invalidation, and where the cache is stored (if
  applicable).
- Any deprecated predecessors and migration notes.
- Any platform-specific behavior differences.
- Platform-specific installation note if the resource ships with DSC:

  ```markdown
  > [!NOTE]
  > This resource is installed with DSC itself on <all platforms | Windows systems>.
  >
  > You can update this resource by updating DSC. When you update DSC, the updated version of this
  > resource is automatically available.
  ```

---

### 7. Requirements (optional)

Omit this section if the resource has no requirements beyond a standard DSC installation.
Otherwise:

```markdown
## Requirements

- <Requirement 1, e.g., platform constraints>
- <Requirement 2, e.g., permissions>
```

---

### 8. Capabilities

List every capability the resource supports. Use this phrasing pattern:

```markdown
## Capabilities

The resource has the following capabilities:

- `get` - You can use the resource to retrieve the actual state of an instance.
- `set` - You can use the resource to enforce the desired state for an instance.
- `whatIf` - The resource is able to report how it would change system state during a **Set**
  operation in what-if mode.
- `test` - You can use the resource to test whether an instance is in the desired state.
- `delete` - You can use the resource to directly remove an instance from the system.
- `export` - You can use the resource to retrieve the actual state of every instance.
- `list` - You can use the resource to list available adapted resources.
```

Include only the capabilities the resource actually has. Then add explanatory notes:

- If the resource uses synthetic testing, add:
  ```markdown
  This resource uses the synthetic test functionality of DSC to determine whether an instance is
  in the desired state.
  ```
- If the resource lacks `set`, add:
  ```markdown
  This resource doesn't have the `set` capability. You can't use it to modify the state of a system.
  ```
- Close with a cross-reference link:
  ```markdown
  For more information about resource capabilities, see [DSC resource capabilities][<ref>].
  ```

---

### 9. Examples

List examples as a numbered Markdown list, each linking to its example file. Write a brief
description for examples covering more than one scenario.

```markdown
## Examples

1. [<Example title>][<ref>] - <One-sentence description.>
2. [<Example title>][<ref>] - <One-sentence description.>
```

For resources with a single example per operation:

```markdown
1. [<Example title>][<ref>]
```

Define the link references in the **Link reference definitions** section at the bottom of the file.

---

### 10. Properties

```markdown
## Properties

The following list describes the properties for the resource.

- **Required properties:** <a id="required-properties"></a> <sentence about required properties>

  - [<propName>](#<propname>) - <Brief description.>

- **Key properties:** <a id="key-properties"></a> <sentence about key properties>

  - [<propName>](#<propname>) (required | optional) - <Brief description.>

- **Instance properties:** <a id="instance-properties"></a> The following properties are optional.
  They define the desired state for an instance of the resource.

  - [<propName>](#<propname>) - <Brief description.>

- **Read-only properties:** <a id="read-only-properties"></a> The resource returns the following
  properties, but they aren't configurable. For more information about read-only properties, see
  the "Read-only resource properties" section in [DSC resource properties][<ref>].

  - [<propName>](#<propname>) - <Brief description.>
```

Use these standard phrases when there are no properties in a category:
- `This resource doesn't have any required properties.`
- `This resource doesn't have any key properties.`

**Per-property subsections:**

For every property listed above, create a third-level heading and a collapsible metadata block:

````markdown
### <propName>

<details><summary>Expand for <code><propName></code> property metadata</summary>

```yaml
Type             : <json type>
IsRequired       : true | false
IsKey            : true | false
IsReadOnly       : true | false
IsWriteOnly      : false
```

</details>

<Description of the property. Include valid values, default value, and any constraints.>
````

Add `DefaultValue`, `ValidValues`, `ConstantValue`, `RequiresProperties`, or range fields as
applicable. For nested objects or array properties, add sub-subsections (`####`) for each
sub-property, each with their own `<details>` metadata block.

**Concrete example — nested object property:** See the `valueData` property section in
`docs/reference/resources/Microsoft/Windows/Registry/index.md` for a complete, maintained example
showing the `### valueData` property with its `<details>` block, sub-property summary list, and
`#### <subpropName> valueData` sub-subsections each containing their own `<details>` block.

---

### 11. Instance validating schema

Embed the JSON Schema that validates an instance of the resource. Source this from the resource
manifest's `schema.embedded` section or from the schema file referenced by `schema.url`.

Omit the following metadata-only keywords: `title`, `description`, `$id`, `$schema`,
`$comment`. Keep all keywords that constrain values: `type`, `required`, `properties`, `enum`,
`pattern`, `minimum`, `maximum`, `additionalProperties`, `dependentRequired`, `readOnly`,
`default`, `items`, `minProperties`, `maxProperties`, `minItems`, `maxItems`.

````markdown
## Instance validating schema

The following snippet contains the JSON Schema that validates an instance of the resource.

```json
{
  "type": "object",
  ...
}
```
````

---

### 12. Exit codes

```markdown
## Exit codes

The resource returns the following exit codes from operations:

- [0](#exit-code-0) - <description>
- [1](#exit-code-1) - <description>
...

### Exit code 0

Indicates the resource operation completed without errors.

### Exit code 1

<Description of the error condition and what to do.>
```

Source exit code descriptions from the resource manifest `exitCodes` field. For each non-zero exit
code, describe the error condition and whether the resource emits an error message.

---

### 13. See also

```markdown
## See also

- [<Related resource or doc>][<ref>]
```

---

### 14. Link reference definitions

Place all reference-style links at the bottom of the file, after all content:

```markdown
<!-- Link definitions -->
[01]: <url or relative path>
[02]: <url or relative path>
```

Use sequential numeric labels (`[01]`, `[02]`, ...). Use workspace-relative paths for links to
other files in this repository. Use `/en-us/...` absolute paths only for links to published
Microsoft Learn pages that are NOT part of this repository. Use `https://aka.ms/...` short links
for schema references.

Relative path examples:
- Sibling file: `./examples/example-name.md`
- Parent directory: `../other-resource/index.md`
- Concepts: `../../../../../concepts/resources/capabilities.md`
- CLI reference: `../../../../cli/resource/get.md`

---

## Example files — Structure and Conventions

Each example document under `examples/` covers one or two focused scenarios.

### Front matter

```yaml
---
description: >
  <One or two sentences summarizing the example.>
ms.date: MM/DD/YYYY
ms.topic: reference
title: <Example title>
---
```

Use a folded block scalar (`>`) for the description when it spans multiple lines.

### Body

````markdown
# <Example title>

## Description  *(optional — use only for examples that need extra context)*

<Brief description.>

## <Scenario heading>

<Setup or context sentence.>

# [Linux](#tab/linux)

```bash
<command>
```

# [macOS](#tab/macos)

```zsh
<command>
```

# [Windows](#tab/windows)

```powershell
<command>
```

---

<Expected output or explanation.>

```yaml
<output>
```

<!-- Link reference definitions -->
[01]: <path>
````

- Use platform tabs (`# [Linux](#tab/linux)`, `# [macOS](#tab/macos)`, `# [Windows](#tab/windows)`)
  when the command differs by platform. Use a single PowerShell block when the example only
  applies to Windows.
- Show representative output after each command.
- Use `dsc resource get`, `dsc resource set`, `dsc resource test`, or `dsc config` commands in
  examples. Don't invoke the resource executable directly.

### DSC YAML config files

For examples that reference a configuration document, place the YAML file in the `examples/`
folder alongside the example Markdown file. Name it using the pattern
`<descriptor>.config.dsc.yaml` or `<descriptor>.cleanup.config.dsc.yaml`.

---

## Gathering information

Before writing, collect the following from the resource source:

| Information | Source |
|-------------|--------|
| Type name, version, tags, kind | Resource manifest (`*.dsc.resource.json`) |
| Capabilities (get/set/test/export/delete/whatIf) | Resource manifest operation sections |
| Exit codes | Resource manifest `exitCodes` field |
| Property names, types, required/key/readOnly flags | Resource manifest `schema.embedded` or linked schema file |
| Platform restrictions | Resource manifest or source code |
| Description text | Existing source comments, README, or changelog |

If the resource manifest is missing or incomplete, report the specific missing information and do
not fabricate values. If the manifest conflicts with the source code, prefer the manifest and note
the discrepancy.

---

## Checklist

Before completing the documentation:

- [ ] Front matter has `description`, `ms.date`, `ms.topic: reference`, and `title`.
- [ ] H1 heading matches the resource type name exactly.
- [ ] Metadata code block values match the resource manifest.
- [ ] Instance definition syntax shows all required and notable optional properties.
- [ ] Capabilities section lists only capabilities that exist in the manifest.
- [ ] Every property listed in the summary also has a named subsection with a `<details>` block.
- [ ] Property metadata values (type, required, key, readOnly) match the schema.
- [ ] Instance validating schema is valid JSON and matches the manifest schema.
- [ ] Exit codes match the manifest `exitCodes` field.
- [ ] All link references used in the document body are defined at the bottom.
- [ ] Relative paths are correct and use forward slashes.
- [ ] At least one example file exists and is linked from the **Examples** section.
