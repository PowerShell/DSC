@{
    RootModule        = 'TestClassResource.psm1'
    ModuleVersion     = '1.0.0'
    GUID              = '5d73a601-4a6c-43c5-ba3f-619b18bbb404'
    Author            = 'Microsoft Corporation'
    CompanyName       = 'Microsoft Corporation'
    Copyright         = '(c) Microsoft Corporation. All rights reserved.'
    Description       = 'PowerShell module for testing DSCv3'
    PowerShellVersion = '5.0'
    DscResourcesToExport = 'TestClassResource'
    FunctionsToExport = @(
	    'Test-World')
    VariablesToExport = '@()'
    AliasesToExport   = @()
    PrivateData       = @{
        PSData = @{
            Tags         = @(
                'DSC',
                'PSEdition_Desktop',
                'PSEdition_Core',
                'Linux',
                'Mac')
            DscCapabilities = @('Get', 'Test')
        }
    }
}
