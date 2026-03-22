# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

@{

# Script module or binary module file associated with this manifest.
RootModule = 'Microsoft.PowerShell.DSC.psm1'

# Version number of this module.
ModuleVersion = '0.0.1'

# ID used to uniquely identify this module
GUID = 'f4a5e270-0e6b-4f6a-b08a-3a1d2c7e9b4d'

# Author of this module
Author = 'Microsoft Corporation'

# Company or vendor of this module
CompanyName = 'Microsoft Corporation'

# Copyright statement for this module
Copyright = '(c) Microsoft Corporation. All rights reserved.'

# Description of the functionality provided by this module
Description = 'Creates adapted resource manifests from class-based PowerShell DSC resources.'

# Minimum version of the PowerShell engine required by this module
PowerShellVersion = '7.0'

# Functions to export from this module, for best performance, do not use wildcards and do not delete the entry, use an empty array if there are no functions to export.
FunctionsToExport = @(
    'New-DscAdaptedResourceManifest'
)

# Cmdlets to export from this module, for best performance, do not use wildcards and do not delete the entry, use an empty array if there are no cmdlets to export.
CmdletsToExport = @()

# Variables to export from this module
VariablesToExport = @()

# Aliases to export from this module, for best performance, do not use wildcards and do not delete the entry, use an empty array if there are no aliases to export.
AliasesToExport = @()

PrivateData = @{
    PSData = @{
        ProjectUri = 'https://github.com/PowerShell/DSC'
    }
}
}
