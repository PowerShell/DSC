# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Windows Update resource schema validation' {
    BeforeAll {
        $resourceType = 'Microsoft.Windows/Updates'
        $manifestPath = Join-Path $PSScriptRoot "..\windowsupdate.dsc.resource.json"
    }

    Context 'Manifest validation' {
        It 'manifest file should exist' {
            Test-Path $manifestPath | Should -Be $true
        }

        It 'manifest should be valid JSON' {
            { Get-Content $manifestPath | ConvertFrom-Json } | Should -Not -Throw
        }

        It 'manifest should have correct type' {
            $manifest = Get-Content $manifestPath | ConvertFrom-Json
            $manifest.type | Should -BeExactly $resourceType
        }

        It 'manifest should have version' {
            $manifest = Get-Content $manifestPath | ConvertFrom-Json
            $manifest.version | Should -Not -BeNullOrEmpty
            $manifest.version | Should -Match '^\d+\.\d+\.\d+$'
        }

        It 'manifest should have description' {
            $manifest = Get-Content $manifestPath | ConvertFrom-Json
            $manifest.description | Should -Not -BeNullOrEmpty
        }

        It 'manifest should have get operation' {
            $manifest = Get-Content $manifestPath | ConvertFrom-Json
            $manifest.get | Should -Not -BeNullOrEmpty
            $manifest.get.executable | Should -BeExactly 'wu_dsc'
            $manifest.get.args | Should -Contain 'get'
            $manifest.get.input | Should -BeExactly 'stdin'
        }

        It 'manifest should have tags' {
            $manifest = Get-Content $manifestPath | ConvertFrom-Json
            $manifest.tags | Should -Not -BeNullOrEmpty
            $manifest.tags | Should -BeOfType [array]
        }
    }

    Context 'Schema validation' {
        It 'should have embedded schema' {
            $manifest = Get-Content $manifestPath | ConvertFrom-Json
            $manifest.schema.embedded | Should -Not -BeNullOrEmpty
        }

        It 'schema should have correct JSON schema version' {
            $manifest = Get-Content $manifestPath | ConvertFrom-Json
            $manifest.schema.embedded.'$schema' | Should -Match 'json-schema.org'
        }

        It 'schema should have title property' {
            $manifest = Get-Content $manifestPath | ConvertFrom-Json
            $manifest.schema.embedded.title | Should -Not -BeNullOrEmpty
        }

        It 'schema should require title property' {
            $manifest = Get-Content $manifestPath | ConvertFrom-Json
            $manifest.schema.embedded.required | Should -Contain 'title'
        }

        It 'schema should define all expected properties' {
            $manifest = Get-Content $manifestPath | ConvertFrom-Json
            $properties = $manifest.schema.embedded.properties
            
            $expectedProperties = @(
                'title',
                'isInstalled',
                'description',
                'id',
                'isUninstallable',
                'KBArticleIDs',
                'maxDownloadSize',
                'msrcSeverity',
                'securityBulletinIds',
                'updateType'
            )
            
            foreach ($prop in $expectedProperties) {
                $properties.$prop | Should -Not -BeNullOrEmpty -Because "Property '$prop' should be defined"
            }
        }

        It 'title property should be string type' {
            $manifest = Get-Content $manifestPath | ConvertFrom-Json
            $manifest.schema.embedded.properties.title.type | Should -BeExactly 'string'
        }

        It 'isInstalled property should be boolean and readOnly' {
            $manifest = Get-Content $manifestPath | ConvertFrom-Json
            $isInstalled = $manifest.schema.embedded.properties.isInstalled
            $isInstalled.type | Should -BeExactly 'boolean'
            $isInstalled.readOnly | Should -Be $true
        }

        It 'description property should be string and readOnly' {
            $manifest = Get-Content $manifestPath | ConvertFrom-Json
            $description = $manifest.schema.embedded.properties.description
            $description.type | Should -BeExactly 'string'
            $description.readOnly | Should -Be $true
        }

        It 'id property should be string and readOnly' {
            $manifest = Get-Content $manifestPath | ConvertFrom-Json
            $id = $manifest.schema.embedded.properties.id
            $id.type | Should -BeExactly 'string'
            $id.readOnly | Should -Be $true
        }

        It 'isUninstallable property should be boolean and readOnly' {
            $manifest = Get-Content $manifestPath | ConvertFrom-Json
            $isUninstallable = $manifest.schema.embedded.properties.isUninstallable
            $isUninstallable.type | Should -BeExactly 'boolean'
            $isUninstallable.readOnly | Should -Be $true
        }

        It 'KBArticleIDs property should be array and readOnly' {
            $manifest = Get-Content $manifestPath | ConvertFrom-Json
            $kbArticles = $manifest.schema.embedded.properties.KBArticleIDs
            $kbArticles.type | Should -BeExactly 'array'
            $kbArticles.readOnly | Should -Be $true
            $kbArticles.items.type | Should -BeExactly 'string'
        }

        It 'maxDownloadSize property should be integer int64 and readOnly' {
            $manifest = Get-Content $manifestPath | ConvertFrom-Json
            $maxDownloadSize = $manifest.schema.embedded.properties.maxDownloadSize
            $maxDownloadSize.type | Should -BeExactly 'integer'
            $maxDownloadSize.format | Should -BeExactly 'int64'
            $maxDownloadSize.readOnly | Should -Be $true
        }

        It 'msrcSeverity property should be enum with correct values' {
            $manifest = Get-Content $manifestPath | ConvertFrom-Json
            $msrcSeverity = $manifest.schema.embedded.properties.msrcSeverity
            $msrcSeverity.type | Should -BeExactly 'string'
            $msrcSeverity.enum | Should -Contain 'Critical'
            $msrcSeverity.enum | Should -Contain 'Important'
            $msrcSeverity.enum | Should -Contain 'Moderate'
            $msrcSeverity.enum | Should -Contain 'Low'
            $msrcSeverity.readOnly | Should -Be $true
        }

        It 'securityBulletinIds property should be array and readOnly' {
            $manifest = Get-Content $manifestPath | ConvertFrom-Json
            $bulletinIds = $manifest.schema.embedded.properties.securityBulletinIds
            $bulletinIds.type | Should -BeExactly 'array'
            $bulletinIds.readOnly | Should -Be $true
            $bulletinIds.items.type | Should -BeExactly 'string'
        }

        It 'updateType property should be enum with correct values' {
            $manifest = Get-Content $manifestPath | ConvertFrom-Json
            $updateType = $manifest.schema.embedded.properties.updateType
            $updateType.type | Should -BeExactly 'string'
            $updateType.enum | Should -Contain 'Software'
            $updateType.enum | Should -Contain 'Driver'
            $updateType.readOnly | Should -Be $true
        }

        It 'schema should not allow additional properties' {
            $manifest = Get-Content $manifestPath | ConvertFrom-Json
            $manifest.schema.embedded.additionalProperties | Should -Be $false
        }

        It 'all properties should have descriptions' {
            $manifest = Get-Content $manifestPath | ConvertFrom-Json
            $properties = $manifest.schema.embedded.properties
            
            foreach ($propName in $properties.PSObject.Properties.Name) {
                $prop = $properties.$propName
                $prop.description | Should -Not -BeNullOrEmpty -Because "Property '$propName' should have a description"
            }
        }

        It 'all properties should have titles' {
            $manifest = Get-Content $manifestPath | ConvertFrom-Json
            $properties = $manifest.schema.embedded.properties
            
            foreach ($propName in $properties.PSObject.Properties.Name) {
                $prop = $properties.$propName
                $prop.title | Should -Not -BeNullOrEmpty -Because "Property '$propName' should have a title"
            }
        }
    }

    Context 'Documentation links' {
        It 'schema should have valid schema ID URL' {
            $manifest = Get-Content $manifestPath | ConvertFrom-Json
            $schemaId = $manifest.schema.embedded.'$id'
            $schemaId | Should -Not -BeNullOrEmpty
            $schemaId | Should -Match '^https://'
            $schemaId | Should -Match 'Microsoft\.Windows/Updates'
        }

        It 'description should reference documentation URL' {
            $manifest = Get-Content $manifestPath | ConvertFrom-Json
            $manifest.schema.embedded.description | Should -Match 'https://'
        }
    }
}
