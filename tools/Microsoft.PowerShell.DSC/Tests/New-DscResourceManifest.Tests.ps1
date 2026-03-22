# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'New-DscResourceManifest' {

    BeforeAll {
        $modulePath = Join-Path (Join-Path $PSScriptRoot '..') 'Microsoft.PowerShell.DSC.psd1'
        Import-Module $modulePath -Force

        $fixturesPath = Join-Path $PSScriptRoot 'Fixtures'
    }

    Context 'With adapted resources from pipeline' {

        BeforeAll {
            $psd1 = Join-Path $fixturesPath 'SimpleResource' 'SimpleResource.psd1'
            $adapted = New-DscAdaptedResourceManifest -Path $psd1
            $result = $adapted | New-DscResourceManifest
        }

        It 'Returns a DscResourceManifestList object' {
            $result.GetType().Name | Should -BeExactly 'DscResourceManifestList'
        }

        It 'Contains one adapted resource' {
            $result.AdaptedResources | Should -HaveCount 1
        }

        It 'Has no command-based resources' {
            $result.Resources | Should -HaveCount 0
        }

        It 'Adapted resource has the correct type' {
            $result.AdaptedResources[0]['type'] | Should -BeExactly 'SimpleResource/SimpleResource'
        }

        It 'Adapted resource has the correct schema URI' {
            $result.AdaptedResources[0]['$schema'] | Should -BeExactly 'https://aka.ms/dsc/schemas/v3/bundled/adaptedresource/manifest.json'
        }
    }

    Context 'With multiple adapted resources from pipeline' {

        BeforeAll {
            $paths = @(
                (Join-Path $fixturesPath 'SimpleResource' 'SimpleResource.psd1')
                (Join-Path $fixturesPath 'MultiResource' 'MultiResource.psd1')
            )
            $adapted = $paths | New-DscAdaptedResourceManifest
            $result = $adapted | New-DscResourceManifest
        }

        It 'Contains three adapted resources' {
            $result.AdaptedResources | Should -HaveCount 3
        }

        It 'Includes all resource types' {
            $types = $result.AdaptedResources | ForEach-Object { $_['type'] }
            $types | Should -Contain 'SimpleResource/SimpleResource'
            $types | Should -Contain 'MultiResource/ResourceA'
            $types | Should -Contain 'MultiResource/ResourceB'
        }
    }

    Context 'With AdaptedResource parameter' {

        BeforeAll {
            $psd1 = Join-Path $fixturesPath 'SimpleResource' 'SimpleResource.psd1'
            $adapted = New-DscAdaptedResourceManifest -Path $psd1
            $result = New-DscResourceManifest -AdaptedResource $adapted
        }

        It 'Returns a DscResourceManifestList object' {
            $result.GetType().Name | Should -BeExactly 'DscResourceManifestList'
        }

        It 'Contains one adapted resource' {
            $result.AdaptedResources | Should -HaveCount 1
        }
    }

    Context 'With command-based Resource parameter' {

        BeforeAll {
            $resource = @{
                '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.json'
                type      = 'MyCompany/MyTool'
                version   = '1.0.0'
                get       = @{ executable = 'mytool'; args = @('get') }
                set       = @{
                    executable       = 'mytool'
                    args             = @('set')
                    implementsPretest = $false
                    return           = 'state'
                }
                test      = @{
                    executable = 'mytool'
                    args       = @('test')
                    return     = 'state'
                }
                exitCodes = @{ '0' = 'Success'; '1' = 'Error' }
                schema    = @{
                    command = @{
                        executable = 'mytool'
                        args       = @('schema')
                    }
                }
            }
            $result = New-DscResourceManifest -Resource $resource
        }

        It 'Returns a DscResourceManifestList object' {
            $result.GetType().Name | Should -BeExactly 'DscResourceManifestList'
        }

        It 'Has no adapted resources' {
            $result.AdaptedResources | Should -HaveCount 0
        }

        It 'Contains one command-based resource' {
            $result.Resources | Should -HaveCount 1
        }

        It 'Resource has the correct type' {
            $result.Resources[0]['type'] | Should -BeExactly 'MyCompany/MyTool'
        }

        It 'Resource has the correct schema URI' {
            $result.Resources[0]['$schema'] | Should -BeExactly 'https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.json'
        }

        It 'Resource has get method defined' {
            $result.Resources[0]['get'] | Should -Not -BeNullOrEmpty
            $result.Resources[0]['get']['executable'] | Should -BeExactly 'mytool'
        }
    }

    Context 'Combining adapted and command-based resources' {

        BeforeAll {
            $psd1 = Join-Path $fixturesPath 'SimpleResource' 'SimpleResource.psd1'
            $adapted = New-DscAdaptedResourceManifest -Path $psd1

            $resource = @{
                '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.json'
                type      = 'MyCompany/MyTool'
                version   = '1.0.0'
                get       = @{ executable = 'mytool'; args = @('get') }
            }
            $result = $adapted | New-DscResourceManifest -Resource $resource
        }

        It 'Contains one adapted resource' {
            $result.AdaptedResources | Should -HaveCount 1
        }

        It 'Contains one command-based resource' {
            $result.Resources | Should -HaveCount 1
        }

        It 'Adapted resource has the correct type' {
            $result.AdaptedResources[0]['type'] | Should -BeExactly 'SimpleResource/SimpleResource'
        }

        It 'Command-based resource has the correct type' {
            $result.Resources[0]['type'] | Should -BeExactly 'MyCompany/MyTool'
        }
    }

    Context 'Multiple command-based resources' {

        BeforeAll {
            $resources = @(
                @{
                    '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.json'
                    type      = 'MyCompany/ToolA'
                    version   = '1.0.0'
                    get       = @{ executable = 'toolA'; args = @('get') }
                }
                @{
                    '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.json'
                    type      = 'MyCompany/ToolB'
                    version   = '2.0.0'
                    get       = @{ executable = 'toolB'; args = @('get') }
                }
            )
            $result = New-DscResourceManifest -Resource $resources
        }

        It 'Contains two command-based resources' {
            $result.Resources | Should -HaveCount 2
        }

        It 'Includes both resource types' {
            $types = $result.Resources | ForEach-Object { $_['type'] }
            $types | Should -Contain 'MyCompany/ToolA'
            $types | Should -Contain 'MyCompany/ToolB'
        }
    }

    Context 'ToJson serialization' {

        BeforeAll {
            $psd1 = Join-Path $fixturesPath 'SimpleResource' 'SimpleResource.psd1'
            $adapted = New-DscAdaptedResourceManifest -Path $psd1

            $resource = @{
                '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.json'
                type      = 'MyCompany/MyTool'
                version   = '1.0.0'
                get       = @{ executable = 'mytool'; args = @('get') }
            }
            $manifestList = $adapted | New-DscResourceManifest -Resource $resource
            $json = $manifestList.ToJson()
            $parsed = $json | ConvertFrom-Json
        }

        It 'Produces valid JSON' {
            { $json | ConvertFrom-Json } | Should -Not -Throw
        }

        It 'Contains adaptedResources array' {
            $parsed.adaptedResources | Should -Not -BeNullOrEmpty
            $parsed.adaptedResources | Should -HaveCount 1
        }

        It 'Contains resources array' {
            $parsed.resources | Should -Not -BeNullOrEmpty
            $parsed.resources | Should -HaveCount 1
        }

        It 'Adapted resource in JSON has correct type' {
            $parsed.adaptedResources[0].type | Should -BeExactly 'SimpleResource/SimpleResource'
        }

        It 'Command resource in JSON has correct type' {
            $parsed.resources[0].type | Should -BeExactly 'MyCompany/MyTool'
        }

        It 'Adapted resource schema is embedded in JSON' {
            $parsed.adaptedResources[0].schema.embedded | Should -Not -BeNullOrEmpty
        }
    }

    Context 'ToJson with only adapted resources' {

        BeforeAll {
            $psd1 = Join-Path $fixturesPath 'SimpleResource' 'SimpleResource.psd1'
            $adapted = New-DscAdaptedResourceManifest -Path $psd1
            $manifestList = $adapted | New-DscResourceManifest
            $json = $manifestList.ToJson()
            $parsed = $json | ConvertFrom-Json
        }

        It 'Contains adaptedResources array' {
            $parsed.adaptedResources | Should -Not -BeNullOrEmpty
        }

        It 'Does not contain resources key when empty' {
            $parsed.PSObject.Properties.Name | Should -Not -Contain 'resources'
        }
    }

    Context 'ToJson with only command-based resources' {

        BeforeAll {
            $resource = @{
                '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.json'
                type      = 'MyCompany/MyTool'
                version   = '1.0.0'
                get       = @{ executable = 'mytool'; args = @('get') }
            }
            $manifestList = New-DscResourceManifest -Resource $resource
            $json = $manifestList.ToJson()
            $parsed = $json | ConvertFrom-Json
        }

        It 'Contains resources array' {
            $parsed.resources | Should -Not -BeNullOrEmpty
        }

        It 'Does not contain adaptedResources key when empty' {
            $parsed.PSObject.Properties.Name | Should -Not -Contain 'adaptedResources'
        }
    }

    Context 'No inputs' {

        It 'Returns an empty manifest list when called without arguments' {
            $result = New-DscResourceManifest
            $result.AdaptedResources | Should -HaveCount 0
            $result.Resources | Should -HaveCount 0
        }

        It 'Empty manifest list produces empty JSON object' {
            $result = New-DscResourceManifest
            $json = $result.ToJson()
            $parsed = $json | ConvertFrom-Json
            $parsed.PSObject.Properties.Name | Should -Not -Contain 'adaptedResources'
            $parsed.PSObject.Properties.Name | Should -Not -Contain 'resources'
        }
    }

    Context 'End-to-end pipeline from module to manifests file' {

        It 'Produces valid JSON matching the ManifestList schema structure' {
            $psd1 = Join-Path $fixturesPath 'MultiResource' 'MultiResource.psd1'
            $json = New-DscAdaptedResourceManifest -Path $psd1 |
                New-DscResourceManifest |
                ForEach-Object { $_.ToJson() }

            $parsed = $json | ConvertFrom-Json
            $parsed.adaptedResources | Should -HaveCount 2
            $parsed.adaptedResources[0].type | Should -Not -BeNullOrEmpty
            $parsed.adaptedResources[0].requireAdapter | Should -BeExactly 'Microsoft.Adapter/PowerShell'
            $parsed.adaptedResources[0].schema.embedded | Should -Not -BeNullOrEmpty
        }
    }
}
