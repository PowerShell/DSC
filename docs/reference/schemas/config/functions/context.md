---
description: Reference for the 'context' DSC configuration document function
ms.date:     02/20/2026
ms.topic:    reference
title:       context
---

# context

## Synopsis

Returns contextual information about the system and execution environment.

## Syntax

```Syntax
context()
```

## Description

The `context()` function returns an object with information about the operating system and the
security context that DSC is running under. This is useful when you want to configure resources
differently depending on the host platform, OS version, bitness, or whether DSC is running with
elevated privileges.

The returned object has two top-level properties:

- **os** - An object describing the operating system.
- **security** - A string indicating the security context.

You can access individual properties from the returned object using dot-path notation, such as
`context().os.family` or `context().security`.

### os properties

| Property       | Type    | Always present | Description                                                      |
|----------------|---------|:--------------:|------------------------------------------------------------------|
| `family`       | string  |      Yes       | The OS family: `Linux`, `macOS`, or `Windows`.                   |
| `version`      | string  |      Yes       | The OS version string.                                           |
| `edition`      | string  |       No       | The Windows edition, e.g. `Windows 11 Enterprise`. Windows only. |
| `codename`     | string  |       No       | The Linux distribution codename from `lsb_release`. Linux only.  |
| `bitness`      | integer |       No       | The OS bitness: `32` or `64`.                                    |
| `architecture` | string  |       No       | The processor architecture, e.g. `x86_64` or `arm64`.            |

### security values

| Value        | Description                                                                   |
|--------------|-------------------------------------------------------------------------------|
| `Elevated`   | DSC is running with elevated (administrator or root) privileges.              |
| `Restricted` | DSC is running with restricted (standard user) privileges.                    |
| `Current`    | The security context is forwarded from the calling process without elevation. |

## Examples

### Example 1 - Echo the full context object

This example shows how to echo the entire `context()` output to inspect all available fields
on the machine where DSC is running.

```yaml
# context.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Show context
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[context()]"
```

```bash
dsc config get --file context.example.1.dsc.config.yaml
```

```yaml
results:
- name: Show context
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        os:
          family: Windows
          version: 10.0.26100
          edition: Windows 11 Enterprise
          bitness: 64
          architecture: x86_64
        security: Elevated
messages: []
hadErrors: false
```

### Example 2 - Access individual OS properties

This example uses dot-path notation to access specific fields from the context object. It
echoes the OS family and architecture separately, which is useful when you need only a subset
of the context data.

```yaml
# context.example.2.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: OS details
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      family:       "[context().os.family]"
      version:      "[context().os.version]"
      architecture: "[context().os.architecture]"
      security:     "[context().security]"
```

```bash
dsc config get --file context.example.2.dsc.config.yaml
```

```yaml
results:
- name: OS details
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        family: Linux
        version: 22.04
        architecture: x86_64
        security: Restricted
messages: []
hadErrors: false
```

### Example 3 - Use context to tailor resource input

This example passes OS context into a resource property so that the downstream configuration
logic can adapt to the current platform. It combines `context()` with `createObject()` to
build a structured payload, and uses `base64()` to encode it for safe transport.

```yaml
# context.example.3.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Platform-aware payload
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      platform: "[context().os.family]"
      isElevated: "[equals(context().security, 'Elevated')]"
      encodedContext: "[base64(string(context()))]"
```

```bash
dsc config get --file context.example.3.dsc.config.yaml
```

```yaml
results:
- name: Platform-aware payload
  type: Microsoft.DSC.Debug/Echo
  result:
    actualState:
      output:
        platform: macOS
        isElevated: false
        encodedContext: eyJvcyI6eyJmYW1pbHkiOiJtYWNPUyIsInZlcnNpb24iOiIxNS4zLjEiLCJiaXRuZXNzIjo2NCwiYXJjaGl0ZWN0dXJlIjoiYXJtNjQifSwic2VjdXJpdHkiOiJSZXN0cmljdGVkIn0=
messages: []
hadErrors: false
```

## Parameters

The `context()` function takes no arguments.

```yaml
Type:         none
Required:     false
MinimumCount: 0
MaximumCount: 0
```

## Output

Returns an object with two top-level properties: **os** and **security**. Use dot-path notation
to access nested properties, such as `context().os.family`.

```yaml
Type: object
```

## Related functions

- [`parameters()`][00] - Returns the value of a configuration parameter.
- [`variables()`][01] - Returns the value of a configuration variable.
- [`string()`][02] - Converts a value to its string representation.
- [`base64()`][03] - Returns the base64 representation of a string.

<!-- Link reference definitions -->
[00]: ./parameters.md
[01]: ./variables.md
[02]: ./string.md
[03]: ./base64.md
