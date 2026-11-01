# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

$originalResourcePath = $env:DSC_RESOURCE_PATH
$originalRestrictedPath = $env:DSC_RESTRICTED_PATH
$resourceDirectory = New-Item -Path (Join-Path $TestDrive 'resources') -ItemType Directory -Force
Copy-Item -LiteralPath (Get-Command csp -CommandType Application).Source -Destination $resourceDirectory
Copy-Item -LiteralPath (Join-Path $PSScriptRoot '..' 'csp.dsc.resource.json') -Destination $resourceDirectory
$env:DSC_RESOURCE_PATH = $resourceDirectory.FullName
$env:DSC_RESTRICTED_PATH = $resourceDirectory.FullName
$adapterType = 'Microsoft.Windows.Adapter/ConfigurationServiceProvider'
$resourceType = 'Microsoft.Windows.CSP/MDM_Policy_User_Config01_Authentication02'
$identity = [Security.Principal.WindowsIdentity]::GetCurrent()
$isSystem = $identity.User.Value -eq 'S-1-5-18'
$inputJson = @{
    ParentID = './Vendor/MSFT/Policy/Config'
    InstanceID = 'Authentication'
} | ConvertTo-Json -Compress
