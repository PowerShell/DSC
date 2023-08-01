# Structured message schema reference

## Synopsis

A message emitted by a DSC Resource with associated metadata.

## Metadata

```yaml
Schema Dialect : https://json-schema.org/draft/2020-12/schema
Schema ID      : https://schemas.microsoft.com/dsc/2023/07/definitions/message.yaml
Type           : object
```

## Description

## Required properties

Every message must be an object that defines these properties:

- [name](#name)
- [type](#type)
- [message](#message)
- [level](#level)

## Properties

### name

Identifies the instance by its short, unique, human-readable name as defined in the configuration
document.

```yaml
Type:     string
Required: true
```

### type

Identifies the instance's DSC Resource by its fully qualified type name. For more information about
type names, see [DSC Resource fully qualified type name schema reference][01].

```yaml
Type:     string
Required: true
Pattern:  ^\w+(\.\w+){0,2}\/\w+$
```

### message

The actual content of the message as emitted by the DSC Resource.

```yaml
Type:     string
Required: true
```

### level

Indicates the severity of the message.

```yaml
Type:     string
Required: true
Valid Values:
  - Error
  - Warning
  - Information
```

[01]: resourceType.md
