@{
    # Script module or binary module file associated with this manifest.
    RootModule            = 'TestScriptBaseDSC.psm1'

    # Version number of this module.
    moduleVersion        = '0.0.1'

    # ID used to uniquely identify this module
    GUID                 = 'c3775be8-84a1-43f5-a99c-1b9f2d6bc178'

    # Author of this module
    Author               = ''

    # Company or vendor of this module
    CompanyName          = ''

    # Copyright statement for this module
    Copyright            = ''

    # Description of the functionality provided by this module
    Description          = ''

    # Minimum version of the Windows PowerShell engine required by this module
    PowerShellVersion    = '5.0'

    # Cmdlets to export from this module
    CmdletsToExport      = @()

    # Variables to export from this module
    VariablesToExport    = @()

    # Aliases to export from this module
    AliasesToExport      = @()

    # Dsc Resources to export from this module
    DscResourcesToExport = @('CredentialValidation')

    # Private data to pass to the module specified in RootModule/ModuleToProcess. This may also contain a PSData hashtable with additional module metadata used by PowerShell.
    PrivateData          = @{

        PSData = @{

            # Tags applied to this module. These help with module discovery in online galleries.
            Tags         = @('DesiredStateConfiguration', 'DSC', 'DSCResourceKit', 'DSCResource')

            # A URL to the license for this module.
            LicenseUri   = ''

            # A URL to the main website for this project.
            ProjectUri   = ''

            # A URL to an icon representing this module.
            IconUri      = ''

            # ReleaseNotes of this module
            ReleaseNotes = ''

            # Set to a prerelease string value if the release should be a prerelease.
            Prerelease   = ''
        } # End of PSData hashtable
    } # End of PrivateData hashtable
}
