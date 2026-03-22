# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Import-DscAdaptedResourceManifest' {

    BeforeAll {
        $modulePath = Join-Path (Join-Path $PSScriptRoot '..') 'Microsoft.PowerShell.DSC.psd1'
        Import-Module $modulePath -Force

        $fixturesPath = Join-Path $PSScriptRoot 'Fixtures'
    }

    Context 'Importing a full adapted resource manifest' {

        BeforeAll {
            $jsonPath = Join-Path $fixturesPath 'SimpleResource.dsc.adaptedResource.json'
            $result = Import-DscAdaptedResourceManifest -Path $jsonPath
        }

        It 'Returns a DscAdaptedResourceManifest object' {
            $result.GetType().Name | Should -BeExactly 'DscAdaptedResourceManifest'
        }

        It 'Imports the schema URI' {
            $result.Schema | Should -BeExactly 'https://aka.ms/dsc/schemas/v3/bundled/adaptedresource/manifest.json'
        }

        It 'Imports the type' {
            $result.Type | Should -BeExactly 'SimpleResource/SimpleResource'
        }

        It 'Imports the kind' {
            $result.Kind | Should -BeExactly 'resource'
        }

        It 'Imports the version' {
            $result.Version | Should -BeExactly '1.0.0'
        }

        It 'Imports capabilities as an array' {
            $result.Capabilities | Should -HaveCount 3
            $result.Capabilities | Should -Contain 'get'
            $result.Capabilities | Should -Contain 'set'
            $result.Capabilities | Should -Contain 'test'
        }

        It 'Imports the description' {
            $result.Description | Should -BeExactly 'A simple DSC resource for testing.'
        }

        It 'Imports the author' {
            $result.Author | Should -BeExactly 'Microsoft'
        }

        It 'Imports the requireAdapter' {
            $result.RequireAdapter | Should -BeExactly 'Microsoft.Adapter/PowerShell'
        }

        It 'Imports the path' {
            $result.Path | Should -BeExactly 'SimpleResource/SimpleResource.psd1'
        }

        It 'Imports the embedded schema' {
            $result.ManifestSchema | Should -Not -BeNullOrEmpty
            $result.ManifestSchema.Embedded | Should -Not -BeNullOrEmpty
        }

        It 'Has correct schema properties' {
            $result.ManifestSchema.Embedded['properties'] | Should -Not -BeNullOrEmpty
            $result.ManifestSchema.Embedded['properties']['Name'] | Should -Not -BeNullOrEmpty
            $result.ManifestSchema.Embedded['properties']['Value'] | Should -Not -BeNullOrEmpty
        }

        It 'Has correct required fields in embedded schema' {
            $result.ManifestSchema.Embedded['required'] | Should -Contain 'Name'
        }
    }

    Context 'Importing a minimal adapted resource manifest without optional fields' {

        BeforeAll {
            $jsonPath = Join-Path $fixturesPath 'MinimalResource.dsc.adaptedResource.json'
            $result = Import-DscAdaptedResourceManifest -Path $jsonPath
        }

        It 'Returns a DscAdaptedResourceManifest object' {
            $result.GetType().Name | Should -BeExactly 'DscAdaptedResourceManifest'
        }

        It 'Imports the type' {
            $result.Type | Should -BeExactly 'TestModule/MinimalResource'
        }

        It 'Defaults kind to resource when missing' {
            $result.Kind | Should -BeExactly 'resource'
        }

        It 'Defaults capabilities to empty array when missing' {
            $result.Capabilities.Count | Should -Be 0
        }

        It 'Defaults description to empty string when missing' {
            $result.Description | Should -BeExactly ''
        }

        It 'Defaults author to empty string when missing' {
            $result.Author | Should -BeExactly ''
        }

        It 'Defaults path to empty string when missing' {
            $result.Path | Should -BeExactly ''
        }

        It 'Handles schema without embedded wrapper' {
            $result.ManifestSchema | Should -Not -BeNullOrEmpty
            $result.ManifestSchema.Embedded | Should -Not -BeNullOrEmpty
            $result.ManifestSchema.Embedded['properties']['Id'] | Should -Not -BeNullOrEmpty
        }
    }

    Context 'Pipeline input' {

        It 'Accepts paths from the pipeline' {
            $jsonPath = Join-Path $fixturesPath 'SimpleResource.dsc.adaptedResource.json'
            $result = $jsonPath | Import-DscAdaptedResourceManifest
            $result.Type | Should -BeExactly 'SimpleResource/SimpleResource'
        }

        It 'Accepts FileInfo objects from the pipeline' {
            $jsonPath = Join-Path $fixturesPath 'SimpleResource.dsc.adaptedResource.json'
            $result = Get-Item $jsonPath | Import-DscAdaptedResourceManifest
            $result.Type | Should -BeExactly 'SimpleResource/SimpleResource'
        }

        It 'Processes multiple files from the pipeline' {
            $files = @(
                (Join-Path $fixturesPath 'SimpleResource.dsc.adaptedResource.json')
                (Join-Path $fixturesPath 'MinimalResource.dsc.adaptedResource.json')
            )
            $results = $files | Import-DscAdaptedResourceManifest
            $results | Should -HaveCount 2
            $results[0].Type | Should -BeExactly 'SimpleResource/SimpleResource'
            $results[1].Type | Should -BeExactly 'TestModule/MinimalResource'
        }
    }

    Context 'Round-trip fidelity' {

        It 'Produces identical JSON after import and re-export' {
            $jsonPath = Join-Path $fixturesPath 'SimpleResource.dsc.adaptedResource.json'
            $original = Get-Content -LiteralPath $jsonPath -Raw | ConvertFrom-Json
            $imported = Import-DscAdaptedResourceManifest -Path $jsonPath
            $reExported = $imported.ToJson() | ConvertFrom-Json

            $reExported.type | Should -BeExactly $original.type
            $reExported.kind | Should -BeExactly $original.kind
            $reExported.version | Should -BeExactly $original.version
            $reExported.requireAdapter | Should -BeExactly $original.requireAdapter
            $reExported.path | Should -BeExactly $original.path
            $reExported.author | Should -BeExactly $original.author
            $reExported.description | Should -BeExactly $original.description
        }
    }

    Context 'ToHashtable round-trip' {

        It 'Converts imported manifest to hashtable correctly' {
            $jsonPath = Join-Path $fixturesPath 'SimpleResource.dsc.adaptedResource.json'
            $imported = Import-DscAdaptedResourceManifest -Path $jsonPath
            $ht = $imported.ToHashtable()

            $ht['type'] | Should -BeExactly 'SimpleResource/SimpleResource'
            $ht['version'] | Should -BeExactly '1.0.0'
            $ht['requireAdapter'] | Should -BeExactly 'Microsoft.Adapter/PowerShell'
            $ht['path'] | Should -BeExactly 'SimpleResource/SimpleResource.psd1'
            $ht['schema']['embedded'] | Should -Not -BeNullOrEmpty
        }
    }

    Context 'Error handling' {

        It 'Throws when the path does not exist' {
            { Import-DscAdaptedResourceManifest -Path 'nonexistent.json' } | Should -Throw '*does not exist*'
        }
    }

    Context 'Integration with New-DscResourceManifest' {

        It 'Imported manifests can be added to a resource manifest list' {
            $jsonPath = Join-Path $fixturesPath 'SimpleResource.dsc.adaptedResource.json'
            $imported = Import-DscAdaptedResourceManifest -Path $jsonPath

            $list = $imported | New-DscResourceManifest
            $list.AdaptedResources | Should -HaveCount 1
            $list.AdaptedResources[0]['type'] | Should -BeExactly 'SimpleResource/SimpleResource'
        }
    }
}
