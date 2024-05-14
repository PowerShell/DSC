# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Apt resource tests' {
    BeforeAll {
        $aptExists = ($null -ne (Get-Command apt -CommandType Application -ErrorAction Ignore))
    }

    Context "export" {
        It "should have more than 20 resources" {
            if (-not $aptExists) {
                Set-ItResult -Skip -Because "Apt not found"
            }

            $result = dsc resource export --resource DSC.PackageManagement/Apt | ConvertFrom-Json
            $result.resources.Count | Should -BeGreaterThan 20
        }
    }

    Context "wget tests" {
        BeforeAll {
            $pkgName = "wget"
            $yamlPath = "$PSScriptRoot/assets/apt_${pkgName}.dsc.yaml"
        }

        It 'Config get works' {
            if (-not $aptExists) {
                Set-ItResult -Skip -Because "Apt not found"
            }
            $out = dsc config get -p $yamlPath | ConvertFrom-Json -Depth 10
            $LASTEXITCODE | Should -Be 0
            $exists = $null -ne (Get-Command $pkgName -CommandType Application -ErrorAction Ignore)
            $observed = $out.results[1].result.actualState._exist
            $observed | Should -Be $exists
        }

        It 'Config test works' {
            if (-not $aptExists) {
                Set-ItResult -Skip -Because "Apt not found"
            }

            $out = dsc config test -p $yamlPath| ConvertFrom-Json -Depth 10
            $LASTEXITCODE | Should -Be 0
            $exists = $null -ne (Get-Command pkgName -CommandType Application -ErrorAction Ignore)
            $out.results[1].result.inDesiredState | Should -Be $exists
        }
    }

    Context "install/uninstall rolldice tests" {
        BeforeAll {
            $pkgName = "rolldice"
            $yamlInstallPath = "$PSScriptRoot/assets/apt_install_${pkgName}.dsc.yaml"
            $yamlUnInstallPath = "$PSScriptRoot/assets/apt_uninstall_${pkgName}.dsc.yaml"
        }

        It 'Can install a package' {
            Set-ItResult -Skip -Because "Apt requires sudo"

            if (apt list $pkgname 2>&1 | Select-String installed ) {
                apt remove -y $pkgname
            }

            $result = dsc config set -p $yamlInstallPath | ConvertFrom-Json
            $result.results[1].result.beforestate._exist | Should -Be false
            $result.results[1].result.afterstate._exist | Should -Be true
        }

        It 'Can uninstall a package' {
            Set-ItResult -Skip -Because "Apt requires sudo"

            if ($null -eq (apt list $pkgName 2>&1 | Select-String installed)) {
                apt install -y $pkgname
            }

            $result = dsc config set -p $yamlUnInstallPath | ConvertFrom-Json
            $result.results[1].result.beforestate._exist | Should -Be true
            $result.results[1].result.afterstate._exist | Should -Be false
        }
    }
}
