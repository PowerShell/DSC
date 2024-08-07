BeforeAll {
    # TODO: Find way how to install / uninstall powershell-yaml or unload from current session
    $script:moduleName = 'Microsoft.PowerShell.DSC'

    # If the module is not found, run the build task 'noop'.
    if (-not (Get-Module -Name $script:moduleName -ListAvailable))
    {
        # Redirect all streams to $null, except the error stream (stream 2)
        & "$PSScriptRoot/../../build.ps1" -Tasks 'noop' 2>&1 4>&1 5>&1 6>&1 > $null
    }

    # Re-import the module using force to get any code changes between runs.
    Import-Module -Name $script:moduleName -Force -ErrorAction 'Stop'

    $PSDefaultParameterValues['InModuleScope:ModuleName'] = $script:moduleName
}

AfterAll {
    $PSDefaultParameterValues.Remove('InModuleScope:ModuleName')

    Remove-Module -Name $script:moduleName
}

Describe 'Test-YamlModule' {
    Context 'When the module is found' {
        It 'Should return true because module exist' {
            InModuleScope -ScriptBlock {
                $result = Test-YamlModule
                $result | Should -BeTrue
            }
        }
    }
}