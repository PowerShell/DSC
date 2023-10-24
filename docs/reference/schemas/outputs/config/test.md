---
description: JSON schema reference for the data returned by the 'dsc config test' command.
ms.date:     08/04/2023
ms.topic:    reference
title:       dsc config test result schema reference
---

# dsc config test result schema reference

## Synopsis

The result output from the `dsc config test` command.

## Metadata

```yaml
SchemaDialect: https://json-schema.org/draft/2020-12/schema
SchemaID:      https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/10/outputs/config/test.json
Type:          object
```

## Description

The output from the `dsc config test` command includes the state of every resource instance in the
configuration before and after the test operation, and the list of properties the operation changed
for each instance.

## Required properties

The output always includes these properties:

- [results](#results)
- [messages](#messages)
- [hadErrors](#haderrors)

## Properties

### results

Defines the list of results for the `test` operation invoked against every instance in the
configuration document. Every entry in the list includes the resource's type name, instance name,
and the result data for an instance.

```yaml
Type:      array
Required:  true
ItemsType: object
```

#### type

An item's `type` property identifies the instance's DSC Resource by its fully qualified type name.
For more information about type names, see
[DSC Resource fully qualified type name schema reference][01].

```yaml
Type:     string
Required: true
Pattern:  ^\w+(\.\w+){0,2}\/\w+$
```

#### name

An item's `name` property identifies the instance by its short, unique, human-readable name.

```yaml
Type:     string
Required: true
```

#### result

An item's `result` property includes the actual state for the resource instance. The value for this
property adheres to the same schema as the output for the `dsc resource test` command. For more
information, see [dsc resource test result schema reference][02].

### messages

Defines the list of structured messages emitted by resources during the test operation. For more
information, see [Structured message schema reference][03].

```yaml
Type:     array
Required: true
```

### hadErrors

Indicates whether the operation encountered any errors. This value is `true` if the configuration
document failed validation or any resource exited with an exit code other than `0`.

```yaml
Type:     boolean
Required: true
```

[01]: ../../definitions/resourceType.md
[02]: ../resource/test.md
[03]: ../../definitions/message.md
