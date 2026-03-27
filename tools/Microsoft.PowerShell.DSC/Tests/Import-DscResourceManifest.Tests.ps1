# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Import-DscResourceManifest' {

    BeforeAll {
        $modulePath = Join-Path (Join-Path $PSScriptRoot '..') 'Microsoft.PowerShell.DSC.psd1'
        Import-Module $modulePath -Force

        $fixturesPath = Join-Path $PSScriptRoot 'Fixtures'
    }

    Context 'Importing a manifest list with all sections' {

        BeforeAll {
            $jsonPath = Join-Path $fixturesPath 'TestModule.dsc.manifests.json'
            $result = Import-DscResourceManifest -Path $jsonPath
        }

        It 'Returns a DscResourceManifestList object' {
            $result.GetType().Name | Should -BeExactly 'DscResourceManifestList'
        }

        It 'Imports two adapted resources' {
            $result.AdaptedResources | Should -HaveCount 2
        }

        It 'Imports the first adapted resource type' {
            $result.AdaptedResources[0]['type'] | Should -BeExactly 'TestModule/ResourceOne'
        }

        It 'Imports the second adapted resource type' {
            $result.AdaptedResources[1]['type'] | Should -BeExactly 'TestModule/ResourceTwo'
        }

        It 'Imports adapted resource capabilities' {
            $result.AdaptedResources[0]['capabilities'] | Should -Contain 'get'
            $result.AdaptedResources[0]['capabilities'] | Should -Contain 'set'
        }

        It 'Imports adapted resource version' {
            $result.AdaptedResources[0]['version'] | Should -BeExactly '1.0.0'
        }

        It 'Imports adapted resource requireAdapter' {
            $result.AdaptedResources[0]['requireAdapter'] | Should -BeExactly 'Microsoft.Adapter/PowerShell'
        }

        It 'Imports adapted resource schema with embedded key' {
            $result.AdaptedResources[0]['schema'] | Should -Not -BeNullOrEmpty
            $result.AdaptedResources[0]['schema']['embedded'] | Should -Not -BeNullOrEmpty
        }

        It 'Imports one command-based resource' {
            $result.Resources | Should -HaveCount 1
        }

        It 'Imports the resource type' {
            $result.Resources[0]['type'] | Should -BeExactly 'Test/CommandResource'
        }

        It 'Imports the resource version' {
            $result.Resources[0]['version'] | Should -BeExactly '0.1.0'
        }

        It 'Imports the resource get command' {
            $result.Resources[0]['get'] | Should -Not -BeNullOrEmpty
            $result.Resources[0]['get']['executable'] | Should -BeExactly 'testcmd'
        }

        It 'Imports one extension' {
            $result.Extensions | Should -HaveCount 1
        }

        It 'Imports the extension type' {
            $result.Extensions[0]['type'] | Should -BeExactly 'Test/Extension'
        }

        It 'Imports the extension discover command' {
            $result.Extensions[0]['discover'] | Should -Not -BeNullOrEmpty
            $result.Extensions[0]['discover']['executable'] | Should -BeExactly 'testcmd'
        }
    }

    Context 'Importing a manifest list with only adapted resources' {

        BeforeAll {
            $json = @{
                adaptedResources = @(
                    @{
                        '$schema'      = 'https://aka.ms/dsc/schemas/v3/bundled/adaptedresource/manifest.json'
                        type           = 'OnlyAdapted/Resource'
                        version        = '2.0.0'
                        requireAdapter = 'Microsoft.Adapter/PowerShell'
                        schema         = @{
                            embedded = @{
                                type       = 'object'
                                properties = @{}
                            }
                        }
                    }
                )
            } | ConvertTo-Json -Depth 10

            $tempFile = Join-Path $TestDrive 'adapted-only.dsc.manifests.json'
            $json | Set-Content -LiteralPath $tempFile -Encoding utf8
            $result = Import-DscResourceManifest -Path $tempFile
        }

        It 'Imports the adapted resource' {
            $result.AdaptedResources | Should -HaveCount 1
            $result.AdaptedResources[0]['type'] | Should -BeExactly 'OnlyAdapted/Resource'
        }

        It 'Has empty resources list' {
            $result.Resources | Should -HaveCount 0
        }

        It 'Has empty extensions list' {
            $result.Extensions | Should -HaveCount 0
        }
    }

    Context 'Importing a manifest list with only resources' {

        BeforeAll {
            $json = @{
                resources = @(
                    @{
                        '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.json'
                        type      = 'OnlyCommand/Resource'
                        version   = '0.5.0'
                        get       = @{
                            executable = 'mycmd'
                            args       = @('get')
                        }
                    }
                )
            } | ConvertTo-Json -Depth 10

            $tempFile = Join-Path $TestDrive 'resources-only.dsc.manifests.json'
            $json | Set-Content -LiteralPath $tempFile -Encoding utf8
            $result = Import-DscResourceManifest -Path $tempFile
        }

        It 'Has empty adapted resources list' {
            $result.AdaptedResources | Should -HaveCount 0
        }

        It 'Imports the resource' {
            $result.Resources | Should -HaveCount 1
            $result.Resources[0]['type'] | Should -BeExactly 'OnlyCommand/Resource'
        }

        It 'Has empty extensions list' {
            $result.Extensions | Should -HaveCount 0
        }
    }

    Context 'Pipeline input' {

        It 'Accepts paths from the pipeline' {
            $jsonPath = Join-Path $fixturesPath 'TestModule.dsc.manifests.json'
            $result = $jsonPath | Import-DscResourceManifest
            $result.AdaptedResources | Should -HaveCount 2
        }

        It 'Accepts FileInfo objects from the pipeline' {
            $jsonPath = Join-Path $fixturesPath 'TestModule.dsc.manifests.json'
            $result = Get-Item $jsonPath | Import-DscResourceManifest
            $result.AdaptedResources | Should -HaveCount 2
        }
    }

    Context 'Round-trip fidelity' {

        It 'Re-exports JSON that preserves adapted resource types' {
            $jsonPath = Join-Path $fixturesPath 'TestModule.dsc.manifests.json'
            $imported = Import-DscResourceManifest -Path $jsonPath
            $reExported = $imported.ToJson() | ConvertFrom-Json

            $reExported.adaptedResources | Should -HaveCount 2
            $reExported.adaptedResources[0].type | Should -BeExactly 'TestModule/ResourceOne'
            $reExported.adaptedResources[1].type | Should -BeExactly 'TestModule/ResourceTwo'
        }

        It 'Re-exports JSON that preserves resource types' {
            $jsonPath = Join-Path $fixturesPath 'TestModule.dsc.manifests.json'
            $imported = Import-DscResourceManifest -Path $jsonPath
            $reExported = $imported.ToJson() | ConvertFrom-Json

            $reExported.resources | Should -HaveCount 1
            $reExported.resources[0].type | Should -BeExactly 'Test/CommandResource'
        }

        It 'Re-exports JSON that preserves extension types' {
            $jsonPath = Join-Path $fixturesPath 'TestModule.dsc.manifests.json'
            $imported = Import-DscResourceManifest -Path $jsonPath
            $reExported = $imported.ToJson() | ConvertFrom-Json

            $reExported.extensions | Should -HaveCount 1
            $reExported.extensions[0].type | Should -BeExactly 'Test/Extension'
        }
    }

    Context 'Error handling' {

        It 'Throws when the path does not exist' {
            { Import-DscResourceManifest -Path 'nonexistent.json' } | Should -Throw '*does not exist*'
        }
    }

    Context 'Integration with Import-DscAdaptedResourceManifest' {

        It 'Imported adapted manifests can be added to an imported manifest list' {
            $manifestPath = Join-Path $fixturesPath 'TestModule.dsc.manifests.json'
            $adaptedPath = Join-Path $fixturesPath 'SimpleResource.dsc.adaptedResource.json'

            $list = Import-DscResourceManifest -Path $manifestPath
            $adapted = Import-DscAdaptedResourceManifest -Path $adaptedPath
            $list.AddAdaptedResource($adapted)

            $list.AdaptedResources | Should -HaveCount 3
            $list.AdaptedResources[2]['type'] | Should -BeExactly 'SimpleResource/SimpleResource'
        }
    }
}
