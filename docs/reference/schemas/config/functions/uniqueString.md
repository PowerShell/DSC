---
description: Reference for the 'uniqueString' DSC configuration document function
ms.date:     08/12/2025
ms.topic:    reference
title:       uniqueString
---

# uniqueString

## Synopsis

Creates a deterministic lowercase Base32 string from one or more input strings.

## Syntax

```Syntax
uniqueString(<value1>[, <value2>, ...])
```

## Description

The `uniqueString()` function produces a stable hash-based string from one or
more input strings. The inputs are concatenated with dash (`-`) separators then
hashed using MurmurHash64 and Base32-encoded (RFC 4648 lowercase, no padding).
The same inputs always produce the same output, making it useful for generating
repeatable names that satisfy length and character constraints.

Because the output is a non-cryptographic hash, there's no direct reverse
operation, but it doesn't provide secrecy. If the possible inputs are
predictable (such as a small set of env, region, or service names), an attacker
can brute-force or dictionary-guess the original values by recomputing hashes.
Don't use `uniqueString()` for secrets or security decisions. Use it only for
stable, deterministic naming.

## Examples

### Example 1 - Generate a name from components

The following example generates a unique, deterministic identifier from a set
of input properties.

```yaml
# uniquestring.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  env:
    type: string
    defaultValue: prod
  service:
    type: string
    defaultValue: billing
  region:
    type: string
    defaultValue: westus
resources:
- name: Deterministic name
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      name: "[uniqueString(parameters('env'), parameters('service'), parameters('region'))]"
```

```bash
dsc config get --file uniquestring.example.1.dsc.config.yaml
```

```yaml
results:
- name: Deterministic name
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        name: f5saooq7aoueg
messages: []
hadErrors: false
```

### Example 2 - Stable bucket partition keys

The following example shows using `uniqueString()` to build a stable partition
key for storage or grouping.

```yaml
# uniquestring.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  tenantId:
    type: string
    defaultValue: tenantA
  dataType:
    type: string
    defaultValue: metrics
resources:
- name: Partition key generation
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      partitionKey: "[uniqueString(parameters('tenantId'), parameters('dataType'))]"
```

```bash
dsc config get --file uniquestring.example.2.dsc.config.yaml
```

```yaml
results:
- name: Partition key generation
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        partitionKey: i5w466eimgg52
messages: []
hadErrors: false
```

### Example 3 - Compose with other functions

The following example builds a prefixed resource identifier combining a literal
prefix and the hash output.

```yaml
# uniquestring.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  project:
    type: string
    defaultValue: analytics
  zone:
    type: string
    defaultValue: eu-central
resources:
- name: Prefixed identifier
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      resourceId: "[concat('res-', uniqueString(parameters('project'), parameters('zone')))]"
```

```bash
dsc config get --file uniquestring.example.3.dsc.config.yaml
```

```yaml
results:
- name: Prefixed identifier
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        resourceId: res-ce56slj5eites
messages: []
hadErrors: false
```

## Parameters

### value1

The first string value to include in the hash input sequence.

```yaml
Type:     string
Required: true
Position: 1
```

### value2, ... (additional values)

Optional additional string values to include. Each is appended with a dash
separator before hashing. Argument order affects the result.

```yaml
Type:     string
Required: false
Position: 2+
```

## Output

The `uniqueString()` function returns a deterministic lowercase Base32 string.

```yaml
Type: string
```

## Related functions

- [`concat()`][00] - Concatenates strings together
- [`string()`][01] - Converts values to strings
- [`utcNow()`][02] - Returns the current UTC timestamp

<!-- Link reference definitions -->
[00]: ./concat.md
[01]: ./string.md
[02]: ./utcNow.md
