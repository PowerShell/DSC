---
description: Reference for how DSC schemas are versioned and published and the URIs used to retrieve them.
ms.date:     02/28/2025
ms.topic:    reference
title:       DSC JSON Schema URIs
---

# DSC JSON Schema URIs

This document describes how the JSON Schemas are versioned and published for the Microsoft Desired
State Configuration (DSC) platform.

DSC uses JSON schemas extensively to describe and validate the data that it takes as input and
returns as output. To ensure compatibility and simplify the user experience, DSC schemas are
published in multiple versions and forms.

The URIs for DSC schemas use the following syntax:

```syntax
<uri-prefix>/<version-folder>/<form-path-prefix?>/<path>.<form-extension>
```

## Schema URI prefixes

The schemas for DSC are hosted in the `schemas` folder of the DSC repository. The URI prefix for
accessing the schemas in GitHub is `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas`.

However, DSC also provides short links for every schema URI. When using the short link to a schema,
the URI prefix is `https://aka.ms/dsc/schemas`.

You can use either prefix in your configuration documents, resource manifests, or when retrieving
the schemas programmatically.

## Schema Versioning

DSC uses [semantic versioning](https://semver.org) and aligns the version of the CLI, the platform,
and the JSON schemas. A non-prerelease semantic version includes three segments:

```syntax
<major>.<minor>.<patch>
```

- When the next release of DSC contains only fixes, not improvements or breaking changes, the
  `patch` segment increments by one.
- When the next release of DSC contains any improvements and doesn't include any breaking changes,
  the `minor` segment increments by one.
- When the next release of DSC contains any breaking changes, the `major` segment of the version
  increments by one.

### Version folders

For every release of DSC, the schemas are published to three versioned folders:

- `v<major>.<minor>.<patch>` - The full semantic version folder. This folder is unique to each
  release.
- `v<major>.<minor>` - The minor version folder for the current major version. The schemas in this
  folder are always for the latest patch release of that minor version.
- `v<major>` - The major version folder. The schemas in this folder are always for the latest
  release of that major version.

To illustrate the versioning, the following table shows which folders the schemas for each release
publish to. Entries in the table with an asterisk suffix (`*`) indicate that the entry is the
latest schema for that version folder.

| Release | Major version folder | Minor version folder | Full version folder |
|:-------:|:--------------------:|:--------------------:|:-------------------:|
| `3.0.0` |         `v3`         |        `v3.0`        |      `v3.0.0*`      |
| `3.0.1` |         `v3`         |       `v3.0*`        |      `v3.0.1*`      |
| `3.1.0` |         `v3`         |        `v3.1`        |      `v3.1.0*`      |
| `3.1.1` |        `v3*`         |       `v3.1*`        |      `v3.1.1*`      |

### Pinning to a version folder

Publishing the schemas under multiple version folders enables you to choose which version you want
to use for your resource manifests, configuration documents, and integrating tools.

If you pin to a full semantic version folder, like `v3.0.0`, you're pinning to schemas that won't
change. However, to take advantage of any improvements or fixes to the schemas, you'll need to
update the URI with each release.

If you pin to a minor version folder, like `v3.0`, the schemas you use will update with every patch
release. This enables you to take advantage of fixes to the schemas without continually updating
your schema URIs. However, to take advantage of any improvements or new features, you'll need to
update the URI whenever a new minor version is released.

If you pin to a major version folder, like `v3`, the schemas you use will update with every
non-breaking release. You can use those schemas until you want or need to migrate to a new major
version of DSC.

Microsoft recommends that the majority of users pin to the major version folder for ease of use. If
you're an integrating developer or a resource author, consider pinning to a specific minor version
to indicate that your resource or software hasn't been updated to take advantage of new features.

## Schema forms

The schemas for DSC are always published in their canonical form, where the schema lives at its own
URI. Schemas for top-level items, like configuration documents, resource manifests, and the output
types for DSC, are also published in their canonically bundled form and in their enhanced authoring
form.

All JSON schemas published for DSC use the [2020-12 JSON Schema Specification][xx] unless otherwise
noted, regardless of their form.

The canonical (non-bundled) form schemas don't have a prefix folder for their path. They always use
the `.json` file extension for their URI. For example, the URI for the canonical schema describing
a resource manifest is `<uri-prefix>/<version-folder>/resource/manifest.json`. The `$id` keyword
for every schema is always set to the canonical form of the schema for that version folder and uses
the GitHub URI prefix. This ensures that the schemas can always be correctly resolved by reference.

The canonically bundled form schemas are placed in the `bundled` prefix folder for their path. They
always use the `.json` file extension for their URI. For example, the URI for the canonically
bundled schema describing a resource manifest is
`<uri-prefix>/<version-folder>/bundled/resource/manifest.json`.

The enhanced authoring form for schemas are placed in the `bundled` prefix folder for their path.
They always use the `.vscode.json` file extension for their URI. For example, the URI for the
enhanced authoring schema describing a resource manifest is
`<uri-prefix>/<version-folder>/bundled/resource/manifest.vscode.json`.

The following table illustrates these differences between schema forms:

| Schema form             | Prefix folder | File extension |
|:------------------------|:-------------:|:--------------:|
| Canonical (non-bundled) |    _None_     |    `.json`     |
| Canonically bundled     |   `bundled`   |    `.json`     |
| Enhanced autoring       |   `bundled`   | `.vscode.json` |

### Canonical (non-bundled) schemas

The canonical form for each schema describes a single type for DSC. If the schema references any
other DSC types with the [$ref keyword](), those references are site-relative. Publishing the
schemas in this format enables users and developers to select only the schemas for the data types
they want to use without needing to download or handle other schemas they may not require.

While DSC is able to validate any of its data without network connectivity, be aware that using the
canonical non-bundled form for a schema may require more than one network call to retrieve any
references schemas. To minimize the number of network operations, use the canonically bundled form
for the schema instead.

### Canonically bundled schemas

Not every DSC schema is available in the
[canonically bundled](https://json-schema.org/blog/posts/bundling-json-schema-compound-documents)
form. Only top-level schemas, like configuration documents, resource manifests, and DSC's output,
are published in this form.

Canonically bundled schemas generally reference numerous other schemas with the [$ref keyword]().
As with the non-bundled form, these references are site-relative. Unlike the non-bundled form,
the bundled form recursively includes every referenced schema in the `$defs` keyword.

The `$defs` keyword is always an object. For canonically bundled schemas, every key is the
canonical URI to a referenced schema. The value for each key is the schema object hosted at that
URI. Each of the schemas bundled in the `$defs` keyword always defines both the `$id` keyword and
the `$schema` keyword.

### Enhanced authoring schemas

Every DSC Schema published in the canonically bundled form is also published in the enhanced
authoring form. These schemas leverage the extended vocabulary that VS Code recognizes for JSON
Schemas to provide improved IntelliSense, hover documentation, error messaging, and default
snippets. These schemas make it easier to author, edit, and review your configuration documents,
resource manifests, and DSC's output in VS Code.

These schemas validate the data with the same vocabulary as the canonical forms of the schema. They
only affect the experience for authoring, editing, and reviewing the data in VS Code.

These JSON Schemas are _not_ canonical. They use a vocabulary that most JSON Schema libraries and
tools don't understand. In most cases, using these schemas with those tools should not raise any
errors. However, when you want to use the DSC schemas with tools other than VS Code, you should
consider using the canonically bundled form of the schema instead.

## Bundled schema URIs list

This section enumerates every schema published in the canonically bundled form for DSC and the URIs
recognized for each schema.

<!--

TODO: Get data for schemas into usable formats for external consumers

If you would prefer to work with the data directly, you can retrieve the JSON listing of recognized
schemas from the following uri:
`https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/recognizedUris.json`

-->

### Configuration document schema

The following table defines the value of the `$id` keyword for each published version of the
configuration document schema. The `$id` is the same across all forms of the schema and regardless of
the prefix URI used to retrieve the schema.

| Version folder | ID                                                                                          |
|:---------------|:--------------------------------------------------------------------------------------------|
| `v3`           | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/config/document.json`     |
| `v3.0`         | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/config/document.json`   |
| `v3.0.0`       | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/config/document.json` |

The following list of tables defines the recognized URIs for the configuration document schema:

- Short URIs by version and form:

  | Form                    | Version  | Recognized URI                                                          |
  |:------------------------|:---------|:------------------------------------------------------------------------|
  | Canonical (non-bundled) | `v3`     | `https://aka.ms/dsc/schemas/v3/config/document.json`                    |
  | Canonical (non-bundled) | `v3.0`   | `https://aka.ms/dsc/schemas/v3.0/config/document.json`                  |
  | Canonical (non-bundled) | `v3.0.0` | `https://aka.ms/dsc/schemas/v3.0.0/config/document.json`                |
  | Canonically bundled     | `v3`     | `https://aka.ms/dsc/schemas/v3/bundled/config/document.json`            |
  | Canonically bundled     | `v3.0`   | `https://aka.ms/dsc/schemas/v3.0/bundled/config/document.json`          |
  | Canonically bundled     | `v3.0.0` | `https://aka.ms/dsc/schemas/v3.0.0/bundled/config/document.json`        |
  | Enhanced AUthoring      | `v3`     | `https://aka.ms/dsc/schemas/v3/bundled/config/document.vscode.json`     |
  | Enhanced AUthoring      | `v3.0`   | `https://aka.ms/dsc/schemas/v3.0/bundled/config/document.vscode.json`   |
  | Enhanced AUthoring      | `v3.0.0` | `https://aka.ms/dsc/schemas/v3.0.0/bundled/config/document.vscode.json` |

- GitHub URIs by version and form:

  | Form                    | Version  | Recognized URI                                                                                             |
  |:------------------------|:---------|:-----------------------------------------------------------------------------------------------------------|
  | Canonical (non-bundled) | `v3`     | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/config/document.json`                    |
  | Canonical (non-bundled) | `v3.0`   | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/config/document.json`                  |
  | Canonical (non-bundled) | `v3.0.0` | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/config/document.json`                |
  | Canonically bundled     | `v3`     | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/bundled/config/document.json`            |
  | Canonically bundled     | `v3.0`   | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/bundled/config/document.json`          |
  | Canonically bundled     | `v3.0.0` | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/bundled/config/document.json`        |
  | Enhanced AUthoring      | `v3`     | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/bundled/config/document.vscode.json`     |
  | Enhanced AUthoring      | `v3.0`   | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/bundled/config/document.vscode.json`   |
  | Enhanced AUthoring      | `v3.0.0` | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/bundled/config/document.vscode.json` |

### Resource manifest schema

The following table defines the value of the `$id` keyword for each published version of the
resource manifest schema. The `$id` is the same across all forms of the schema and regardless of
the prefix URI used to retrieve the schema.

| Version folder | ID                                                                                            |
|:---------------|:----------------------------------------------------------------------------------------------|
| `v3`           | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/resource/manifest.json`     |
| `v3.0`         | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/resource/manifest.json`   |
| `v3.0.0`       | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/resource/manifest.json` |

The following list of tables defines the recognized URIs for the resource manifest schema:

- Short URIs by version and form:

  | Form                    | Version  | Recognized URI                                                            |
  |:------------------------|:---------|:--------------------------------------------------------------------------|
  | Canonical (non-bundled) | `v3`     | `https://aka.ms/dsc/schemas/v3/resource/manifest.json`                    |
  | Canonical (non-bundled) | `v3.0`   | `https://aka.ms/dsc/schemas/v3.0/resource/manifest.json`                  |
  | Canonical (non-bundled) | `v3.0.0` | `https://aka.ms/dsc/schemas/v3.0.0/resource/manifest.json`                |
  | Canonically bundled     | `v3`     | `https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.json`            |
  | Canonically bundled     | `v3.0`   | `https://aka.ms/dsc/schemas/v3.0/bundled/resource/manifest.json`          |
  | Canonically bundled     | `v3.0.0` | `https://aka.ms/dsc/schemas/v3.0.0/bundled/resource/manifest.json`        |
  | Enhanced AUthoring      | `v3`     | `https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.vscode.json`     |
  | Enhanced AUthoring      | `v3.0`   | `https://aka.ms/dsc/schemas/v3.0/bundled/resource/manifest.vscode.json`   |
  | Enhanced AUthoring      | `v3.0.0` | `https://aka.ms/dsc/schemas/v3.0.0/bundled/resource/manifest.vscode.json` |

- GitHub URIs by version and form:

  | Form                    | Version  | Recognized URI                                                                                               |
  |:------------------------|:---------|:-------------------------------------------------------------------------------------------------------------|
  | Canonical (non-bundled) | `v3`     | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/resource/manifest.json`                    |
  | Canonical (non-bundled) | `v3.0`   | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/resource/manifest.json`                  |
  | Canonical (non-bundled) | `v3.0.0` | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/resource/manifest.json`                |
  | Canonically bundled     | `v3`     | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/bundled/resource/manifest.json`            |
  | Canonically bundled     | `v3.0`   | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/bundled/resource/manifest.json`          |
  | Canonically bundled     | `v3.0.0` | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/bundled/resource/manifest.json`        |
  | Enhanced AUthoring      | `v3`     | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/bundled/resource/manifest.vscode.json`     |
  | Enhanced AUthoring      | `v3.0`   | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/bundled/resource/manifest.vscode.json`   |
  | Enhanced AUthoring      | `v3.0.0` | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/bundled/resource/manifest.vscode.json` |

### Output schema for dsc config get command

The following table defines the value of the `$id` keyword for each published version of the output
schema for the `dsc config get` command. The `$id` is the same across all forms of the schema and
regardless of the prefix URI used to retrieve the schema.

| Version folder | ID                                                                                            |
|:---------------|:----------------------------------------------------------------------------------------------|
| `v3`           | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/outputs/config/get.json`     |
| `v3.0`         | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/outputs/config/get.json`   |
| `v3.0.0`       | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/outputs/config/get.json` |

The following list of tables defines the recognized URIs for the output schema:

- Short URIs by version and form:

  | Form                    | Version  | Recognized URI                                                            |
  |:------------------------|:---------|:--------------------------------------------------------------------------|
  | Canonical (non-bundled) | `v3`     | `https://aka.ms/dsc/schemas/v3/outputs/config/get.json`                    |
  | Canonical (non-bundled) | `v3.0`   | `https://aka.ms/dsc/schemas/v3.0/outputs/config/get.json`                  |
  | Canonical (non-bundled) | `v3.0.0` | `https://aka.ms/dsc/schemas/v3.0.0/outputs/config/get.json`                |
  | Canonically bundled     | `v3`     | `https://aka.ms/dsc/schemas/v3/bundled/outputs/config/get.json`            |
  | Canonically bundled     | `v3.0`   | `https://aka.ms/dsc/schemas/v3.0/bundled/outputs/config/get.json`          |
  | Canonically bundled     | `v3.0.0` | `https://aka.ms/dsc/schemas/v3.0.0/bundled/outputs/config/get.json`        |
  | Enhanced AUthoring      | `v3`     | `https://aka.ms/dsc/schemas/v3/bundled/outputs/config/get.vscode.json`     |
  | Enhanced AUthoring      | `v3.0`   | `https://aka.ms/dsc/schemas/v3.0/bundled/outputs/config/get.vscode.json`   |
  | Enhanced AUthoring      | `v3.0.0` | `https://aka.ms/dsc/schemas/v3.0.0/bundled/outputs/config/get.vscode.json` |

- GitHub URIs by version and form:

  | Form                    | Version  | Recognized URI                                                                                               |
  |:------------------------|:---------|:-------------------------------------------------------------------------------------------------------------|
  | Canonical (non-bundled) | `v3`     | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/outputs/config/get.json`                    |
  | Canonical (non-bundled) | `v3.0`   | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/outputs/config/get.json`                  |
  | Canonical (non-bundled) | `v3.0.0` | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/outputs/config/get.json`                |
  | Canonically bundled     | `v3`     | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/bundled/outputs/config/get.json`            |
  | Canonically bundled     | `v3.0`   | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/bundled/outputs/config/get.json`          |
  | Canonically bundled     | `v3.0.0` | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/bundled/outputs/config/get.json`        |
  | Enhanced AUthoring      | `v3`     | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/bundled/outputs/config/get.vscode.json`     |
  | Enhanced AUthoring      | `v3.0`   | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/bundled/outputs/config/get.vscode.json`   |
  | Enhanced AUthoring      | `v3.0.0` | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/bundled/outputs/config/get.vscode.json` |

### Output schema for dsc config set command

The following table defines the value of the `$id` keyword for each published version of the output
schema for the `dsc config set` command. The `$id` is the same across all forms of the schema and
regardless of the prefix URI used to retrieve the schema.

| Version folder | ID                                                                                            |
|:---------------|:----------------------------------------------------------------------------------------------|
| `v3`           | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/outputs/config/set.json`     |
| `v3.0`         | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/outputs/config/set.json`   |
| `v3.0.0`       | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/outputs/config/set.json` |

The following list of tables defines the recognized URIs for the output schema:

- Short URIs by version and form:

  | Form                    | Version  | Recognized URI                                                            |
  |:------------------------|:---------|:--------------------------------------------------------------------------|
  | Canonical (non-bundled) | `v3`     | `https://aka.ms/dsc/schemas/v3/outputs/config/set.json`                    |
  | Canonical (non-bundled) | `v3.0`   | `https://aka.ms/dsc/schemas/v3.0/outputs/config/set.json`                  |
  | Canonical (non-bundled) | `v3.0.0` | `https://aka.ms/dsc/schemas/v3.0.0/outputs/config/set.json`                |
  | Canonically bundled     | `v3`     | `https://aka.ms/dsc/schemas/v3/bundled/outputs/config/set.json`            |
  | Canonically bundled     | `v3.0`   | `https://aka.ms/dsc/schemas/v3.0/bundled/outputs/config/set.json`          |
  | Canonically bundled     | `v3.0.0` | `https://aka.ms/dsc/schemas/v3.0.0/bundled/outputs/config/set.json`        |
  | Enhanced AUthoring      | `v3`     | `https://aka.ms/dsc/schemas/v3/bundled/outputs/config/set.vscode.json`     |
  | Enhanced AUthoring      | `v3.0`   | `https://aka.ms/dsc/schemas/v3.0/bundled/outputs/config/set.vscode.json`   |
  | Enhanced AUthoring      | `v3.0.0` | `https://aka.ms/dsc/schemas/v3.0.0/bundled/outputs/config/set.vscode.json` |

- GitHub URIs by version and form:

  | Form                    | Version  | Recognized URI                                                                                               |
  |:------------------------|:---------|:-------------------------------------------------------------------------------------------------------------|
  | Canonical (non-bundled) | `v3`     | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/outputs/config/set.json`                    |
  | Canonical (non-bundled) | `v3.0`   | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/outputs/config/set.json`                  |
  | Canonical (non-bundled) | `v3.0.0` | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/outputs/config/set.json`                |
  | Canonically bundled     | `v3`     | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/bundled/outputs/config/set.json`            |
  | Canonically bundled     | `v3.0`   | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/bundled/outputs/config/set.json`          |
  | Canonically bundled     | `v3.0.0` | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/bundled/outputs/config/set.json`        |
  | Enhanced AUthoring      | `v3`     | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/bundled/outputs/config/set.vscode.json`     |
  | Enhanced AUthoring      | `v3.0`   | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/bundled/outputs/config/set.vscode.json`   |
  | Enhanced AUthoring      | `v3.0.0` | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/bundled/outputs/config/set.vscode.json` |

### Output schema for dsc config test command

The following table defines the value of the `$id` keyword for each published version of the output
schema for the `dsc config test` command. The `$id` is the same across all forms of the schema and
regardless of the prefix URI used to retrieve the schema.

| Version folder | ID                                                                                            |
|:---------------|:----------------------------------------------------------------------------------------------|
| `v3`           | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/outputs/config/test.json`     |
| `v3.0`         | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/outputs/config/test.json`   |
| `v3.0.0`       | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/outputs/config/test.json` |

The following list of tables defines the recognized URIs for the output schema:

- Short URIs by version and form:

  | Form                    | Version  | Recognized URI                                                            |
  |:------------------------|:---------|:--------------------------------------------------------------------------|
  | Canonical (non-bundled) | `v3`     | `https://aka.ms/dsc/schemas/v3/outputs/config/test.json`                    |
  | Canonical (non-bundled) | `v3.0`   | `https://aka.ms/dsc/schemas/v3.0/outputs/config/test.json`                  |
  | Canonical (non-bundled) | `v3.0.0` | `https://aka.ms/dsc/schemas/v3.0.0/outputs/config/test.json`                |
  | Canonically bundled     | `v3`     | `https://aka.ms/dsc/schemas/v3/bundled/outputs/config/test.json`            |
  | Canonically bundled     | `v3.0`   | `https://aka.ms/dsc/schemas/v3.0/bundled/outputs/config/test.json`          |
  | Canonically bundled     | `v3.0.0` | `https://aka.ms/dsc/schemas/v3.0.0/bundled/outputs/config/test.json`        |
  | Enhanced AUthoring      | `v3`     | `https://aka.ms/dsc/schemas/v3/bundled/outputs/config/test.vscode.json`     |
  | Enhanced AUthoring      | `v3.0`   | `https://aka.ms/dsc/schemas/v3.0/bundled/outputs/config/test.vscode.json`   |
  | Enhanced AUthoring      | `v3.0.0` | `https://aka.ms/dsc/schemas/v3.0.0/bundled/outputs/config/test.vscode.json` |

- GitHub URIs by version and form:

  | Form                    | Version  | Recognized URI                                                                                               |
  |:------------------------|:---------|:-------------------------------------------------------------------------------------------------------------|
  | Canonical (non-bundled) | `v3`     | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/outputs/config/test.json`                    |
  | Canonical (non-bundled) | `v3.0`   | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/outputs/config/test.json`                  |
  | Canonical (non-bundled) | `v3.0.0` | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/outputs/config/test.json`                |
  | Canonically bundled     | `v3`     | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/bundled/outputs/config/test.json`            |
  | Canonically bundled     | `v3.0`   | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/bundled/outputs/config/test.json`          |
  | Canonically bundled     | `v3.0.0` | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/bundled/outputs/config/test.json`        |
  | Enhanced AUthoring      | `v3`     | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/bundled/outputs/config/test.vscode.json`     |
  | Enhanced AUthoring      | `v3.0`   | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/bundled/outputs/config/test.vscode.json`   |
  | Enhanced AUthoring      | `v3.0.0` | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/bundled/outputs/config/test.vscode.json` |


### Output schema for dsc resource get command

The following table defines the value of the `$id` keyword for each published version of the output
schema for the `dsc resource get` command. The `$id` is the same across all forms of the schema and
regardless of the prefix URI used to retrieve the schema.

| Version folder | ID                                                                                            |
|:---------------|:----------------------------------------------------------------------------------------------|
| `v3`           | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/outputs/resource/get.json`     |
| `v3.0`         | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/outputs/resource/get.json`   |
| `v3.0.0`       | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/outputs/resource/get.json` |

The following list of tables defines the recognized URIs for the output schema:

- Short URIs by version and form:

  | Form                    | Version  | Recognized URI                                                            |
  |:------------------------|:---------|:--------------------------------------------------------------------------|
  | Canonical (non-bundled) | `v3`     | `https://aka.ms/dsc/schemas/v3/outputs/resource/get.json`                    |
  | Canonical (non-bundled) | `v3.0`   | `https://aka.ms/dsc/schemas/v3.0/outputs/resource/get.json`                  |
  | Canonical (non-bundled) | `v3.0.0` | `https://aka.ms/dsc/schemas/v3.0.0/outputs/resource/get.json`                |
  | Canonically bundled     | `v3`     | `https://aka.ms/dsc/schemas/v3/bundled/outputs/resource/get.json`            |
  | Canonically bundled     | `v3.0`   | `https://aka.ms/dsc/schemas/v3.0/bundled/outputs/resource/get.json`          |
  | Canonically bundled     | `v3.0.0` | `https://aka.ms/dsc/schemas/v3.0.0/bundled/outputs/resource/get.json`        |
  | Enhanced AUthoring      | `v3`     | `https://aka.ms/dsc/schemas/v3/bundled/outputs/resource/get.vscode.json`     |
  | Enhanced AUthoring      | `v3.0`   | `https://aka.ms/dsc/schemas/v3.0/bundled/outputs/resource/get.vscode.json`   |
  | Enhanced AUthoring      | `v3.0.0` | `https://aka.ms/dsc/schemas/v3.0.0/bundled/outputs/resource/get.vscode.json` |

- GitHub URIs by version and form:

  | Form                    | Version  | Recognized URI                                                                                               |
  |:------------------------|:---------|:-------------------------------------------------------------------------------------------------------------|
  | Canonical (non-bundled) | `v3`     | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/outputs/resource/get.json`                    |
  | Canonical (non-bundled) | `v3.0`   | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/outputs/resource/get.json`                  |
  | Canonical (non-bundled) | `v3.0.0` | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/outputs/resource/get.json`                |
  | Canonically bundled     | `v3`     | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/bundled/outputs/resource/get.json`            |
  | Canonically bundled     | `v3.0`   | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/bundled/outputs/resource/get.json`          |
  | Canonically bundled     | `v3.0.0` | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/bundled/outputs/resource/get.json`        |
  | Enhanced AUthoring      | `v3`     | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/bundled/outputs/resource/get.vscode.json`     |
  | Enhanced AUthoring      | `v3.0`   | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/bundled/outputs/resource/get.vscode.json`   |
  | Enhanced AUthoring      | `v3.0.0` | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/bundled/outputs/resource/get.vscode.json` |

### Output schema for dsc resource list command

The following table defines the value of the `$id` keyword for each published version of the output
schema for the `dsc resource list` command. The `$id` is the same across all forms of the schema and
regardless of the prefix URI used to retrieve the schema.

| Version folder | ID                                                                                            |
|:---------------|:----------------------------------------------------------------------------------------------|
| `v3`           | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/outputs/resource/list.json`     |
| `v3.0`         | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/outputs/resource/list.json`   |
| `v3.0.0`       | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/outputs/resource/list.json` |

The following list of tables defines the recognized URIs for the output schema:

- Short URIs by version and form:

  | Form                    | Version  | Recognized URI                                                            |
  |:------------------------|:---------|:--------------------------------------------------------------------------|
  | Canonical (non-bundled) | `v3`     | `https://aka.ms/dsc/schemas/v3/outputs/resource/list.json`                    |
  | Canonical (non-bundled) | `v3.0`   | `https://aka.ms/dsc/schemas/v3.0/outputs/resource/list.json`                  |
  | Canonical (non-bundled) | `v3.0.0` | `https://aka.ms/dsc/schemas/v3.0.0/outputs/resource/list.json`                |
  | Canonically bundled     | `v3`     | `https://aka.ms/dsc/schemas/v3/bundled/outputs/resource/list.json`            |
  | Canonically bundled     | `v3.0`   | `https://aka.ms/dsc/schemas/v3.0/bundled/outputs/resource/list.json`          |
  | Canonically bundled     | `v3.0.0` | `https://aka.ms/dsc/schemas/v3.0.0/bundled/outputs/resource/list.json`        |
  | Enhanced AUthoring      | `v3`     | `https://aka.ms/dsc/schemas/v3/bundled/outputs/resource/list.vscode.json`     |
  | Enhanced AUthoring      | `v3.0`   | `https://aka.ms/dsc/schemas/v3.0/bundled/outputs/resource/list.vscode.json`   |
  | Enhanced AUthoring      | `v3.0.0` | `https://aka.ms/dsc/schemas/v3.0.0/bundled/outputs/resource/list.vscode.json` |

- GitHub URIs by version and form:

  | Form                    | Version  | Recognized URI                                                                                               |
  |:------------------------|:---------|:-------------------------------------------------------------------------------------------------------------|
  | Canonical (non-bundled) | `v3`     | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/outputs/resource/list.json`                    |
  | Canonical (non-bundled) | `v3.0`   | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/outputs/resource/list.json`                  |
  | Canonical (non-bundled) | `v3.0.0` | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/outputs/resource/list.json`                |
  | Canonically bundled     | `v3`     | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/bundled/outputs/resource/list.json`            |
  | Canonically bundled     | `v3.0`   | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/bundled/outputs/resource/list.json`          |
  | Canonically bundled     | `v3.0.0` | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/bundled/outputs/resource/list.json`        |
  | Enhanced AUthoring      | `v3`     | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/bundled/outputs/resource/list.vscode.json`     |
  | Enhanced AUthoring      | `v3.0`   | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/bundled/outputs/resource/list.vscode.json`   |
  | Enhanced AUthoring      | `v3.0.0` | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/bundled/outputs/resource/list.vscode.json` |


### Output schema for dsc resource schema command

The following table defines the value of the `$id` keyword for each published version of the output
schema for the `dsc resource schema` command. The `$id` is the same across all forms of the schema and
regardless of the prefix URI used to retrieve the schema.

| Version folder | ID                                                                                            |
|:---------------|:----------------------------------------------------------------------------------------------|
| `v3`           | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/outputs/resource/schema.json`     |
| `v3.0`         | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/outputs/resource/schema.json`   |
| `v3.0.0`       | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/outputs/resource/schema.json` |

The following list of tables defines the recognized URIs for the output schema:

- Short URIs by version and form:

  | Form                    | Version  | Recognized URI                                                            |
  |:------------------------|:---------|:--------------------------------------------------------------------------|
  | Canonical (non-bundled) | `v3`     | `https://aka.ms/dsc/schemas/v3/outputs/resource/schema.json`                    |
  | Canonical (non-bundled) | `v3.0`   | `https://aka.ms/dsc/schemas/v3.0/outputs/resource/schema.json`                  |
  | Canonical (non-bundled) | `v3.0.0` | `https://aka.ms/dsc/schemas/v3.0.0/outputs/resource/schema.json`                |
  | Canonically bundled     | `v3`     | `https://aka.ms/dsc/schemas/v3/bundled/outputs/resource/schema.json`            |
  | Canonically bundled     | `v3.0`   | `https://aka.ms/dsc/schemas/v3.0/bundled/outputs/resource/schema.json`          |
  | Canonically bundled     | `v3.0.0` | `https://aka.ms/dsc/schemas/v3.0.0/bundled/outputs/resource/schema.json`        |
  | Enhanced AUthoring      | `v3`     | `https://aka.ms/dsc/schemas/v3/bundled/outputs/resource/schema.vscode.json`     |
  | Enhanced AUthoring      | `v3.0`   | `https://aka.ms/dsc/schemas/v3.0/bundled/outputs/resource/schema.vscode.json`   |
  | Enhanced AUthoring      | `v3.0.0` | `https://aka.ms/dsc/schemas/v3.0.0/bundled/outputs/resource/schema.vscode.json` |

- GitHub URIs by version and form:

  | Form                    | Version  | Recognized URI                                                                                               |
  |:------------------------|:---------|:-------------------------------------------------------------------------------------------------------------|
  | Canonical (non-bundled) | `v3`     | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/outputs/resource/schema.json`                    |
  | Canonical (non-bundled) | `v3.0`   | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/outputs/resource/schema.json`                  |
  | Canonical (non-bundled) | `v3.0.0` | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/outputs/resource/schema.json`                |
  | Canonically bundled     | `v3`     | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/bundled/outputs/resource/schema.json`            |
  | Canonically bundled     | `v3.0`   | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/bundled/outputs/resource/schema.json`          |
  | Canonically bundled     | `v3.0.0` | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/bundled/outputs/resource/schema.json`        |
  | Enhanced AUthoring      | `v3`     | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/bundled/outputs/resource/schema.vscode.json`     |
  | Enhanced AUthoring      | `v3.0`   | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/bundled/outputs/resource/schema.vscode.json`   |
  | Enhanced AUthoring      | `v3.0.0` | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/bundled/outputs/resource/schema.vscode.json` |


### Output schema for dsc resource set command

The following table defines the value of the `$id` keyword for each published version of the output
schema for the `dsc resource set` command. The `$id` is the same across all forms of the schema and
regardless of the prefix URI used to retrieve the schema.

| Version folder | ID                                                                                            |
|:---------------|:----------------------------------------------------------------------------------------------|
| `v3`           | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/outputs/resource/set.json`     |
| `v3.0`         | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/outputs/resource/set.json`   |
| `v3.0.0`       | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/outputs/resource/set.json` |

The following list of tables defines the recognized URIs for the output schema:

- Short URIs by version and form:

  | Form                    | Version  | Recognized URI                                                            |
  |:------------------------|:---------|:--------------------------------------------------------------------------|
  | Canonical (non-bundled) | `v3`     | `https://aka.ms/dsc/schemas/v3/outputs/resource/set.json`                    |
  | Canonical (non-bundled) | `v3.0`   | `https://aka.ms/dsc/schemas/v3.0/outputs/resource/set.json`                  |
  | Canonical (non-bundled) | `v3.0.0` | `https://aka.ms/dsc/schemas/v3.0.0/outputs/resource/set.json`                |
  | Canonically bundled     | `v3`     | `https://aka.ms/dsc/schemas/v3/bundled/outputs/resource/set.json`            |
  | Canonically bundled     | `v3.0`   | `https://aka.ms/dsc/schemas/v3.0/bundled/outputs/resource/set.json`          |
  | Canonically bundled     | `v3.0.0` | `https://aka.ms/dsc/schemas/v3.0.0/bundled/outputs/resource/set.json`        |
  | Enhanced AUthoring      | `v3`     | `https://aka.ms/dsc/schemas/v3/bundled/outputs/resource/set.vscode.json`     |
  | Enhanced AUthoring      | `v3.0`   | `https://aka.ms/dsc/schemas/v3.0/bundled/outputs/resource/set.vscode.json`   |
  | Enhanced AUthoring      | `v3.0.0` | `https://aka.ms/dsc/schemas/v3.0.0/bundled/outputs/resource/set.vscode.json` |

- GitHub URIs by version and form:

  | Form                    | Version  | Recognized URI                                                                                               |
  |:------------------------|:---------|:-------------------------------------------------------------------------------------------------------------|
  | Canonical (non-bundled) | `v3`     | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/outputs/resource/set.json`                    |
  | Canonical (non-bundled) | `v3.0`   | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/outputs/resource/set.json`                  |
  | Canonical (non-bundled) | `v3.0.0` | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/outputs/resource/set.json`                |
  | Canonically bundled     | `v3`     | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/bundled/outputs/resource/set.json`            |
  | Canonically bundled     | `v3.0`   | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/bundled/outputs/resource/set.json`          |
  | Canonically bundled     | `v3.0.0` | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/bundled/outputs/resource/set.json`        |
  | Enhanced AUthoring      | `v3`     | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/bundled/outputs/resource/set.vscode.json`     |
  | Enhanced AUthoring      | `v3.0`   | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/bundled/outputs/resource/set.vscode.json`   |
  | Enhanced AUthoring      | `v3.0.0` | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/bundled/outputs/resource/set.vscode.json` |

### Output schema for dsc resource test command

The following table defines the value of the `$id` keyword for each published version of the output
schema for the `dsc resource test` command. The `$id` is the same across all forms of the schema and
regardless of the prefix URI used to retrieve the schema.

| Version folder | ID                                                                                            |
|:---------------|:----------------------------------------------------------------------------------------------|
| `v3`           | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/outputs/resource/test.json`     |
| `v3.0`         | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/outputs/resource/test.json`   |
| `v3.0.0`       | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/outputs/resource/test.json` |

The following list of tables defines the recognized URIs for the output schema:

- Short URIs by version and form:

  | Form                    | Version  | Recognized URI                                                            |
  |:------------------------|:---------|:--------------------------------------------------------------------------|
  | Canonical (non-bundled) | `v3`     | `https://aka.ms/dsc/schemas/v3/outputs/resource/test.json`                    |
  | Canonical (non-bundled) | `v3.0`   | `https://aka.ms/dsc/schemas/v3.0/outputs/resource/test.json`                  |
  | Canonical (non-bundled) | `v3.0.0` | `https://aka.ms/dsc/schemas/v3.0.0/outputs/resource/test.json`                |
  | Canonically bundled     | `v3`     | `https://aka.ms/dsc/schemas/v3/bundled/outputs/resource/test.json`            |
  | Canonically bundled     | `v3.0`   | `https://aka.ms/dsc/schemas/v3.0/bundled/outputs/resource/test.json`          |
  | Canonically bundled     | `v3.0.0` | `https://aka.ms/dsc/schemas/v3.0.0/bundled/outputs/resource/test.json`        |
  | Enhanced AUthoring      | `v3`     | `https://aka.ms/dsc/schemas/v3/bundled/outputs/resource/test.vscode.json`     |
  | Enhanced AUthoring      | `v3.0`   | `https://aka.ms/dsc/schemas/v3.0/bundled/outputs/resource/test.vscode.json`   |
  | Enhanced AUthoring      | `v3.0.0` | `https://aka.ms/dsc/schemas/v3.0.0/bundled/outputs/resource/test.vscode.json` |

- GitHub URIs by version and form:

  | Form                    | Version  | Recognized URI                                                                                               |
  |:------------------------|:---------|:-------------------------------------------------------------------------------------------------------------|
  | Canonical (non-bundled) | `v3`     | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/outputs/resource/test.json`                    |
  | Canonical (non-bundled) | `v3.0`   | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/outputs/resource/test.json`                  |
  | Canonical (non-bundled) | `v3.0.0` | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/outputs/resource/test.json`                |
  | Canonically bundled     | `v3`     | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/bundled/outputs/resource/test.json`            |
  | Canonically bundled     | `v3.0`   | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/bundled/outputs/resource/test.json`          |
  | Canonically bundled     | `v3.0.0` | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/bundled/outputs/resource/test.json`        |
  | Enhanced AUthoring      | `v3`     | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/bundled/outputs/resource/test.vscode.json`     |
  | Enhanced AUthoring      | `v3.0`   | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/bundled/outputs/resource/test.vscode.json`   |
  | Enhanced AUthoring      | `v3.0.0` | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/bundled/outputs/resource/test.vscode.json` |

### Output schema for dsc schema command

The following table defines the value of the `$id` keyword for each published version of the output
schema for the `dsc schema` command. The `$id` is the same across all forms of the schema and
regardless of the prefix URI used to retrieve the schema.

| Version folder | ID                                                                                            |
|:---------------|:----------------------------------------------------------------------------------------------|
| `v3`           | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/outputs/schema.json`     |
| `v3.0`         | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/outputs/schema.json`   |
| `v3.0.0`       | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/outputs/schema.json` |

The following list of tables defines the recognized URIs for the output schema:

- Short URIs by version and form:

  | Form                    | Version  | Recognized URI                                                            |
  |:------------------------|:---------|:--------------------------------------------------------------------------|
  | Canonical (non-bundled) | `v3`     | `https://aka.ms/dsc/schemas/v3/outputs/schema.json`                    |
  | Canonical (non-bundled) | `v3.0`   | `https://aka.ms/dsc/schemas/v3.0/outputs/schema.json`                  |
  | Canonical (non-bundled) | `v3.0.0` | `https://aka.ms/dsc/schemas/v3.0.0/outputs/schema.json`                |
  | Canonically bundled     | `v3`     | `https://aka.ms/dsc/schemas/v3/bundled/outputs/schema.json`            |
  | Canonically bundled     | `v3.0`   | `https://aka.ms/dsc/schemas/v3.0/bundled/outputs/schema.json`          |
  | Canonically bundled     | `v3.0.0` | `https://aka.ms/dsc/schemas/v3.0.0/bundled/outputs/schema.json`        |
  | Enhanced AUthoring      | `v3`     | `https://aka.ms/dsc/schemas/v3/bundled/outputs/schema.vscode.json`     |
  | Enhanced AUthoring      | `v3.0`   | `https://aka.ms/dsc/schemas/v3.0/bundled/outputs/schema.vscode.json`   |
  | Enhanced AUthoring      | `v3.0.0` | `https://aka.ms/dsc/schemas/v3.0.0/bundled/outputs/schema.vscode.json` |

- GitHub URIs by version and form:

  | Form                    | Version  | Recognized URI                                                                                               |
  |:------------------------|:---------|:-------------------------------------------------------------------------------------------------------------|
  | Canonical (non-bundled) | `v3`     | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/outputs/schema.json`                    |
  | Canonical (non-bundled) | `v3.0`   | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/outputs/schema.json`                  |
  | Canonical (non-bundled) | `v3.0.0` | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/outputs/schema.json`                |
  | Canonically bundled     | `v3`     | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/bundled/outputs/schema.json`            |
  | Canonically bundled     | `v3.0`   | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/bundled/outputs/schema.json`          |
  | Canonically bundled     | `v3.0.0` | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/bundled/outputs/schema.json`        |
  | Enhanced AUthoring      | `v3`     | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3/bundled/outputs/schema.vscode.json`     |
  | Enhanced AUthoring      | `v3.0`   | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0/bundled/outputs/schema.vscode.json`   |
  | Enhanced AUthoring      | `v3.0.0` | `https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.0.0/bundled/outputs/schema.vscode.json` |
