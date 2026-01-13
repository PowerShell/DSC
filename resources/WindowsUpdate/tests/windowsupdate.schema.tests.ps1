# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Windows Update resource schema validation' {
    BeforeAll {
        $resourceType = 'Microsoft.Windows/UpdateList'
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

        It 'schema should require updates property' {
            $manifest = Get-Content $manifestPath | ConvertFrom-Json
            $manifest.schema.embedded.required | Should -Contain 'updates'
        }

        It 'schema should define updates property as array' {
            $manifest = Get-Content $manifestPath | ConvertFrom-Json
            $properties = $manifest.schema.embedded.properties
            $properties.updates | Should -Not -BeNullOrEmpty
            $properties.updates.type | Should -BeExactly 'array'
        }

        It 'schema should define all expected properties in updates items' {
            $manifest = Get-Content $manifestPath | ConvertFrom-Json
            $itemProperties = $manifest.schema.embedded.properties.updates.items.properties
            
            $expectedProperties = @(
                'title',
                'isInstalled',
                'description',
                'id',
                'isUninstallable',
                'kbArticleIds',
                'minDownloadSize',
                'msrcSeverity',
                'securityBulletinIds',
                'updateType'
            )
            
            foreach ($prop in $expectedProperties) {
                $itemProperties.$prop | Should -Not -BeNullOrEmpty -Because "Property '$prop' should be defined"
            }
        }

        It 'title property should be string type' {
            $manifest = Get-Content $manifestPath | ConvertFrom-Json
            $manifest.schema.embedded.properties.updates.items.properties.title.type | Should -BeExactly 'string'
        }

        It 'isInstalled property should be boolean' {
            $manifest = Get-Content $manifestPath | ConvertFrom-Json
            $isInstalled = $manifest.schema.embedded.properties.updates.items.properties.isInstalled
            $isInstalled.type | Should -BeExactly 'boolean'
        }

        It 'description property should be string' {
            $manifest = Get-Content $manifestPath | ConvertFrom-Json
            $description = $manifest.schema.embedded.properties.updates.items.properties.description
            $description.type | Should -BeExactly 'string'
        }

        It 'id property should be string' {
            $manifest = Get-Content $manifestPath | ConvertFrom-Json
            $id = $manifest.schema.embedded.properties.updates.items.properties.id
            $id.type | Should -BeExactly 'string'
        }

        It 'isUninstallable property should be boolean' {
            $manifest = Get-Content $manifestPath | ConvertFrom-Json
            $isUninstallable = $manifest.schema.embedded.properties.updates.items.properties.isUninstallable
            $isUninstallable.type | Should -BeExactly 'boolean'
        }

        It 'kbArticleIds property should be array' {
            $manifest = Get-Content $manifestPath | ConvertFrom-Json
            $kbArticles = $manifest.schema.embedded.properties.updates.items.properties.kbArticleIds
            $kbArticles.type | Should -BeExactly 'array'
            $kbArticles.items.type | Should -BeExactly 'string'
        }

        It 'minDownloadSize property should be integer int64' {
            $manifest = Get-Content $manifestPath | ConvertFrom-Json
            $minDownloadSize = $manifest.schema.embedded.properties.updates.items.properties.minDownloadSize
            $minDownloadSize.type | Should -BeExactly 'integer'
            $minDownloadSize.format | Should -BeExactly 'int64'
        }

        It 'msrcSeverity property should be enum with correct values' {
            $manifest = Get-Content $manifestPath | ConvertFrom-Json
            $msrcSeverity = $manifest.schema.embedded.properties.updates.items.properties.msrcSeverity
            $msrcSeverity.type | Should -BeExactly 'string'
            $msrcSeverity.enum | Should -Contain 'Critical'
            $msrcSeverity.enum | Should -Contain 'Important'
            $msrcSeverity.enum | Should -Contain 'Moderate'
            $msrcSeverity.enum | Should -Contain 'Low'
        }

        It 'securityBulletinIds property should be array' {
            $manifest = Get-Content $manifestPath | ConvertFrom-Json
            $bulletinIds = $manifest.schema.embedded.properties.updates.items.properties.securityBulletinIds
            $bulletinIds.type | Should -BeExactly 'array'
            $bulletinIds.items.type | Should -BeExactly 'string'
        }

        It 'updateType property should be enum with correct values' {
            $manifest = Get-Content $manifestPath | ConvertFrom-Json
            $updateType = $manifest.schema.embedded.properties.updates.items.properties.updateType
            $updateType.type | Should -BeExactly 'string'
            $updateType.enum | Should -Contain 'Software'
            $updateType.enum | Should -Contain 'Driver'
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
            $schemaId | Should -Match 'Microsoft\.Windows/UpdateList'
        }

        It 'description should reference documentation URL' {
            $manifest = Get-Content $manifestPath | ConvertFrom-Json
            $manifest.schema.embedded.description | Should -Match 'https://'
        }
    }
}
