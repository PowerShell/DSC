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

    It 'Intersects selected projects with Pester test groups' {
        Test-ProjectWithPester -Project @($dscProject, $fileContentProject) -Group 'resources'

        Should -Invoke Invoke-Pester -ModuleName helpers.build -Exactly 1 -ParameterFilter {
            $Path.Count -eq 1 -and $Path -contains 'resources/filecontent'
        }
    }

    It 'Fails when selected projects do not belong to a Pester test group' {
        {
            Test-ProjectWithPester -Project $dscProject -Group 'resources'
        } | Should -Throw 'None of the selected projects are in the specified Pester test groups.'

        Should -Invoke Invoke-Pester -ModuleName helpers.build -Exactly 0
    }
}
