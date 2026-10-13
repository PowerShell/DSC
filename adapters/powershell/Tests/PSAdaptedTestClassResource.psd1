# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

@{
    RootModule = 'PSAdaptedTestClassResource.psm1'
    ModuleVersion = '0.1.0'
    GUID = '6592806d-ceef-4949-b576-b5bf32eefcef'
    Author = 'Microsoft'
    CompanyName = 'Microsoft Corporation'
    Copyright = '(c) Microsoft. All rights reserved.'
    FunctionsToExport = @()
    CmdletsToExport = @()
    VariablesToExport = @()
    AliasesToExport = @()
    DscResourcesToExport = @('PSAdaptedTestClass')

    # Private data to pass to the module specified in RootModule/ModuleToProcess. This may also contain a PSData hashtable with additional module metadata used by PowerShell.
    PrivateData = @{
        PSData = @{
            DscCapabilities = @(
                'get'
            )
        }
    }
}
