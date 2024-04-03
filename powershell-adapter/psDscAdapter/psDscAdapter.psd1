# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

@{

# Script module or binary module file associated with this manifest.
RootModule = 'psDscAdapter.psm1'

# Version number of this module.
moduleVersion = '0.0.1'

# ID used to uniquely identify this module
GUID = 'e0dd561d-c47f-4132-aac9-cd9dc8739bb1'

# Author of this module
Author = 'Microsoft Corporation'

# Company or vendor of this module
CompanyName = 'Microsoft Corporation'

# Copyright statement for this module
Copyright = '(c) Microsoft Corporation. All rights reserved.'

# Description of the functionality provided by this module
Description = 'PowerShell Desired State Configuration Module for DSC PowerShell Adapter'

# Functions to export from this module, for best performance, do not use wildcards and do not delete the entry, use an empty array if there are no functions to export.
FunctionsToExport = @(
        'Get-DscResource'
        'Invoke-DscResource'
        'Get-ConfigObject'
        'Invoke-CacheRefresh'
        'Get-ActualState'
    )

# Cmdlets to export from this module, for best performance, do not use wildcards and do not delete the entry, use an empty array if there are no cmdlets to export.
CmdletsToExport = @()

# Variables to export from this module
VariablesToExport = '*'

# Aliases to export from this module, for best performance, do not use wildcards and do not delete the entry, use an empty array if there are no aliases to export.
AliasesToExport = @()

PrivateData = @{
    PSData = @{
        ProjectUri   = 'https://github.com/PowerShell/dsc'
    }
}
}
