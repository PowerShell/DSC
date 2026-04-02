# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

@{
    RootModule        = 'MixedHelpResource.psm1'
    ModuleVersion     = '1.0.0'
    GUID              = 'e5f6a7b8-c9d0-1234-ef01-456789012cde'
    Author            = 'Microsoft'
    CompanyName       = 'Microsoft Corporation'
    Copyright         = '(c) Microsoft. All rights reserved.'
    Description       = 'Module with two classes, one with help and one without.'
    FunctionsToExport = @()
    CmdletsToExport   = @()
    VariablesToExport = @()
    AliasesToExport   = @()
    DscResourcesToExport = @('DocumentedResource', 'UndocumentedResource')
}
