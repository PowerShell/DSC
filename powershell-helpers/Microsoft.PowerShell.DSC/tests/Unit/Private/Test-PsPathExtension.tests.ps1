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

Describe 'Test-PsPathExtension' {
    Context 'When file paths are correct' {
        BeforeAll {
            New-Item -Path (Join-Path -Path $TestDrive -ChildPath 'test.ps1') -ItemType File
        }

        AfterAll {
            Remove-Item -Path (Join-Path -Path $TestDrive -ChildPath 'test.ps1') -Recurse -Force
        }

        It 'Should return PowerShell script file is true' {
            InModuleScope -ScriptBlock {
                $result = Test-PsPathExtension -Path (Join-Path -Path $TestDrive -ChildPath 'test.ps1')

                $result | Should -BeTrue
            }
        }
    }

    Context 'When file paths are incorrect' {
        BeforeAll {
            New-Item -Path (Join-Path -Path $TestDrive -ChildPath 'test.psm1') -ItemType File
        }

        AfterAll {
            Remove-Item -Path (Join-Path -Path $TestDrive -ChildPath 'test.psm1') -Recurse -Force
        }

        It 'Should return PowerShell script file is false because it is a PowerShell module file' {
            InModuleScope -ScriptBlock {
                $result = Test-PsPathExtension -Path (Join-Path -Path $TestDrive -ChildPath 'test.psm1')

                $result | Should -BeFalse
            }
        }

        It 'Should return false because file path does not exist' {
            InModuleScope -ScriptBlock {
                $result = Test-PsPathExtension -Path (Join-Path -Path $TestDrive -ChildPath 'thisdoesnotexist.txt')

                $result | Should -BeFalse
            }
        }
    }
}