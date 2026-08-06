---
description: Reference for the 'restartRequired' DSC configuration document function
ms.date:     07/11/2026
ms.topic:    reference
title:       restartRequired
---

# restartRequired

## Synopsis

Returns whether a system, service, or process requires a restart.

## Syntax

```Syntax
restartRequired('system')
restartRequired('service', '<serviceName>')
restartRequired('process', '<processName>')
```

## Description

The `restartRequired()` function returns whether a resource in the current configuration operation
reported a required restart. DSC aggregates restart requirements returned by resources in the
configuration context.

Use `system` to query whether a system restart is required. Use `service` or `process` to query a
specific service or process by name. For `service` and `process`, the `name` argument is required.
For `system`, the `name` argument isn't allowed.

The function returns `false` when no matching restart requirement has been reported.

## Examples

### Example 1 - Query restart requirements

This configuration queries whether installing the OpenSSH Client capability with the
[Microsoft.Windows/FeatureOnDemandList][01] resource requires a system restart. When the resource
reports a system restart requirement, `restartRequired()` returns `true`.

> [!NOTE]
> This example requires Windows, an elevated session, and access to a Windows Update or WSUS
> source. Capability installation does not always require a restart.

```yaml
# restartRequired.example.1.dsc.config.yaml
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: Install OpenSSH Client
  type: Microsoft.Windows/FeatureOnDemandList
  properties:
    capabilities:
    - identity: OpenSSH.Client~~~~0.0.1.0
      state: Installed
outputs:
  systemRestartRequired:
    type: bool
    value: "[restartRequired('system')]"
```

```bash
dsc config set --file restartRequired.example.1.dsc.config.yaml
```

When the resource instances report the corresponding restart requirements, DSC returns:

```yaml
outputs:
  systemRestartRequired: true
```

To query a service or process restart requirement, specify its name as the second argument:

```Syntax
restartRequired('service', 'example-service')
restartRequired('process', 'example-process')
```

## Parameters

### kind

The kind of restart requirement to query. The value must be `process`, `service`, or `system`.

```yaml
Type:         string
Required:     true
MinimumCount: 1
MaximumCount: 1
AllowedValues:
- process
- service
- system
```

### name

The name of the process or service to query. This argument is required when **kind** is `process`
or `service`, and it isn't allowed when **kind** is `system`.

```yaml
Type:         string
Required:     conditional
MinimumCount: 0
MaximumCount: 1
```

## Output

The `restartRequired()` function returns `true` when a matching restart requirement was reported
by a resource in the current configuration operation. Otherwise, it returns `false`.

```yaml
Type: bool
```

<!-- Link reference definitions -->
[01]: ../../../resources/Microsoft/Windows/FeatureOnDemandList/index.md
