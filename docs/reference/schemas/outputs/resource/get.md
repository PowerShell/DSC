# dsc resource get result schema reference

## Synopsis

The result output from the `dsc resource get` command.

## Metadata

```yaml
Schema Dialect : https://json-schema.org/draft/2020-12/schema
Schema ID      : https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/08/outputs/resource/get.json
Type           : object
```

## Description

The output from the `dsc resource get` command includes the actual state for the specified resource
instance.

## Required properties

The output always includes these properties:

- [actualState](#actualstate)

## Properties

### actualState

The `actualState` property always includes the state of the instance returned when DSC invokes the
resource's get operation. DSC validates this property's value against the resource's instance
schema.

```yaml
Type:     object
Required: true
```
