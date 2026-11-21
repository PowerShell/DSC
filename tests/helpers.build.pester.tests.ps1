# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Test-ProjectWithPester' {
    BeforeAll {
        Import-Module (Join-Path $PSScriptRoot '..' 'helpers.build.psm1') -Force
        $buildData = Import-DscBuildData -RefreshProjects
        $dscProject = $buildData.Projects | Where-Object -Property Name -EQ 'dsc'
        $fileContentProject = $buildData.Projects | Where-Object -Property Name -EQ 'file_dsc'
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
            $Path -contains 'resources/file_dsc'
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
            $Path.Count -eq 1 -and $Path -contains 'resources/file_dsc'
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

Describe 'Build-DscResourcePackage' {
    BeforeAll {
        Import-Module (Join-Path $PSScriptRoot '..' 'helpers.build.psm1') -Force

        function Get-ZipEntryName {
            param([string]$Path)

            $archive = [System.IO.Compression.ZipFile]::OpenRead($Path)
            try {
                $archive.Entries.FullName -replace '\\', '/'
            } finally {
                $archive.Dispose()
            }
        }
    }

    BeforeEach {
        $buildData = Import-DscBuildData
        $bin = Join-Path $TestDrive ([guid]::NewGuid().ToString())
        $null = New-Item -Path $bin -ItemType Directory
        foreach ($section in @('All', 'Linux', 'macOS', 'Windows')) {
            foreach ($package in $buildData.ResourcePackageFiles.$section.Values) {
                foreach ($file in $package.zipContents) {
                    $path = Join-Path $bin $file
                    $null = New-Item -Path (Split-Path -Parent $path) -ItemType Directory -Force
                    Set-Content -LiteralPath $path -Value $file
                    if (-not [System.IO.Path]::HasExtension($file)) {
                        Set-Content -LiteralPath "$path.exe" -Value $file
                    }
                }
            }
        }
        $artifactDirectory = @{ Bin = $bin }
        $outputDirectory = Join-Path $bin 'resource-packages'
    }

    It 'Preserves package definitions when serializing refreshed build data' {
        $refreshed = Import-DscBuildData -RefreshProjects
        $roundTrip = $refreshed.ToJson() | ConvertFrom-Json -AsHashtable

        foreach ($section in @('All', 'Linux', 'macOS', 'Windows')) {
            $expected = $buildData.ResourcePackageFiles.$section
            $actual = $roundTrip.ResourcePackageFiles.$section
            $actual.Count | Should -Be $expected.Count
            foreach ($name in $expected.Keys) {
                $actual[$name].zipContents | Should -Be $expected[$name].zipContents
                $actual[$name].manifests | Should -Be $expected[$name].manifests
            }
        }
    }

    It 'Excludes separately packaged binaries from distribution package file lists' {
        foreach ($platform in @('Linux', 'macOS', 'Windows')) {
            foreach ($section in @('All', $platform)) {
                foreach ($package in $buildData.ResourcePackageFiles.$section.Values) {
                    foreach ($file in $package.zipContents) {
                        if ([System.IO.Path]::HasExtension($file) -and
                            [System.IO.Path]::GetExtension($file) -ne '.exe') {
                            continue
                        }
                        if ($platform -eq 'Windows' -and -not [System.IO.Path]::HasExtension($file)) {
                            $file = "$file.exe"
                        } elseif ($platform -ne 'Windows' -and $file.EndsWith('.exe')) {
                            $file = $file.Substring(0, $file.Length - 4)
                        }
                        $buildData.PackageFiles.$platform | Should -Not -Contain $file
                    }
                }
            }
        }
    }

    It 'Preserves distribution manifests, scripts, and support files' {
        foreach ($platform in @('Linux', 'macOS', 'Windows')) {
            foreach ($file in @(
                'process.dsc.manifests.json',
                'file_dsc.dsc.manifests.json',
                'sshd_config.dsc.resource.json',
                'RunCommandOnSet.dsc.resource.json',
                'psscript.ps1',
                'psscript.dsc.resource.json'
            )) {
                $buildData.PackageFiles.$platform | Should -Contain $file
            }
        }
        foreach ($file in @(
            'registry.dsc.manifests.json',
            'optionalfeature.dsc.resource.json',
            'featureondemand.dsc.resource.json',
            'windows_feature.dsc.resource.json',
            'sshd-windows.dsc.resource.json',
            'winpsscript.dsc.resource.json',
            'reboot_pending.resource.ps1',
            'wmi.resource.ps1',
            'wmiAdapter.psd1',
            'wmiAdapter.psm1',
            'windowspowershell.dsc.resource.json',
            'WindowsPowerShell_adapter.dsc.resource.json'
        )) {
            $buildData.PackageFiles.Windows | Should -Contain $file
        }
        $buildData.PackageFiles.Linux | Should -Contain 'apt.dsc.resource.sh'
        $buildData.PackageFiles.Linux | Should -Contain 'apt.dsc.resource.json'
        $buildData.PackageFiles.Executable | Should -Contain 'apt.dsc.resource.sh'
    }

    It 'Packages All and only the target platform for <Architecture>' -ForEach @(
        @{ Architecture = 'x86_64-unknown-linux-gnu'; Count = 6; Apt = $true }
        @{ Architecture = 'aarch64-apple-darwin'; Count = 5; Apt = $false }
        @{ Architecture = 'aarch64-pc-windows-msvc'; Count = 17; Apt = $false }
    ) {
        Build-DscResourcePackage -BuildData $buildData -ArtifactDirectory $artifactDirectory -Architecture $Architecture

        @(Get-ChildItem -Path $outputDirectory -Filter '*.zip') | Should -HaveCount $Count
        $entries = @(Get-ZipEntryName (Join-Path $outputDirectory 'Microsoft.System.Process.zip'))
        $entries | Should -HaveCount 1
        $entries | Should -Contain 'process.exe'
        Test-Path (Join-Path $outputDirectory 'DSC.PackageManagement.Apt.zip') | Should -Be $Apt
        Test-Path (Join-Path $outputDirectory 'Microsoft.Windows.DISM.zip') |
            Should -Be ($Architecture -match 'windows')
        Join-Path $outputDirectory 'Microsoft.Adapter.WindowsPowerShell.zip' | Should -Not -Exist
    }

    It 'Merges zipContents, deduplicates shared files, and excludes existing manifests' {
        $buildData.ResourcePackageFiles.All['Microsoft.OpenSSH.SSHD'].zipContents += 'nested\extra.ps1'
        $null = New-Item -Path (Join-Path $bin 'nested') -ItemType Directory
        Set-Content -LiteralPath (Join-Path $bin 'nested\extra.ps1') -Value 'extra'
        Set-Content -LiteralPath (Join-Path $bin 'sshd-windows.dsc.resource.json') -Value '{}'

        Build-DscResourcePackage -BuildData $buildData -ArtifactDirectory $artifactDirectory -Architecture x86_64-pc-windows-msvc

        $sshd = @(Get-ZipEntryName (Join-Path $outputDirectory 'Microsoft.OpenSSH.SSHD.zip'))
        $sshd | Should -HaveCount 2
        $sshd | Should -Contain 'sshdconfig.exe'
        $sshd | Should -Contain 'nested/extra.ps1'
        $dism = @(Get-ZipEntryName (Join-Path $outputDirectory 'Microsoft.Windows.DISM.zip'))
        $dism | Should -HaveCount 1
        $dism | Should -Contain 'dism_dsc.exe'
        $wmi = @(Get-ZipEntryName (Join-Path $outputDirectory 'Microsoft.Windows.WMI.zip'))
        $wmi | Should -HaveCount 3
        $wmi | Should -Contain 'wmi.resource.ps1'
        $wmi | Should -Contain 'wmiAdapter.psd1'
        $wmi | Should -Contain 'wmiAdapter.psm1'
    }

    It 'Uses the current platform and artifact directory when no target is specified' {
        Mock Get-ArtifactDirectoryPath -ModuleName helpers.build { @{ Bin = $bin } }

        Build-DscResourcePackage -BuildData $buildData

        Get-ZipEntryName (Join-Path $outputDirectory 'Microsoft.System.Process.zip') | Should -Contain 'process.exe'
        Should -Invoke Get-ArtifactDirectoryPath -ModuleName helpers.build -Exactly 1 -ParameterFilter {
            $Architecture -eq 'current' -and -not $Release
        }
    }

    It 'Fails rather than producing a partial package when a file is missing' {
        Remove-Item -LiteralPath (Join-Path $bin 'process.exe')

        {
            Build-DscResourcePackage -BuildData $buildData -ArtifactDirectory $artifactDirectory -Architecture x86_64-unknown-linux-gnu
        } | Should -Throw '*process*'
        Join-Path $outputDirectory 'Microsoft.System.Process.zip' | Should -Not -Exist
    }

    It 'Creates release packages through the build switch without invoking compilation' {
        $fixtureRoot = Join-Path $TestDrive ([guid]::NewGuid().ToString())
        $null = New-Item -Path $fixtureRoot -ItemType Directory
        foreach ($file in @('build.ps1', 'helpers.build.psm1', 'data.build.json')) {
            Copy-Item -LiteralPath (Join-Path $PSScriptRoot '..' $file) -Destination $fixtureRoot
        }
        $architecture = 'x86_64-unknown-linux-gnu'
        $target = Join-Path $fixtureRoot 'bin' $architecture 'release'
        $null = New-Item -Path (Split-Path -Parent $target) -ItemType Directory -Force
        Copy-Item -LiteralPath $bin -Destination $target -Recurse
        $pwsh = (Get-Process -Id $PID).Path

        Push-Location $fixtureRoot
        try {
            $output = & $pwsh -NoProfile -File (Join-Path $fixtureRoot 'build.ps1') `
                -CreateResourcePackages -Architecture $architecture -Release 2>&1
            $exitCode = $LASTEXITCODE
        } finally {
            Pop-Location
        }

        $exitCode | Should -Be 0 -Because ($output -join "`n")
        $packages = Join-Path $target 'resource-packages'
        @(Get-ChildItem -Path $packages -Filter '*.zip') | Should -HaveCount 6
        Get-ZipEntryName (Join-Path $packages 'Microsoft.System.Process.zip') | Should -Contain 'process.exe'
        $output -join "`n" | Should -Not -Match 'Compiling Rust|Configuring Rust environment'
    }

    It 'Excludes test-only project files from package definitions' {
        $testProjects = (Import-DscBuildData -RefreshProjects).Projects | Where-Object TestOnly
        $packageFiles = foreach ($section in @('All', 'Linux', 'macOS', 'Windows')) {
            foreach ($package in $buildData.ResourcePackageFiles.$section.Values) {
                $package.zipContents
                $package.manifests
            }
        }
        foreach ($project in $testProjects) {
            foreach ($binary in $project.Binaries) {
                $packageFiles | Should -Not -Contain $binary
                $packageFiles | Should -Not -Contain "$binary.exe"
            }
            foreach ($section in @('All', 'Linux', 'macOS', 'Windows')) {
                foreach ($file in $project.CopyFiles.$section) {
                    $packageFiles | Should -Not -Contain $file
                }
            }
        }
    }
}
