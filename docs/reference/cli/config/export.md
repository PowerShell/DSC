---
description: Command line reference for the 'dsc config export' command
ms.date:     09/06/2023
ms.topic:    reference
title:       dsc config export
---

# dsc config export

## Synopsis

Generates a configuration document that defines the existing instances of a set of resources.

## Syntax

```sh
dsc config export [Options]
```

## Description

The `export` subcommand generates a configuration document that includes every instance of a set of
resources. This command expects a configuration document formatted as JSON or YAML over stdin. The
input configuration defines the resources to export. DSC ignores any properties specified for the
resources in the input configuration for the operation, but the input document and any properties
for resource instances must still validate against the configuration document and resource instance
schemas.

Only specify resources with a resource manifest that defines the [export][01] section in the input
configuration. Only define each resource type once. If the configuration document includes any
resource instance where the resource type isn't exportable or has already been declared in the
configuration, DSC raises an error.

## Options

### -h, --help

Displays the help for the current command or subcommand. When you specify this option, the
application ignores all options and arguments after this one.

```yaml
Type:      Boolean
Mandatory: false
```

## Output

This command returns JSON output that defines a configuration document including every instance of
the resources declared in the input configuration. For more information, see
[DSC Configuration document schema reference][02].

[01]: ../../schemas/resource/manifest/export.md
[02]: ../../schemas/config/document.md
