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

The `secret()` function retrieves secrets from extensions that support the secret capability. It
queries all registered extensions that implement secret management and returns the requested secret
value. If multiple extensions return different values for the same secret name, an error is
returned unless a vault is specified to disambiguate.

The function supports two calling patterns:

- Single argument: Retrieves a secret by name from any available vault
- Two arguments: Retrieves a secret by name from a specific vault

If multiple extensions return the same secret value, the function succeeds and returns that value.
This allows for redundancy across secret management systems.

## Examples

### Example 1 - Retrieve a secret by name

The following example retrieves a secret named `DatabasePassword` from any available vault. The
secret expression is used directly as the output value.

```yaml
# secret.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Database Connection
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[secret('DatabasePassword')]"
```

```bash
dsc config get --file secret.example.1.dsc.config.yaml
```

```yaml
executionInformation:
  duration: PT11.2326172S
  endDatetime: 2026-05-05T09:55:33.313388600-05:00
  executionType: actual
  operation: get
  securityContext: restricted
  startDatetime: 2026-05-05T09:55:22.080771400-05:00
  version: 3.2.0
metadata:
  Microsoft.DSC:
    duration: PT11.2325964S
    endDatetime: 2026-05-05T09:55:33.313367800-05:00
    executionType: actual
    operation: get
    securityContext: restricted
    startDatetime: 2026-05-05T09:55:22.080771400-05:00
    version: 3.2.0
results:
- executionInformation:
    duration: PT2.0098514S
  metadata:
    Microsoft.DSC:
      duration: PT2.0098514S
  name: Database Connection
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: "MySecretPassword123"
messages: []
hadErrors: false
```

> [!NOTE]
> In this example the secret is emitted in plain text in the output. This is because the `secret`
> function doesn't automatically wrap the result in a `secureString` wrapper. It passes the
> discovered secret directly to the resource as a string.
>
> DSC doesn't emit secrets to the console or through trace messages itself. However, DSC can't
> control whether a resource emits secrets. In this case, the secret was emitted by the `Echo`
> resource in its output. For more information about handling secrets in a configuration, see
> [Security considerations](#security-considerations).

### Example 2 - Pass a secret through a parameter default

The following example defines a `secureString` parameter whose default value is the secret. The
resource then uses the parameter value for the output property.

```yaml
# secret.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  myString:
    type: secureString
    defaultValue: "[secret('MySecret')]"
resources:
- name: Database Connection
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[parameters('myString')]"
```

```bash
dsc config get --file secret.example.2.dsc.config.yaml
```

```yaml
executionInformation:
  duration: PT13.5097206S
  endDatetime: 2026-05-05T10:26:29.721210400-05:00
  executionType: actual
  operation: get
  securityContext: restricted
  startDatetime: 2026-05-05T10:26:16.211489800-05:00
  version: 3.2.0
metadata:
  Microsoft.DSC:
    duration: PT13.5097028S
    endDatetime: 2026-05-05T10:26:29.721192600-05:00
    executionType: actual
    operation: get
    securityContext: restricted
    startDatetime: 2026-05-05T10:26:16.211489800-05:00
    version: 3.2.0
results:
- executionInformation:
    duration: PT1.451969S
  metadata:
    Microsoft.DSC:
      duration: PT1.451969S
  name: Database Connection
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output: <secureValue>
messages: []
hadErrors: false
```

> [!NOTE]
> In this case, the value for the secret has been redacted because it was passed to the `Echo`
> resource as a `secureString` value.

## Parameters

### name

The name of the secret to retrieve.

DSC passes this value to every extension with the `secret` capability as-is. Whether this value is
case-sensitive depends on the extension and the secret vault that it uses.

```yaml
Type:     string
Required: true
Position: 1
```

### vault

The name of the vault or secret store to retrieve the secret from. When specified, only the named
vault is queried for the secret, which helps disambiguate when multiple vaults contain secrets with
the same name.

DSC passes this value to every extension with the `secret` capability as-is. Whether this value is
case-sensitive depends on the extension and the secret vault that it uses.

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

- DSC retrieves secret values at runtime and doesn't cache them.
- DSC invokes the `secret` operation for every available extension with that capability on each
  usage of the `secret()` function in a configuration document.
- When you invoke DSC with `--trace-level` as `TRACE`, unwrapped secret values are emitted in trace
  messages as part of the JSON Validation trace message for resource input.

  When the secret values are wrapped as a `secureObject` or `secureString`, DSC redacts the value
  in trace messaging where it appears instead as `<secureValue>`.
- DSC doesn't automatically wrap retrieved secrets as `secureString` instances. When a secret is
  wrapped as a `secureString`, like `{"secureString":"<secret>"}`, the resource is responsible for
  unwrapping the data. Check the JSON Schema and documentation for the resources you use to see
  whether they support `secureString` values.

## Related functions

- [`parameters()`][00] - Access configuration parameters that may influence
  secret selection

<!-- Link reference definitions -->
[00]: ./parameters.md
