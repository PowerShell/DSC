# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Apt resource tests' {
    BeforeAll {
        $aptExists = ($null -ne (Get-Command apt -CommandType Application -ErrorAction Ignore))
    }

    Context "export" {
        It "should have more than 20 resources" -Skip:$(! $IsLinux) {
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

        It 'Config get works' -Skip:$(! $IsLinux) {
            if (-not $aptExists) {
                Set-ItResult -Skip -Because "Apt not found"
            }
            $out = dsc -l trace config get -f $yamlPath 2> "$TestDrive/stderr.txt" | ConvertFrom-Json -Depth 10
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content "$TestDrive/stderr.txt" | Out-String)
            $exists = $null -ne (Get-Command $pkgName -CommandType Application -ErrorAction Ignore)
            $observed = $out.results[1].result.actualState._exist
            $observed | Should -Be $exists
        }

        It 'Config test works' -Skip:$(! $IsLinux) {
            if (-not $aptExists) {
                Set-ItResult -Skip -Because "Apt not found"
            }

            $out = dsc config test -f $yamlPath| ConvertFrom-Json -Depth 10
            $LASTEXITCODE | Should -Be 0
            $exists = $null -ne (Get-Command $pkgName -CommandType Application -ErrorAction Ignore)
            $out.results[1].result.inDesiredState | Should -Be $exists
        }
    }

    Context "install/uninstall rolldice tests" {
        BeforeAll {
            $pkgName = "rolldice"
            $yamlInstallPath = "$PSScriptRoot/assets/apt_install_${pkgName}.dsc.yaml"
            $yamlUnInstallPath = "$PSScriptRoot/assets/apt_uninstall_${pkgName}.dsc.yaml"
        }

        It 'Can install a package' -Skip:$(! $IsLinux) {
            Set-ItResult -Skip -Because "Apt requires sudo"

            if (apt list $pkgname 2>&1 | Select-String installed ) {
                apt remove -y $pkgname
            }

            $result = dsc config set -f $yamlInstallPath | ConvertFrom-Json
            $result.results[1].result.beforestate._exist | Should -Be false
            $result.results[1].result.afterstate._exist | Should -Be true
        }

        It 'Can uninstall a package' -Skip:$(! $IsLinux) {
            Set-ItResult -Skip -Because "Apt requires sudo"

            if ($null -eq (apt list $pkgName 2>&1 | Select-String installed)) {
                apt install -y $pkgname
            }

            $result = dsc config set -f $yamlUnInstallPath | ConvertFrom-Json
            $result.results[1].result.beforestate._exist | Should -Be true
            $result.results[1].result.afterstate._exist | Should -Be false
        }
    }
}
