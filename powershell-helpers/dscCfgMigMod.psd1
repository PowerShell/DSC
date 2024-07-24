@{

    # Script module or binary module file associated with this manifest.
    RootModule        = 'dscCfgMigMod.psm1'
    
    # Version number of this module.
    moduleVersion     = '0.0.1'
    
    # ID used to uniquely identify this module
    GUID              = '42bf8cb0-210c-4dac-8614-319d9287c6dc'
    
    # Author of this module
    Author            = 'Microsoft Corporation'
    
    # Company or vendor of this module
    CompanyName       = 'Microsoft Corporation'
    
    # Copyright statement for this module
    Copyright         = '(c) Microsoft Corporation. All rights reserved.'
    
    # Description of the functionality provided by this module
    Description       = 'PowerShell Desired State Configuration Migration Module helper'

    # Modules that must be imported into the global environment prior to importing this module
    RequiredModules = @('powershell-yaml')
    
    # Functions to export from this module, for best performance, do not use wildcards and do not delete the entry, use an empty array if there are no functions to export.
    FunctionsToExport = @(
        'ConvertTo-DscJson'
        'ConvertTo-DscYaml'
    )
    
    # Cmdlets to export from this module, for best performance, do not use wildcards and do not delete the entry, use an empty array if there are no cmdlets to export.
    CmdletsToExport   = @()
    
    # Variables to export from this module
    VariablesToExport = @()
    
    # Aliases to export from this module, for best performance, do not use wildcards and do not delete the entry, use an empty array if there are no aliases to export.
    AliasesToExport   = @()
    
    PrivateData       = @{
        PSData = @{
            ProjectUri = 'https://github.com/PowerShell/dsc'
        }
    }
}
