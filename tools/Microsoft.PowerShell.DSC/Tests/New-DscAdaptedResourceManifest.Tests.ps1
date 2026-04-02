# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'New-DscAdaptedResourceManifest' {

    BeforeAll {
        $modulePath = Join-Path (Join-Path $PSScriptRoot '..') 'Microsoft.PowerShell.DSC.psd1'
        Import-Module $modulePath -Force

        $fixturesPath = Join-Path $PSScriptRoot 'Fixtures'
    }

    Context 'Simple module with a single DSC resource' {

        BeforeAll {
            $psd1 = Join-Path $fixturesPath 'SimpleResource' 'SimpleResource.psd1'
            $result = New-DscAdaptedResourceManifest -Path $psd1
        }

        It 'Returns exactly one manifest object' {
            $result | Should -HaveCount 1
        }

        It 'Returns a DscAdaptedResourceManifest object' {
            $result.GetType().Name | Should -BeExactly 'DscAdaptedResourceManifest'
        }

        It 'Sets the correct resource type' {
            $result.Type | Should -BeExactly 'SimpleResource/SimpleResource'
        }

        It 'Sets the kind to resource' {
            $result.Kind | Should -BeExactly 'resource'
        }

        It 'Sets the version from the module manifest' {
            $result.Version | Should -BeExactly '1.0.0'
        }

        It 'Sets the description from the module manifest' {
            $result.Description | Should -BeExactly 'A simple DSC resource for testing.'
        }

        It 'Sets the author from the module manifest' {
            $result.Author | Should -BeExactly 'Microsoft'
        }

        It 'Sets the schema URI' {
            $result.Schema | Should -BeExactly 'https://aka.ms/dsc/schemas/v3/bundled/adaptedresource/manifest.json'
        }

        It 'Sets the require adapter to Microsoft.DSC/PowerShell' {
            $result.RequireAdapter | Should -BeExactly 'Microsoft.Adapter/PowerShell'
        }

        It 'Sets the path to the psd1 relative path' {
            $result.Path | Should -BeLike '*SimpleResource*'
        }

        It 'Detects get, set, and test capabilities' {
            $result.Capabilities | Should -Contain 'get'
            $result.Capabilities | Should -Contain 'set'
            $result.Capabilities | Should -Contain 'test'
        }

        It 'Does not include capabilities for methods that do not exist' {
            $result.Capabilities | Should -Not -Contain 'delete'
            $result.Capabilities | Should -Not -Contain 'export'
            $result.Capabilities | Should -Not -Contain 'whatIf'
        }

        It 'Includes an embedded JSON schema' {
            $result.ManifestSchema | Should -Not -BeNullOrEmpty
            $result.ManifestSchema.Embedded | Should -Not -BeNullOrEmpty
        }

        It 'Schema has correct $schema URI' {
            $result.ManifestSchema.Embedded['$schema'] | Should -BeExactly 'https://json-schema.org/draft/2020-12/schema'
        }

        It 'Schema has type set to object' {
            $result.ManifestSchema.Embedded['type'] | Should -BeExactly 'object'
        }

        It 'Schema includes Key property as required' {
            $result.ManifestSchema.Embedded['required'] | Should -Contain 'Name'
        }

        It 'Schema includes Mandatory property as required' {
            $result.ManifestSchema.Embedded['required'] | Should -Contain 'Value'
        }

        It 'Schema maps string properties correctly' {
            $result.ManifestSchema.Embedded['properties']['Name']['type'] | Should -BeExactly 'string'
            $result.ManifestSchema.Embedded['properties']['Value']['type'] | Should -BeExactly 'string'
        }

        It 'Schema maps bool properties correctly' {
            $result.ManifestSchema.Embedded['properties']['Enabled']['type'] | Should -BeExactly 'boolean'
        }
    }

    Context 'Module with multiple DSC resources, inheritance, and enums' {

        BeforeAll {
            $psd1 = Join-Path $fixturesPath 'MultiResource' 'MultiResource.psd1'
            $results = @(New-DscAdaptedResourceManifest -Path $psd1)
        }

        It 'Returns two manifest objects' {
            $results | Should -HaveCount 2
        }

        It 'Returns manifests for ResourceA and ResourceB' {
            $results.Type | Should -Contain 'MultiResource/ResourceA'
            $results.Type | Should -Contain 'MultiResource/ResourceB'
        }

        It 'All manifests share the same module version' {
            $results | ForEach-Object {
                $_.Version | Should -BeExactly '2.5.0'
            }
        }

        It 'All manifests share the same author' {
            $results | ForEach-Object {
                $_.Author | Should -BeExactly 'Microsoft'
            }
        }

        Context 'ResourceA - inheritance, enums, delete, export' {

            BeforeAll {
                $resourceA = $results | Where-Object { $_.Type -eq 'MultiResource/ResourceA' }
            }

            It 'Detects get, set, test, delete, and export capabilities' {
                $resourceA.Capabilities | Should -Contain 'get'
                $resourceA.Capabilities | Should -Contain 'set'
                $resourceA.Capabilities | Should -Contain 'test'
                $resourceA.Capabilities | Should -Contain 'delete'
                $resourceA.Capabilities | Should -Contain 'export'
            }

            It 'Includes inherited BaseProperty from base class' {
                $resourceA.ManifestSchema.Embedded['properties'].Keys | Should -Contain 'BaseProperty'
            }

            It 'Includes own properties' {
                $props = $resourceA.ManifestSchema.Embedded['properties']
                $props.Keys | Should -Contain 'Name'
                $props.Keys | Should -Contain 'Ensure'
                $props.Keys | Should -Contain 'Count'
                $props.Keys | Should -Contain 'Tags'
            }

            It 'Maps the Ensure enum to string type with enum values' {
                $ensureProp = $resourceA.ManifestSchema.Embedded['properties']['Ensure']
                $ensureProp['type'] | Should -BeExactly 'string'
                $ensureProp['enum'] | Should -Contain 'Present'
                $ensureProp['enum'] | Should -Contain 'Absent'
            }

            It 'Maps int property to integer type' {
                $resourceA.ManifestSchema.Embedded['properties']['Count']['type'] | Should -BeExactly 'integer'
            }

            It 'Maps string[] property to array type with string items' {
                $tagsProp = $resourceA.ManifestSchema.Embedded['properties']['Tags']
                $tagsProp['type'] | Should -BeExactly 'array'
                $tagsProp['items']['type'] | Should -BeExactly 'string'
            }

            It 'Has Key property Name as required' {
                $resourceA.ManifestSchema.Embedded['required'] | Should -Contain 'Name'
            }
        }

        Context 'ResourceB - whatIf capability and hashtable property' {

            BeforeAll {
                $resourceB = $results | Where-Object { $_.Type -eq 'MultiResource/ResourceB' }
            }

            It 'Detects get, set, test, and whatIf capabilities' {
                $resourceB.Capabilities | Should -Contain 'get'
                $resourceB.Capabilities | Should -Contain 'set'
                $resourceB.Capabilities | Should -Contain 'test'
                $resourceB.Capabilities | Should -Contain 'whatIf'
            }

            It 'Does not include delete or export capabilities' {
                $resourceB.Capabilities | Should -Not -Contain 'delete'
                $resourceB.Capabilities | Should -Not -Contain 'export'
            }

            It 'Maps hashtable property to object type' {
                $resourceB.ManifestSchema.Embedded['properties']['Settings']['type'] | Should -BeExactly 'object'
            }
        }
    }

    Context 'Standalone .ps1 file with a DSC resource' {

        BeforeAll {
            $ps1Path = Join-Path $fixturesPath 'StandaloneResource.ps1'
            $result = New-DscAdaptedResourceManifest -Path $ps1Path
        }

        It 'Returns a manifest object' {
            $result | Should -HaveCount 1
        }

        It 'Uses the file name as the module name' {
            $result.Type | Should -BeExactly 'StandaloneResource/StandaloneResource'
        }

        It 'Defaults version to 0.0.1 when no psd1 exists' {
            $result.Version | Should -BeExactly '0.0.1'
        }

        It 'Defaults author to empty string when no psd1 exists' {
            $result.Author | Should -BeExactly ''
        }

        It 'Sets path to the actual script file' {
            $result.Path | Should -BeExactly 'StandaloneResource.ps1'
        }
    }

    Context 'File with no DSC resources' {

        It 'Emits a warning and returns nothing' {
            $psm1Path = Join-Path $fixturesPath 'NoDscResource.psm1'
            $result = New-DscAdaptedResourceManifest -Path $psm1Path -WarningVariable warn -WarningAction SilentlyContinue
            $result | Should -BeNullOrEmpty
            $warn | Should -Not -BeNullOrEmpty
            $warn[0] | Should -BeLike '*No class-based DSC resources found*'
        }
    }

    Context 'ToJson serialization' {

        BeforeAll {
            $psd1 = Join-Path $fixturesPath 'SimpleResource' 'SimpleResource.psd1'
            $manifest = New-DscAdaptedResourceManifest -Path $psd1
            $json = $manifest.ToJson()
            $parsed = $json | ConvertFrom-Json
        }

        It 'Produces valid JSON' {
            { $json | ConvertFrom-Json } | Should -Not -Throw
        }

        It 'Contains the $schema key' {
            $parsed.'$schema' | Should -BeExactly 'https://aka.ms/dsc/schemas/v3/bundled/adaptedresource/manifest.json'
        }

        It 'Contains the type key' {
            $parsed.type | Should -BeExactly 'SimpleResource/SimpleResource'
        }

        It 'Contains the kind key' {
            $parsed.kind | Should -BeExactly 'resource'
        }

        It 'Contains the version key' {
            $parsed.version | Should -BeExactly '1.0.0'
        }

        It 'Contains the requireAdapter key' {
            $parsed.requireAdapter | Should -BeExactly 'Microsoft.Adapter/PowerShell'
        }

        It 'Contains the schema.embedded object with properties' {
            $parsed.schema.embedded | Should -Not -BeNullOrEmpty
            $parsed.schema.embedded.properties | Should -Not -BeNullOrEmpty
        }
    }

    Context 'Pipeline input' {

        It 'Accepts Path from pipeline by value' {
            $psd1 = Join-Path $fixturesPath 'SimpleResource' 'SimpleResource.psd1'
            $result = $psd1 | New-DscAdaptedResourceManifest
            $result | Should -HaveCount 1
            $result.Type | Should -BeExactly 'SimpleResource/SimpleResource'
        }

        It 'Accepts multiple paths from pipeline' {
            $paths = @(
                (Join-Path $fixturesPath 'SimpleResource' 'SimpleResource.psd1')
                (Join-Path $fixturesPath 'MultiResource' 'MultiResource.psd1')
            )
            $results = $paths | New-DscAdaptedResourceManifest
            $results | Should -HaveCount 3  # 1 from Simple + 2 from Multi
        }

        It 'Accepts FileInfo objects from Get-ChildItem via pipeline' {
            $results = Get-ChildItem -Path $fixturesPath -Filter '*.psd1' -Recurse | New-DscAdaptedResourceManifest
            $results | Should -HaveCount 9 
        }
    }

    Context 'Input via .psm1 path resolves co-located .psd1' {

        It 'Uses psd1 metadata when psm1 is provided and psd1 exists' {
            $psm1Path = Join-Path $fixturesPath 'SimpleResource' 'SimpleResource.psm1'
            $result = New-DscAdaptedResourceManifest -Path $psm1Path
            $result.Version | Should -BeExactly '1.0.0'
            $result.Author | Should -BeExactly 'Microsoft'
        }
    }

    Context 'Parameter validation' {

        It 'Throws when path does not exist' {
            { New-DscAdaptedResourceManifest -Path 'C:\NonExistent\Fake.psd1' } | Should -Throw '*does not exist*'
        }

        It 'Throws when path has an unsupported extension' {
            $txtFile = Join-Path $TestDrive 'test.txt'
            Set-Content -Path $txtFile -Value 'not a ps file'
            { New-DscAdaptedResourceManifest -Path $txtFile } | Should -Throw '*must be a .ps1, .psm1, or .psd1 file*'
        }

        It 'Is a mandatory parameter' {
            (Get-Command New-DscAdaptedResourceManifest).Parameters['Path'].Attributes |
                Where-Object { $_ -is [System.Management.Automation.ParameterAttribute] } |
                ForEach-Object { $_.Mandatory | Should -BeTrue }
        }
    }

    Context 'Schema additionalProperties' {

        It 'Sets additionalProperties to false' {
            $psd1 = Join-Path $fixturesPath 'SimpleResource' 'SimpleResource.psd1'
            $result = New-DscAdaptedResourceManifest -Path $psd1
            $result.ManifestSchema.Embedded['additionalProperties'] | Should -BeFalse
        }
    }
}
