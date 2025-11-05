# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

@{
    RootModule           = 'PSTestClassResource.psm1'
    ModuleVersion        = '0.0.1'
    GUID                 = 'b267fa32-e77d-48e6-9248-676cc6f2327e'
    Author               = 'Microsoft'
    CompanyName          = 'Microsoft Corporation'
    Copyright            = '(c) Microsoft. All rights reserved.'
    FunctionsToExport    = @()
    CmdletsToExport      = @()
    VariablesToExport    = @()
    AliasesToExport      = @()
    DscResourcesToExport = @('PSTestClassResource', 'PSNoExport')
    PrivateData          = @{
        PSData = @{
            DscCapabilities = @(
                'get'
                'test'
            )
        }
    }
}

