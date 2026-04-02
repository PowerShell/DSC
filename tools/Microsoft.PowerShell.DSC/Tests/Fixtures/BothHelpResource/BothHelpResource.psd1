# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

@{
    RootModule        = 'BothHelpResource.psm1'
    ModuleVersion     = '1.0.0'
    GUID              = 'f6a7b8c9-d0e1-2345-f012-567890123def'
    Author            = 'Microsoft'
    CompanyName       = 'Microsoft Corporation'
    Copyright         = '(c) Microsoft. All rights reserved.'
    Description       = 'Module with two classes, both with help.'
    FunctionsToExport = @()
    CmdletsToExport   = @()
    VariablesToExport = @()
    AliasesToExport   = @()
    DscResourcesToExport = @('FirstResource', 'SecondResource')
}
