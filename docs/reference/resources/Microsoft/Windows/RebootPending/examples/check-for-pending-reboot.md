---
description: >
    Example showing how to use the Microsoft.Windows/RebootPending resource with DSC to check if a Windows system has a pending reboot.
ms.date: 03/25/2025
ms.topic: reference
title: Check for pending reboot
---

# Check for pending reboot

This example shows how you can use the `Microsoft.Windows/RebootPending` resource to check whether a Windows system has a pending reboot.

## Check reboot status

The following snippet shows how to use the resource with the [dsc resource get][01] command to retrieve the pending reboot status.

```powershell
dsc resource get --resource Microsoft.Windows/RebootPending
```

When you run this command, DSC returns the following result if a reboot is pending:

```yaml
actualState:
    rebootPending: true
```

If no reboot is pending, the result is:

```yaml
actualState:
    rebootPending: false
```

The `rebootPending` property indicates whether the system requires a reboot (`true`) or not (`false`).

> The resource doesn't implement the **Set**, **WhatIf**, **Export**, **Delete**, or **Test**
> capabilities. You can't use this resource to enforce or export configurations.
>
> Note that even though the resource doesn't implement **Test**, you can still invoke the test
> operation against the resource and use it in the `Microsoft.Dsc/Assertion` group resource. This
> resource relies on the synthetic testing provided by DSC. For more information about synthetic
> testing with DSC, see
> [DSC resource capabiltiies](../../../../../concepts/resources/capabilities.md#test).
>
> For an example using this resource in an assertion, see
> [Use the RebootPending resource in a configuration](./use-rebootpending-in-configuration.md).

<!-- Link reference definitions -->
[01]: ../../../../../cli/resource/get.md