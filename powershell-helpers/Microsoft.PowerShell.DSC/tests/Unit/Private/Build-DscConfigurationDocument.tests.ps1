BeforeAll {
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

Describe 'Build-DscConfigurationDocument' {
    Context 'Build Desired State Configuration document' {
        BeforeAll {
            New-Item -Path (Join-Path -Path $TestDrive -ChildPath 'test.ps1') -ItemType File
            $content = @'
configuration MyConfiguration {
    Import-DscResource -ModuleName PSDesiredStateConfiguration 
    Node localhost
    {
        Environment CreatePathEnvironmentVariable
        {
            Name = 'TestPathEnvironmentVariable'
            Value = 'TestValue'
            Ensure = 'Present'
            Path = $true
            Target = @('Process')
        }
    }
}
'@
            Set-Content -Path (Join-Path -Path $TestDrive -ChildPath 'test.ps1') -Value $content
        }

        AfterAll {
            Remove-Item -Path (Join-Path -Path $TestDrive -ChildPath 'test.ps1') -Recurse -Force
        }
        # TODO: Timing because of Get-DscResource in function
        It 'Should return a valid DSC Configuration Document in JSON' {
            InModuleScope -ScriptBlock {
                $file = Build-DscConfigurationDocument -Path (Join-Path -Path $TestDrive -ChildPath 'test.ps1') -Format JSON
                $file | Should -Not -BeNullOrEmpty
            }
        }

        It 'Should return a valid DSC Configuration Document in YAML' {
            InModuleScope -ScriptBlock {
                $file = Build-DscConfigurationDocument -Path (Join-Path -Path $TestDrive -ChildPath 'test.ps1') -Format YAML
                $file | Should -Not -BeNullOrEmpty
            }
        }

        It 'Should return a valid DSC Configuration Document in default format' {
            InModuleScope -ScriptBlock {
                $file = Build-DscConfigurationDocument -Path (Join-Path -Path $TestDrive -ChildPath 'test.ps1') -Format Default
                $file | Should -Not -BeNullOrEmpty
            }
        }
    }
}