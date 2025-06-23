# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

@{

    # Script module or binary module file associated with this manifest.
    RootModule        = 'wmiAdapter.psm1'

    # Version number of this module.
    moduleVersion     = '1.0.0'

    # ID used to uniquely identify this module
    GUID              = '420c66dc-d243-4bf8-8de0-66467328f4b7'

    # Author of this module
    Author            = 'Microsoft Corporation'

    # Company or vendor of this module
    CompanyName       = 'Microsoft Corporation'

    # Copyright statement for this module
    Copyright         = '(c) Microsoft Corporation. All rights reserved.'

    # Description of the functionality provided by this module
    Description       = 'PowerShell Desired State Configuration Module for DSC WMI Adapter'

    # Functions to export from this module, for best performance, do not use wildcards and do not delete the entry, use an empty array if there are no functions to export.
    FunctionsToExport = @(
        'Invoke-DscWmi'
    )

    # Cmdlets to export from this module, for best performance, do not use wildcards and do not delete the entry, use an empty array if there are no cmdlets to export.
    CmdletsToExport   = @()

    # Variables to export from this module
    VariablesToExport = @()

    # Aliases to export from this module, for best performance, do not use wildcards and do not delete the entry, use an empty array if there are no aliases to export.
    AliasesToExport   = @()

    PrivateData       = @{
        PSData = @{
            DscCapabilities = @(
            'get'
            'test'
            'set'
            )
            
            ProjectUri = 'https://github.com/PowerShell/dsc'
        }
    }
}