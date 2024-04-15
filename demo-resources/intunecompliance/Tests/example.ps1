# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.
import-module microsoft.powershell.management -Version 3.1.0.0

$WMI_ComputerSystem = Get-WMIObject -class Win32_ComputerSystem
$WMI_BIOS = Get-WMIObject -class Win32_BIOS 
$TPM = Get-Tpm

$hash = @{ Manufacturer = $WMI_ComputerSystem.Manufacturer; BiosVersion = $WMI_BIOS.SMBIOSBIOSVersion; TPMChipPresent = $TPM.TPMPresent}
return $hash | ConvertTo-Json -Compress