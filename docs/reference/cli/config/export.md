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

### Configuration document from stdin

```sh
<document-string> | dsc config export [Options]
```

### Configuration document from option string

```sh
dsc config export [Options] --document <document-string>
```

### Configuration document from file

```sh
dsc config export [Options] --path <document-filepath>
```

## Description

The `export` subcommand generates a configuration document that includes every instance of a set of
resources.

The configuration document must be passed to this command as JSON or YAML over stdin, as a string
with the **document** option, or from a file with the **path** option.

The input configuration defines the resources to export. DSC ignores any properties specified for
the resources in the input configuration for the operation, but the input document and any
properties for resource instances must still validate against the configuration document and
resource instance schemas.

Only specify resources with a resource manifest that defines the [export][01] section in the input
configuration. Only define each resource type once. If the configuration document includes any
resource instance where the resource type isn't exportable or has already been declared in the
configuration, DSC raises an error.

## Options

### -d, --document

Specifies the configuration document to export from as a JSON or YAML object. DSC validates the
document against the configuration document schema. If the validation fails, DSC raises an error.

This option can't be used with configuration document over stdin or the `--path` option. Choose
whether to pass the configuration document to the command over stdin, from a file with the `--path`
option, or with the `--document` option.

```yaml
Type:      String
Mandatory: false
```

### -p, --path

Defines the path to a configuration document to export instead of piping the document from stdin or
passing it as a string with the `--document` option. The specified file must contain a
configuration document as a JSON or YAML object. DSC validates the document against the
configuration document schema. If the validation fails, or if the specified file doesn't exist, DSC
raises an error.

This option is mutually exclusive with the `--document` option. When you use this option, DSC
ignores any input from stdin.

```yaml
Type:      String
Mandatory: false
```

### -f, --format

The `--format` option controls the console output format for the command. If the command output is
redirected or captured as a variable, the output is always JSON.

```yaml
Type:         String
Mandatory:    false
DefaultValue: yaml
ValidValues:  [json, pretty-json, yaml]
```

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
