---
description: Reference for the 'secret' DSC configuration document function
ms.date:     08/22/2025
ms.topic:    reference
title:       secret
---

# secret

## Synopsis

Retrieves secrets from registered secret management extensions.

## Syntax

```Syntax
secret(<name>)
secret(<name>, <vault>)
```

## Description

The `secret()` function retrieves secrets from extensions that support the
secret capability. It queries all registered extensions that implement secret
management and returns the requested secret value. If multiple extensions
return different values for the same secret name, an error is returned unless
a vault is specified to disambiguate.

The function supports two calling patterns:

- Single argument: Retrieves a secret by name from any available vault
- Two arguments: Retrieves a secret by name from a specific vault

If multiple extensions return the same secret value, the function succeeds and
returns that value. This allows for redundancy across secret management
systems.

## Examples

### Example 1 - Retrieve a secret by name

The following example retrieves a secret named 'DatabasePassword' from any
available vault.

```yaml
# secret.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Database Connection
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "Connection string with password: [secret('DatabasePassword')]"
```

```bash
dsc config get --file secret.example.1.dsc.config.yaml
```

```yaml
results:
- name: Database Connection
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: "Connection string with password: MySecretPassword123"
messages: []
hadErrors: false
```

### Example 2 - Retrieve a secret from a specific vault

The following example retrieves a secret from a specific vault to avoid
ambiguity when the same secret name exists in multiple vaults.

```yaml
# secret.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: API Configuration
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "API Key: [secret('ApiKey', 'ProductionVault')]"
```

```bash
dsc config get --file secret.example.2.dsc.config.yaml
```

```yaml
results:
- name: API Configuration
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: "API Key: prod-api-key-xyz789"
messages: []
hadErrors: false
```

### Example 3 - Multiple secrets in configuration

The following example demonstrates using multiple secrets within a single
resource configuration.

```yaml
# secret.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Service Configuration
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      username: "[secret('ServiceUsername')]"
      password: "[secret('ServicePassword')]"
      connectionString: "Server=myserver;User=[secret('DbUser')];Password=[secret('DbPassword')]"
```

```bash
dsc config get --file secret.example.3.dsc.config.yaml
```

```yaml
results:
- name: Service Configuration
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        username: "serviceaccount"
        password: "SecurePassword456"
        connectionString: "Server=myserver;User=dbuser;Password=DbPass789"
messages: []
hadErrors: false
```

### Example 4 - Conditional secret usage

The following example shows using secrets conditionally based on environment
parameters.

```yaml
# secret.example.4.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  environment:
    type: string
    defaultValue: production
resources:
- name: Environment-specific config
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      apiKey: >-
        [if(equals(parameters('environment'), 'production'),
        secret('ProdApiKey', 'ProdVault'),
        secret('DevApiKey', 'DevVault'))]
```

```bash
dsc config get --file secret.example.4.dsc.config.yaml
```

```yaml
results:
- name: Environment-specific config
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        apiKey: "prod-secure-key-abc123"
messages: []
hadErrors: false
```

## Parameters

### name

The name of the secret to retrieve.

```yaml
Type:     string
Required: true
Position: 1
```

### vault

The name of the vault or secret store to retrieve the secret from. When
specified, only the named vault is queried for the secret, which helps
disambiguate when multiple vaults contain secrets with the same name.

```yaml
Type:     string
Required: false
Position: 2
```

## Output

The `secret()` function returns the secret value as a string.

```yaml
Type: string
```

## Error conditions

The `secret()` function can return errors in the following situations:

- **No extensions available**: No secret management extensions are registered
  or available
- **Secret not found**: The specified secret name does not exist in any
  available vault
- **Multiple different values**: Multiple extensions return different values
  for the same secret name (specify a vault to disambiguate)
- **Vault not found**: The specified vault does not exist or is not accessible
- **Extension error**: An underlying secret management extension returns an
  error

## Security considerations

- Secret values are retrieved at runtime and should be handled securely
- Secrets are not cached by DSC and are retrieved fresh on each function call
- Secret values are logged in trace output
- Extensions should implement appropriate authentication and authorization for
  secret access

## Extension requirements

To support the `secret()` function, extensions must:

1. Declare the `secret` capability in their manifest
2. Implement a secret retrieval method that accepts name and optional vault
   parameters
3. Return secret values as single-line strings (multi-line values are not
   supported)
4. Handle authentication and authorization according to their secret
   management system

## Related functions

- [`if()`][00] - Returns values based on a condition for conditional secret
  usage
- [`equals()`][01] - Compares values for conditional logic
- [`parameters()`][02] - Access configuration parameters that may influence
  secret selection

<!-- Link reference definitions -->
[00]: ./if.md
[01]: ./equals.md
[02]: ./parameters.md
