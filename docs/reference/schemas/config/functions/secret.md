---
description: Reference for the 'secret' DSC configuration document function
ms.date:     06/04/2025
ms.topic:    reference
title:       secret
---

# secret

## Synopsis

Retrieves a secret from a vault.

## Syntax

```Syntax
secret(<secretName>, [vaultName])
```

## Description

The `secret()` function searches secret extensions for a secret with the provided name. You must
pass a name of a valid secret that exists in at least one extension. 

If more than one secret exists with the name and a different value, an error will be raised. If
all the duplicate secrets share a value, it will return that value.

## Examples

### Example 1 - Echo a secret's value

The configuration uses the `secret()` function to query from secret extensions and echo the value
of the secret.

```yaml
# secret.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: Echo secret value
    type: Microsoft.DSC.Debug/Echo
    properties:
      output: "[secret('mySecretName')]"
```

```bash
dsc config get --file secret.example.1.dsc.config.yaml
```

```yaml
results:
- name: Echo secret value
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: Password123!
messages: []
hadErrors: false
```

## Parameters

### secretName

The name of the secret to retrieve.

```yaml
Type:     string
Required: true
Position: 1
```

### vaultName

The name of the vault to retrieve the secret from. The implementation of the is dependent on the
secret extension and may not be required.

```yaml
Type:     string
Required: false
Position: 2
```

## Output

The `secret()` function returns the value of the secret as a string.

```yaml
Type: string
```

## Errors

The function returns an error in the following cases:

- **Invalid type**: Any argument is not a string
- **No extensions**: There are no secret extensions available
- **Multiple secrets**: Multiple secrets with the same name but different values were returned
- **Extension returned error**: A secret extension returned an error during the query
- **Secret not found**: No secret was found 

<!-- Link reference definitions -->
[01]: ../../../schemas/extension/manifest/discover.md