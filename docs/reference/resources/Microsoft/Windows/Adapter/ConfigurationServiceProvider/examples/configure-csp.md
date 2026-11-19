---
description: >
  Configure a device policy through the CSP adapter, test the specified setting,
  and export its class instances.
ms.date: 10/25/2026
ms.topic: reference
title: Configure, test, and export a device CSP
---

# Configure, test, and export a device CSP

## Configure and test a device policy

The example disables internet sharing through the Policy CSP's Wi-Fi configuration class.
Check the [Wi-Fi CSP policy documentation][01] for Windows edition and version requirements.
Run every command in this example as **LocalSystem**.

> [!WARNING]
> **Set** changes a device policy. Use a test device and record the original policy before applying
> the example. Deleting an instance removes the entire Wi-Fi policy instance, not one property.

```powershell
$json = @{
    ParentID = './Vendor/MSFT/Policy/Config'
    InstanceID = 'WiFi'
    AllowInternetSharing = 0
} | ConvertTo-Json -Compress

$json | dsc resource get -r Microsoft.Windows.CSP/MDM_Policy_Config01_Wifi02 -f -
$json | dsc resource set -r Microsoft.Windows.CSP/MDM_Policy_Config01_Wifi02 -f -
$json | dsc resource test -r Microsoft.Windows.CSP/MDM_Policy_Config01_Wifi02 -f -
```

**Set** returns the provider's actual state, including values the provider normalized. It preserves
unspecified properties of an existing instance. **Test** reports `inDesiredState: true` when the
requested keys and `AllowInternetSharing` match. Other policy properties do not affect that test.

You can also use the [configuration document][02]:

```powershell
dsc config set --file .\wifi.config.dsc.yaml
```

## Export the Wi-Fi configuration class

```powershell
dsc resource export -r Microsoft.Windows.CSP/MDM_Policy_Config01_Wifi02
```

The result is a configuration document with one resource per visible instance of this class.
An optional key filter narrows the export:

```powershell
'{"InstanceID":"WiFi"}' |
    dsc resource export -r Microsoft.Windows.CSP/MDM_Policy_Config01_Wifi02 -f -
```

Exported values describe configured bridge state, not necessarily effective policy after MDM or
Group Policy precedence is resolved. Inspect the corresponding read-only `MDM_Policy_Result*`
class where available when evaluating resultant policy.

<!-- Link definitions -->
[01]: /windows/client-management/mdm/policy-csp-wifi
[02]: ./wifi.config.dsc.yaml
