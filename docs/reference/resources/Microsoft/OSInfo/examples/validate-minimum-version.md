---
description: Validate a minimum operating system version with Microsoft/OSInfo
ms.date:     07/11/2026
ms.topic:    reference
title:       Validate a minimum operating system version
---

# Validate a minimum operating system version

This example uses the `Microsoft/OSInfo` resource with the `Microsoft.DSC/Assertion` group
resource to verify that the operating system version meets a minimum requirement before DSC runs
another resource.

> [!IMPORTANT]
> The `osinfo` command and `Microsoft/OSInfo` resource are a proof-of-concept example for use with
> DSC. Don't use it in production.

## Definition

The **Operating System Assertion** group contains a `Microsoft/OSInfo` resource instance with the
version constraint `>= 10.0`. The `Microsoft.DSC/Assertion` resource always invokes **Test** for
nested instances. If the operating system is earlier than version `10.0`, the configuration fails
and DSC doesn't invoke **Show operating system**.

:::code language="yaml" source="validate-minimum-version.config.dsc.yaml":::

## Running the configuration

Run the configuration with the [dsc config set][01] command:

```bash
dsc config set --file ./validate-minimum-version.config.dsc.yaml
```

On an operating system whose version is at least `10.0`, DSC returns successful results for both
the assertion group and the dependent echo resource:

```yaml
results:
- name: Operating System Assertion
  type: Microsoft.DSC/Assertion
  result:
    beforeState:
    - name: Minimum operating system version
      type: Microsoft/OSInfo
      result:
        actualState:
          family: Windows
          version: 10.0.26200
          _inDesiredState: true
    afterState:
    - name: Minimum operating system version
      type: Microsoft/OSInfo
      result:
        desiredState:
          version: '>= 10.0'
        actualState:
          family: Windows
          version: 10.0.26200
          edition: Windows 11
          bitness: 64
          architecture: x86_64
          _inDesiredState: true
        inDesiredState: true
        differingProperties:
        - version
    changedProperties: []
- name: Show operating system
  type: Microsoft.DSC.Debug/Echo
  result:
    beforeState:
      output: The operating system meets the minimum version requirement.
    afterState:
      output: The operating system meets the minimum version requirement.
    changedProperties: null
messages: []
hadErrors: false
```

<!-- Link references -->
[01]: ../../../../cli/config/set.md
