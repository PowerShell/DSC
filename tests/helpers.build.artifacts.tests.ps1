# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

BeforeDiscovery {
    Import-Module (Join-Path $PSScriptRoot '..' 'helpers.build.psm1') -Force
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
