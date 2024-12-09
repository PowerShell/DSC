# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

@{

    # Script module or binary module file associated with this manifest.
    RootModule           = 'TestClassNoVersion.psm1'

    # Version number of this module.
    ModuleVersion        = '0.0.1'

    # Supported PSEditions
    # CompatiblePSEditions = @()

    # ID used to uniquely identify this module
    GUID                 = 'ec985d60-82f4-4d45-83e0-b6f935654350'

    # Author of this module
    Author               = 'Microsoft'

    # Company or vendor of this module
    CompanyName          = 'Microsoft Corporation'

    # Copyright statement for this module
    Copyright            = '(c) Microsoft. All rights reserved.'

    # Functions to export from this module, for best performance, do not use wildcards and do not delete the entry, use an empty array if there are no functions to export.
    FunctionsToExport    = @()

    # Cmdlets to export from this module, for best performance, do not use wildcards and do not delete the entry, use an empty array if there are no cmdlets to export.
    CmdletsToExport      = '*'

    # Variables to export from this module
    VariablesToExport    = @()

    # Aliases to export from this module, for best performance, do not use wildcards and do not delete the entry, use an empty array if there are no aliases to export.
    AliasesToExport      = @()

    # DSC resources to export from this module
    DscResourcesToExport = 'TestClassNoVersion'

    # Private data to pass to the module specified in RootModule/ModuleToProcess. This may also contain a PSData hashtable with additional module metadata used by PowerShell.
    PrivateData          = @{

        PSData = @{
            DscCapabilities = @('Get', 'Test', 'Set')

        } # End of PSData hashtable

    } # End of PrivateData hashtable
}

