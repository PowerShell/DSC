---
description: JSON schema reference for the data returned by the 'dsc lint --list-rules' command.
ms.date:     10/20/2026
ms.topic:    reference
title:       dsc lint rules schema reference
---

# dsc lint rules schema reference

## Synopsis

The output from the `dsc lint --list-rules` command.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/outputs/lint/rules.json
Type:          object
```

## Description

The `dsc lint --list-rules` command returns an array of objects with this schema, one per rule,
sorted by rule identifier. Each object describes the rule and the level it reports at after the
settings file and any `--rule` overrides are applied.

## Required properties

Each entry always includes these properties:

- [id](#id)
- [category](#category)
- [tier](#tier)
- [defaultLevel](#defaultlevel)
- [level](#level)
- [fixed](#fixed)
- [description](#description)

## Properties

### id

The identifier of the rule, such as `duplicate-resource`.

```yaml
Type:     string
Required: true
```

### category

The area of the document the rule inspects: `document`, `expression`, `parameter`, `dependency`,
`deprecation`, `environment`, or `resource`.

```yaml
Type:     string
Required: true
```

### tier

When the rule can run: `static` rules only need the document, `resource` rules need the installed
resources and only run when discovery is enabled.

```yaml
Type:        string
Required:    true
ValidValues: [static, resource]
```

### defaultLevel

The level the rule reports at unless it is overridden: `off`, `info`, `warning`, or `error`.

```yaml
Type:        string
Required:    true
ValidValues: [off, info, warning, error]
```

### level

The level the rule reports at in this run, after the settings file and any `--rule` overrides are
applied.

```yaml
Type:        string
Required:    true
ValidValues: [off, info, warning, error]
```

### fixed

Whether the level of the rule can't be changed.

```yaml
Type:     boolean
Required: true
```

### description

What the rule checks.

```yaml
Type:     string
Required: true
```
