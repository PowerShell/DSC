# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Test-ProjectWithPester' {
    BeforeAll {
        Import-Module (Join-Path $PSScriptRoot '..' 'helpers.build.psm1') -Force
        $buildData = Import-DscBuildData -RefreshProjects
        $dscProject = $buildData.Projects | Where-Object -Property Name -EQ 'dsc'
        $fileContentProject = $buildData.Projects | Where-Object -Property Name -EQ 'filecontent'
    }

    BeforeEach {
        $originalPSModulePath = $env:PSModulePath
        Mock Invoke-Pester -ModuleName helpers.build
    }

    AfterEach {
        $env:PSModulePath = $originalPSModulePath
    }

    It 'Runs tests only from the selected project paths' {
        Test-ProjectWithPester -Project @($dscProject, $fileContentProject)

        Should -Invoke Invoke-Pester -ModuleName helpers.build -Exactly 1 -ParameterFilter {
            $Path.Count -eq 2 -and
            $Path -contains 'dsc' -and
            $Path -contains 'resources/filecontent'
        }
    }

    Describe 'build.ps1 project selection' {
        It 'Fails when no projects match the requested names' {
            $pwsh = (Get-Process -Id $PID).Path
            $output = & $pwsh -NoProfile -File (Join-Path $PSScriptRoot '..' 'build.ps1') `
                -SkipBuild -Test -ExcludeRustTests -ExcludePesterTests -Project 'does-not-exist' 2>&1

            $LASTEXITCODE | Should -Not -Be 0
            $output -join "`n" | Should -Match 'No projects matched the specified project names: does-not-exist'
        }
    }

    It 'Intersects selected projects with Pester test groups' {
        Test-ProjectWithPester -Project @($dscProject, $fileContentProject) -Group 'resources'

        Should -Invoke Invoke-Pester -ModuleName helpers.build -Exactly 1 -ParameterFilter {
            $Path.Count -eq 1 -and $Path -contains 'resources/filecontent'
        }
    }

    It 'Uses the narrower test group when the root project is selected' {
        $rootProject = $buildData.Projects | Where-Object -Property Name -EQ 'root'

        Test-ProjectWithPester -Project $rootProject -Group 'resources'

        Should -Invoke Invoke-Pester -ModuleName helpers.build -Exactly 1 -ParameterFilter {
            $Path.Count -eq 1 -and $Path -contains 'resources'
        }
    }

    It 'Fails when selected projects do not belong to a Pester test group' {
        {
            Test-ProjectWithPester -Project $dscProject -Group 'resources'
        } | Should -Throw 'None of the selected projects are in the specified Pester test groups.'

        Should -Invoke Invoke-Pester -ModuleName helpers.build -Exactly 0
    }
}
