# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

BeforeDiscovery {
    Import-Module (Join-Path $PSScriptRoot '..' 'helpers.build.psm1') -Force
}

Describe 'Individual OCI manifest inputs' {
    BeforeAll {
        $repositoryRoot = Join-Path $PSScriptRoot '..'
        Import-Module (Join-Path $repositoryRoot 'helpers.build.psm1') -Force
        $buildData = Import-DscBuildData -RefreshProjects
        $artifactRoot = Join-Path $TestDrive 'individual-manifests'
        foreach ($project in ($buildData.Projects | Where-Object { -not $_.TestOnly })) {
            foreach ($section in @('All', 'Linux', 'macOS', 'Windows')) {
                foreach ($file in $project.CopyFiles.$section) {
                    if ($file -like '*.dsc.manifests.json') {
                        Export-DscResourceManifest -Path (Join-Path $repositoryRoot $project.RelativePath $file) `
                            -Destination (Join-Path $artifactRoot 'resource-manifests')
                    }
                }
            }
        }
    }

    It 'Uses disjoint generated resource identities for <TargetOS>' -ForEach @(
        @{ TargetOS = 'Linux' }, @{ TargetOS = 'macOS' }, @{ TargetOS = 'Windows' }
    ) {
        $owners = @{}
        foreach ($section in @('All', $TargetOS)) {
            foreach ($package in $buildData.ResourcePackageFiles.$section.GetEnumerator()) {
                foreach ($file in $package.Value.manifests) {
                    $file | Should -Not -BeLike '*.dsc.manifests.json'
                    if ($file -notlike 'resource-manifests/*') {
                        continue
                    }
                    $resource = Get-Content -LiteralPath (Join-Path $artifactRoot $file) -Raw | ConvertFrom-Json
                    if ($owners.ContainsKey($resource.type)) {
                        $owners[$resource.type] | Should -Be $package.Key
                    }
                    $owners[$resource.type] = $package.Key
                }
            }
        }
        $owners.Keys | Should -Contain 'Microsoft.Filesystem.File/Content'
        if ($TargetOS -eq 'Windows') {
            $owners.Keys | Should -Contain 'Microsoft.Windows.Filesystem/AccessControlList'
        } else {
            $owners.Keys | Should -Not -Contain 'Microsoft.Windows.Filesystem/AccessControlList'
        }
    }
}

Describe 'Copy-BuildArtifact manifest migration' {
    InModuleScope helpers.build {
        BeforeEach {
            $caseDirectory = Join-Path $TestDrive ([guid]::NewGuid().ToString())
            $sourceDirectory = New-Item -Path (Join-Path $caseDirectory 'source') -ItemType Directory -Force
            $outputDirectory = New-Item -Path (Join-Path $caseDirectory 'output') -ItemType Directory -Force
            $project = [DscProjectDefinition]@{
                Name = 'manifest-migration'
                RelativePath = '.'
                CopyFiles = [DscProjectCopyFiles]@{
                    All = @('service.dsc.manifests.json')
                }
            }
            Mock Get-ArtifactDirectoryPath {
                @{ Bin = $outputDirectory.FullName }
            }
            Mock Test-ShouldSkipProject { $false }
            Mock Test-ShouldSkipFileForArchitecture { $false }
            Mock Resolve-Path { Join-Path $sourceDirectory 'service.dsc.manifests.json' }
            Set-Content -LiteralPath (Join-Path $sourceDirectory 'service.dsc.manifests.json') -Value '{"resources":[]}'
            Set-Content -LiteralPath (Join-Path $outputDirectory 'service.dsc.resource.json') -Value 'obsolete'
            Set-Content -LiteralPath (Join-Path $outputDirectory 'unrelated.dsc.resource.json') -Value 'unrelated'
        }

        It 'Removes the obsolete standalone manifest after copying its <Platform> replacement' -ForEach @(
            @{ Platform = 'All' }
            @{ Platform = 'Windows' }
        ) {
            $project.CopyFiles = [DscProjectCopyFiles]@{}
            $project.CopyFiles.$Platform = @('service.dsc.manifests.json')

            Copy-BuildArtifact -Project $project

            Join-Path $outputDirectory 'service.dsc.manifests.json' | Should -FileContentMatch '"resources"'
            Join-Path $outputDirectory 'service.dsc.resource.json' | Should -Not -Exist
            Join-Path $outputDirectory 'unrelated.dsc.resource.json' | Should -Exist
        }

        It 'Preserves a standalone manifest that still exists in the source project' {
            Set-Content -LiteralPath (Join-Path $sourceDirectory 'service.dsc.resource.json') -Value 'still used'

            Copy-BuildArtifact -Project $project

            Join-Path $outputDirectory 'service.dsc.resource.json' | Should -Exist
        }

        It 'Generates separate publishing manifests while preserving the distribution bundle and ignoring actions' {
            $bundle = @{
                resources = @(
                    @{
                        '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.json'
                        type = 'Microsoft.Test/Service'
                        version = '1.0.0'
                        author = 'Existing author'
                        get = @{ executable = 'service' }
                    },
                    @{ type = 'Microsoft.Test.Adapter/Service'; version = '0.1.0' }
                )
                actions = @(@{ type = 'Microsoft.Test.Action/RestartService'; version = '1.0.0' })
            }
            $bundle | ConvertTo-Json -Depth 10 |
                Set-Content -LiteralPath (Join-Path $sourceDirectory 'service.dsc.manifests.json')

            Copy-BuildArtifact -Project $project

            $manifests = Join-Path $outputDirectory 'resource-manifests'
            @(Get-ChildItem -LiteralPath $manifests -File) | Should -HaveCount 2
            $resource = Get-Content -LiteralPath (Join-Path $manifests 'Microsoft.Test.Service.dsc.resource.json') -Raw |
                ConvertFrom-Json
            $resource.author | Should -Be 'Existing author'
            $resource.get.executable | Should -Be 'service'
            $resource.'$schema' | Should -Be 'https://aka.ms/dsc/schemas/v3/resource/manifest.json'
            Join-Path $manifests 'Microsoft.Test.Adapter.Service.dsc.resource.json' | Should -Exist
            Join-Path $outputDirectory 'service.dsc.manifests.json' | Should -FileContentMatch 'RestartService'
        }

        It 'Preserves the old output if copying the replacement fails' {
            Mock Copy-Item { throw 'Copy failed' }

            { Copy-BuildArtifact -Project $project } | Should -Throw '*Copy failed*'

            Join-Path $outputDirectory 'service.dsc.resource.json' | Should -Exist
        }

        It 'Does not copy or remove manifests for an excluded platform' {
            $project.CopyFiles = [DscProjectCopyFiles]@{
                Windows = @('service.dsc.manifests.json')
            }
            Mock Test-ShouldSkipFileForArchitecture { $OperatingSystem -eq 'Windows' }

            Copy-BuildArtifact -Project $project

            Join-Path $outputDirectory 'service.dsc.resource.json' | Should -Exist
            Join-Path $outputDirectory 'service.dsc.manifests.json' | Should -Not -Exist
        }
    }
}
