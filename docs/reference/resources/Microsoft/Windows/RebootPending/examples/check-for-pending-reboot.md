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

> [!NOTE]
> You can only use the **Get** operation to check the reboot status.
> The resource does not support **Set**, **WhatIf**, **Export**, **Delete**, or **Test** operations.

<!-- Link reference definitions -->
[01]: ../../../../../cli/resource/get.md