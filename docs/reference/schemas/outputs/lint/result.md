---
description: JSON schema reference for the data returned by the 'dsclint' command and the 'lint_dsc_config' MCP tool.
ms.date:     10/23/2026
ms.topic:    reference
title:       dsclint result schema reference
---

# dsclint result schema reference

## Synopsis

The result output from the `dsclint` command and the `lint_dsc_config` tool of the DSC MCP server.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/v3.1.0/outputs/lint/result.json
Type:          object
```

## Description

The output from the `dsclint` command includes every diagnostic the rules reported, counts of the
diagnostics by level, whether the document is free of error-level diagnostics, and whether
resource discovery ran. The command always emits this object, including when the document doesn't
parse. The `dsc schema --type lint-result` command returns this schema.

## Required properties

The output always includes these properties:

- [diagnostics](#diagnostics)
- [summary](#summary)
- [valid](#valid)
- [discovery](#discovery)

## Properties

### diagnostics

The problems the rules reported, ordered by file, path, and rule. Each entry is an object with the
following properties:

- `level` is the severity of the problem: `error`, `warning`, or `info`.
- `rule` is the identifier of the rule that reported the problem, such as `duplicate-resource`.
- `category` is the area of the document the rule inspects: `document`, `expression`, `parameter`,
  `dependency`, `deprecation`, `environment`, or `resource`.
- `message` describes the problem.
- `path` is the JSON pointer ([RFC 6901][01]) to the location of the problem in the document. An
  empty string identifies the document root.
- `file` is the file the document was read from. The property is omitted for documents passed
  with `--input` or from stdin.
- `location` is the position of the problem in the source text as an object with `line` and
  `column` properties, both starting at `1`. For a `path` the document doesn't spell out, like a
  property that is missing, the location is that of the closest enclosing value. The property is
  omitted when the linter is given an already parsed document, as the MCP server does.

```yaml
Type:     array
Required: true
```

### summary

Counts of the diagnostics by level, as an object with the `errors`, `warnings`, and `info`
properties.

```yaml
Type:     object
Required: true
```

### valid

Whether no error-level diagnostics were reported. This property doesn't depend on the `--fail-on`
option.

```yaml
Type:     boolean
Required: true
```

### discovery

Whether resource discovery ran. When `false`, the rules that inspect installed resources were
skipped, either because of the `--no-discovery` option, because the document sets
`directives.resourceDiscovery` to `duringDeployment`, or because discovery failed.

```yaml
Type:     boolean
Required: true
```

<!-- Link reference definitions -->
[01]: https://www.rfc-editor.org/rfc/rfc6901
