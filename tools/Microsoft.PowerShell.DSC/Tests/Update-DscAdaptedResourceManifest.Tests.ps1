# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

using module ..\Microsoft.PowerShell.DSC.psd1

Describe 'Update-DscAdaptedResourceManifest' {

    BeforeAll {
        $fixturesPath = Join-Path $PSScriptRoot 'Fixtures'
    }

    Context 'Override property description' {

        BeforeAll {
            $psd1 = Join-Path $fixturesPath 'SimpleResource' 'SimpleResource.psd1'
            $manifest = New-DscAdaptedResourceManifest -Path $psd1
            $override = [DscPropertyOverride]@{
                Name        = 'Name'
                Description = 'The unique name identifying this resource instance.'
            }
            $result = $manifest | Update-DscAdaptedResourceManifest -PropertyOverride $override
        }

        It 'Returns a DscAdaptedResourceManifest object' {
            $result.GetType().Name | Should -BeExactly 'DscAdaptedResourceManifest'
        }

        It 'Updates the property description' {
            $result.ManifestSchema.Embedded['properties']['Name']['description'] |
                Should -BeExactly 'The unique name identifying this resource instance.'
        }

        It 'Does not modify other properties' {
            $result.ManifestSchema.Embedded['properties']['Value']['description'] |
                Should -BeExactly 'The Value property.'
        }

        It 'Preserves the property type' {
            $result.ManifestSchema.Embedded['properties']['Name']['type'] |
                Should -BeExactly 'string'
        }
    }

    Context 'Override property title' {

        BeforeAll {
            $psd1 = Join-Path $fixturesPath 'SimpleResource' 'SimpleResource.psd1'
            $manifest = New-DscAdaptedResourceManifest -Path $psd1
            $override = [DscPropertyOverride]@{
                Name  = 'Name'
                Title = 'Resource Name'
            }
            $result = $manifest | Update-DscAdaptedResourceManifest -PropertyOverride $override
        }

        It 'Updates the property title' {
            $result.ManifestSchema.Embedded['properties']['Name']['title'] |
                Should -BeExactly 'Resource Name'
        }
    }

    Context 'Add JSON schema keywords' {

        BeforeAll {
            $psd1 = Join-Path $fixturesPath 'MultiResource' 'MultiResource.psd1'
            $manifests = @(New-DscAdaptedResourceManifest -Path $psd1)
            $resourceA = $manifests | Where-Object { $_.Type -eq 'MultiResource/ResourceA' }
            $override = [DscPropertyOverride]@{
                Name       = 'Count'
                JsonSchema = @{ minimum = 0; maximum = 100; default = 1 }
            }
            $result = $resourceA | Update-DscAdaptedResourceManifest -PropertyOverride $override
        }

        It 'Adds the minimum keyword' {
            $result.ManifestSchema.Embedded['properties']['Count']['minimum'] |
                Should -BeExactly 0
        }

        It 'Adds the maximum keyword' {
            $result.ManifestSchema.Embedded['properties']['Count']['maximum'] |
                Should -BeExactly 100
        }

        It 'Adds the default keyword' {
            $result.ManifestSchema.Embedded['properties']['Count']['default'] |
                Should -BeExactly 1
        }

        It 'Preserves the original type' {
            $result.ManifestSchema.Embedded['properties']['Count']['type'] |
                Should -BeExactly 'integer'
        }
    }

    Context 'Replace enum with anyOf using RemoveKeys' {

        BeforeAll {
            $psd1 = Join-Path $fixturesPath 'MultiResource' 'MultiResource.psd1'
            $manifests = @(New-DscAdaptedResourceManifest -Path $psd1)
            $resourceA = $manifests | Where-Object { $_.Type -eq 'MultiResource/ResourceA' }
            $override = [DscPropertyOverride]@{
                Name       = 'Ensure'
                RemoveKeys = @('type', 'enum')
                JsonSchema = @{
                    anyOf = @(
                        @{ type = 'string'; enum = @('Present', 'Absent') }
                        @{ type = 'integer'; minimum = 0; maximum = 1 }
                    )
                }
            }
            $result = $resourceA | Update-DscAdaptedResourceManifest -PropertyOverride $override
        }

        It 'Removes the type key' {
            $result.ManifestSchema.Embedded['properties']['Ensure'].Contains('type') |
                Should -BeFalse
        }

        It 'Removes the enum key' {
            $result.ManifestSchema.Embedded['properties']['Ensure'].Contains('enum') |
                Should -BeFalse
        }

        It 'Adds the anyOf keyword' {
            $result.ManifestSchema.Embedded['properties']['Ensure']['anyOf'] |
                Should -HaveCount 2
        }

        It 'First anyOf option is string with enum' {
            $first = $result.ManifestSchema.Embedded['properties']['Ensure']['anyOf'][0]
            $first['type'] | Should -BeExactly 'string'
            $first['enum'] | Should -Contain 'Present'
            $first['enum'] | Should -Contain 'Absent'
        }

        It 'Second anyOf option is integer with range' {
            $second = $result.ManifestSchema.Embedded['properties']['Ensure']['anyOf'][1]
            $second['type'] | Should -BeExactly 'integer'
            $second['minimum'] | Should -BeExactly 0
            $second['maximum'] | Should -BeExactly 1
        }

        It 'Preserves the title and description' {
            $result.ManifestSchema.Embedded['properties']['Ensure']['title'] |
                Should -BeExactly 'Ensure'
            $result.ManifestSchema.Embedded['properties']['Ensure']['description'] |
                Should -Not -BeNullOrEmpty
        }
    }

    Context 'Override required status' {

        It 'Adds a property to the required list' {
            $psd1 = Join-Path $fixturesPath 'SimpleResource' 'SimpleResource.psd1'
            $manifest = New-DscAdaptedResourceManifest -Path $psd1
            $override = [DscPropertyOverride]@{
                Name     = 'Enabled'
                Required = $true
            }
            $result = $manifest | Update-DscAdaptedResourceManifest -PropertyOverride $override
            $result.ManifestSchema.Embedded['required'] | Should -Contain 'Enabled'
        }

        It 'Removes a property from the required list' {
            $psd1 = Join-Path $fixturesPath 'SimpleResource' 'SimpleResource.psd1'
            $manifest = New-DscAdaptedResourceManifest -Path $psd1
            $override = [DscPropertyOverride]@{
                Name     = 'Name'
                Required = $false
            }
            $result = $manifest | Update-DscAdaptedResourceManifest -PropertyOverride $override
            $result.ManifestSchema.Embedded['required'] | Should -Not -Contain 'Name'
        }

        It 'Does not duplicate a property already in the required list' {
            $psd1 = Join-Path $fixturesPath 'SimpleResource' 'SimpleResource.psd1'
            $manifest = New-DscAdaptedResourceManifest -Path $psd1
            $override = [DscPropertyOverride]@{
                Name     = 'Name'
                Required = $true
            }
            $result = $manifest | Update-DscAdaptedResourceManifest -PropertyOverride $override
            $count = ($result.ManifestSchema.Embedded['required'] | Where-Object { $_ -eq 'Name' }).Count
            $count | Should -BeExactly 1
        }
    }

    Context 'Override resource-level description' {

        BeforeAll {
            $psd1 = Join-Path $fixturesPath 'SimpleResource' 'SimpleResource.psd1'
            $manifest = New-DscAdaptedResourceManifest -Path $psd1
            $result = $manifest | Update-DscAdaptedResourceManifest -Description 'A custom resource description.'
        }

        It 'Updates the manifest description' {
            $result.Description | Should -BeExactly 'A custom resource description.'
        }

        It 'Updates the embedded schema description' {
            $result.ManifestSchema.Embedded['description'] |
                Should -BeExactly 'A custom resource description.'
        }
    }

    Context 'Multiple property overrides' {

        BeforeAll {
            $psd1 = Join-Path $fixturesPath 'SimpleResource' 'SimpleResource.psd1'
            $manifest = New-DscAdaptedResourceManifest -Path $psd1
            $overrides = @(
                [DscPropertyOverride]@{
                    Name        = 'Name'
                    Description = 'The unique resource identifier.'
                }
                [DscPropertyOverride]@{
                    Name        = 'Value'
                    Description = 'The configuration value to apply.'
                    JsonSchema  = @{ default = '' }
                }
                [DscPropertyOverride]@{
                    Name        = 'Enabled'
                    Description = 'Whether this resource instance is active.'
                    Required    = $true
                }
            )
            $result = $manifest | Update-DscAdaptedResourceManifest -PropertyOverride $overrides
        }

        It 'Updates Name description' {
            $result.ManifestSchema.Embedded['properties']['Name']['description'] |
                Should -BeExactly 'The unique resource identifier.'
        }

        It 'Updates Value description and adds default' {
            $result.ManifestSchema.Embedded['properties']['Value']['description'] |
                Should -BeExactly 'The configuration value to apply.'
            $result.ManifestSchema.Embedded['properties']['Value']['default'] |
                Should -BeExactly ''
        }

        It 'Updates Enabled description and required status' {
            $result.ManifestSchema.Embedded['properties']['Enabled']['description'] |
                Should -BeExactly 'Whether this resource instance is active.'
            $result.ManifestSchema.Embedded['required'] | Should -Contain 'Enabled'
        }
    }

    Context 'Warning for non-existent property' {

        It 'Emits a warning and skips the override' {
            $psd1 = Join-Path $fixturesPath 'SimpleResource' 'SimpleResource.psd1'
            $manifest = New-DscAdaptedResourceManifest -Path $psd1
            $override = [DscPropertyOverride]@{
                Name        = 'DoesNotExist'
                Description = 'Should warn'
            }
            $result = $manifest | Update-DscAdaptedResourceManifest -PropertyOverride $override `
                -WarningVariable warn -WarningAction SilentlyContinue
            $warn | Should -Not -BeNullOrEmpty
            $warn[0] | Should -BeLike "*Property 'DoesNotExist' not found*"
        }
    }

    Context 'No-op when no overrides provided' {

        BeforeAll {
            $psd1 = Join-Path $fixturesPath 'SimpleResource' 'SimpleResource.psd1'
            $manifest = New-DscAdaptedResourceManifest -Path $psd1
            $originalJson = $manifest.ToJson()
            $result = $manifest | Update-DscAdaptedResourceManifest
        }

        It 'Returns the same object' {
            $result | Should -Be $manifest
        }

        It 'Does not modify the manifest' {
            $result.ToJson() | Should -BeExactly $originalJson
        }
    }

    Context 'Pipeline support' {

        It 'Processes multiple manifests from pipeline' {
            $psd1 = Join-Path $fixturesPath 'MultiResource' 'MultiResource.psd1'
            $override = [DscPropertyOverride]@{
                Name        = 'Name'
                Description = 'Overridden name description.'
            }
            $results = New-DscAdaptedResourceManifest -Path $psd1 |
                Update-DscAdaptedResourceManifest -PropertyOverride $override -WarningAction SilentlyContinue
            # ResourceA has Name, ResourceB does not (it has Id)
            $resourceA = $results | Where-Object { $_.Type -eq 'MultiResource/ResourceA' }
            $resourceA.ManifestSchema.Embedded['properties']['Name']['description'] |
                Should -BeExactly 'Overridden name description.'
        }
    }

    Context 'Serialization after update' {

        BeforeAll {
            $psd1 = Join-Path $fixturesPath 'SimpleResource' 'SimpleResource.psd1'
            $manifest = New-DscAdaptedResourceManifest -Path $psd1
            $override = [DscPropertyOverride]@{
                Name        = 'Name'
                Description = 'Custom description for serialization test.'
            }
            $result = $manifest | Update-DscAdaptedResourceManifest -PropertyOverride $override
            $json = $result.ToJson()
            $parsed = $json | ConvertFrom-Json
        }

        It 'Produces valid JSON' {
            { $json | ConvertFrom-Json } | Should -Not -Throw
        }

        It 'Serialized JSON contains the updated description' {
            $parsed.schema.embedded.properties.Name.description |
                Should -BeExactly 'Custom description for serialization test.'
        }
    }
}
