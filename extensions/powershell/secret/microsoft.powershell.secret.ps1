# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$Name,
    [Parameter()]
    [string]$Vault
)

if (Get-Command Get-Secret -ErrorAction Ignore) {
    $secretParams = @{
        Name        = $Name
        AsPlainText = $true
    }

    if (-not ([string]::IsNullOrEmpty($Vault))) {
        $secretParams['Vault'] = $Vault
    }

    $secret = Get-Secret @secretParams -ErrorAction Ignore 

    Write-Output $secret
}