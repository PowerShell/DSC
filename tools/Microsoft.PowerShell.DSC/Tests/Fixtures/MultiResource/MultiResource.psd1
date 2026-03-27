# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

@{
    RootModule        = 'MultiResource.psm1'
    ModuleVersion     = '2.5.0'
    GUID              = 'b2c3d4e5-f6a7-8901-bcde-f12345678901'
    Author            = 'Microsoft'
    CompanyName       = 'Microsoft Corporation'
    Copyright         = '(c) Microsoft. All rights reserved.'
    Description       = 'Module with multiple DSC resources.'
    FunctionsToExport = @()
    CmdletsToExport   = @()
    VariablesToExport = @()
    AliasesToExport   = @()
    DscResourcesToExport = @('ResourceA', 'ResourceB')
}
