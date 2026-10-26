# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'PowerShell adapter embedded schemas' {
    BeforeAll {
        $adapterPath = Split-Path -Path $PSScriptRoot -Parent
        $scriptPath = Join-Path $adapterPath 'psDscAdapter' 'powershell.resource.ps1'
        $oldModulePath = $env:PSModulePath
        $oldLocalAppData = $env:LOCALAPPDATA
        $oldHome = $env:HOME
        $oldRestrictedPath = $env:DSC_RESTRICTED_PATH
        $oldResourcePath = $env:DSC_RESOURCE_PATH
        $env:LOCALAPPDATA = $TestDrive
        $env:HOME = $TestDrive
        $env:PSModulePath = $TestDrive
        $env:DSC_RESOURCE_PATH = $null
        $env:DSC_RESTRICTED_PATH = $adapterPath + [IO.Path]::PathSeparator + $PSHOME

        $modulePath = Join-Path $TestDrive 'TestSchemaResource'
        New-Item -Path $modulePath -ItemType Directory | Out-Null
        @'
@{
    RootModule = 'TestSchemaResource.psm1'
    ModuleVersion = '1.0.0'
    DscResourcesToExport = @('TestSchemaResource')
}
'@ | Set-Content -Path (Join-Path $modulePath 'TestSchemaResource.psd1')
        @'
enum SchemaEnsure {
    Present
    Absent
}

class SchemaBase {
    [DscProperty()]
    [string] $Inherited
}

[DscResource()]
class TestSchemaResource : SchemaBase {
    [DscProperty(Key)]
    [string] $Name

    [DscProperty(Mandatory)]
    [string] $RequiredValue

    [DscProperty()]
    [bool] $Enabled

    [DscProperty()]
    [int] $Count

    [DscProperty()]
    [double] $Ratio

    [DscProperty()]
    [ValidateSet(1, 2)]
    [int] $Selection

    [DscProperty()]
    [ValidateSet('One', 'Two')]
    [string] $Choice

    [DscProperty()]
    [string[]] $Names

    [DscProperty()]
    [SchemaEnsure] $Ensure

    [DscProperty()]
    [hashtable] $Options

    [DscProperty()]
    [PSCredential] $Credential

    [DscProperty()]
    [SecureString] $Secret

    [string] $NotDscProperty

    [TestSchemaResource] Get() { return $this }
    [void] Set() {}
    [bool] Test() { return $true }
}
'@ | Set-Content -Path (Join-Path $modulePath 'TestSchemaResource.psm1')

        $resources = & $scriptPath -Operation List -ResourceType Single | ConvertFrom-Json
        $resource = $resources | Where-Object -Property type -EQ 'TestSchemaResource/TestSchemaResource'
        $schema = $resource.schema | ConvertTo-Json -Depth 100 -Compress
    }

    AfterAll {
        $env:PSModulePath = $oldModulePath
        $env:LOCALAPPDATA = $oldLocalAppData
        $env:HOME = $oldHome
        $env:DSC_RESTRICTED_PATH = $oldRestrictedPath
        $env:DSC_RESOURCE_PATH = $oldResourcePath
    }

    It 'Removes validate and embeds a schema in <Manifest>' -ForEach @(
        @{ Manifest = 'PowerShell_adapter.dsc.resource.json' }
        @{ Manifest = 'WindowsPowerShell_adapter.dsc.resource.json' }
        @{ Manifest = 'powershell.dsc.resource.json' }
        @{ Manifest = 'windowspowershell.dsc.resource.json' }
    ) {
        $manifestContent = Get-Content -Raw -Path (Join-Path $adapterPath $Manifest) | ConvertFrom-Json
        $manifestContent.PSObject.Properties.Name | Should -Not -Contain 'validate'
        $manifestContent.schema.embedded.type | Should -BeExactly 'object'
    }

    It 'Rejects the removed Validate operation' {
        $null = & pwsh -NoProfile -File $scriptPath -Operation Validate 2>&1
        $LASTEXITCODE | Should -Not -Be 0
    }

    It 'Allows legacy resource instances without export filters in <Manifest>' -ForEach @(
        @{ Manifest = 'powershell.dsc.resource.json' }
        @{ Manifest = 'windowspowershell.dsc.resource.json' }
    ) {
        $manifestContent = Get-Content -Raw -Path (Join-Path $adapterPath $Manifest) | ConvertFrom-Json
        $legacySchema = $manifestContent.schema.embedded | ConvertTo-Json -Depth 100
        Test-Json -Json '{"resources":[{"name":"export","type":"TestSchemaResource/TestSchemaResource"}]}' -Schema $legacySchema -ErrorAction Stop |
            Should -BeTrue
        Test-Json -Json '{"resources":"invalid"}' -Schema $legacySchema -ErrorAction SilentlyContinue |
            Should -BeFalse
    }

    It 'Includes a resource schema with inherited DSC properties only' {
        $resource.requireAdapter | Should -BeExactly 'Microsoft.Adapter/PowerShell'
        $resource.schema.type | Should -BeExactly 'object'
        $resource.schema.properties.PSObject.Properties.Name | Should -Contain 'Inherited'
        $resource.schema.properties.PSObject.Properties.Name | Should -Not -Contain 'NotDscProperty'
        $resource.schema.properties.Ensure.anyOf[0].enum | Should -Be @('Present', 'Absent')
    }

    It 'Accepts valid resource data including nullable and complex properties' {
        $json = '{"Name":"test","RequiredValue":"value","Enabled":true,"Count":2,"Ratio":1.5,"Names":["one"],"Ensure":"Present","Options":{"nested":{"value":1}},"Credential":{"username":"user","password":"secret"},"Secret":"secret","Inherited":null}'
        Test-Json -Json $json -Schema $schema -ErrorAction Stop | Should -BeTrue
    }

    It 'Preserves case-insensitive PowerShell property names' {
        Test-Json -Json '{"name":"test","requiredvalue":"value","count":2}' -Schema $schema -ErrorAction Stop |
            Should -BeTrue
    }

    It 'Preserves case-insensitive enumeration values' {
        Test-Json -Json '{"Name":"test","RequiredValue":"value","Ensure":"present","Choice":"one","Selection":1}' -Schema $schema -ErrorAction Stop |
            Should -BeTrue
    }

    It 'Accepts the serialized representation of a SecureString in resource output' {
        Test-Json -Json '{"Name":"test","RequiredValue":"value","Secret":{"Length":6}}' -Schema $schema -ErrorAction Stop |
            Should -BeTrue
    }

    It 'Rejects <Reason>' -ForEach @(
        @{ Reason = 'a missing key'; Json = '{"RequiredValue":"value"}' }
        @{ Reason = 'a missing mandatory property'; Json = '{"Name":"test"}' }
        @{ Reason = 'a non-string key'; Json = '{"Name":42,"RequiredValue":"value"}' }
        @{ Reason = 'a non-boolean value'; Json = '{"Name":"test","RequiredValue":"value","Enabled":"yes"}' }
        @{ Reason = 'a fractional integer'; Json = '{"Name":"test","RequiredValue":"value","Count":1.5}' }
        @{ Reason = 'a non-numeric value'; Json = '{"Name":"test","RequiredValue":"value","Ratio":"bad"}' }
        @{ Reason = 'an invalid array item'; Json = '{"Name":"test","RequiredValue":"value","Names":[1]}' }
        @{ Reason = 'an invalid enumeration'; Json = '{"Name":"test","RequiredValue":"value","Ensure":"Other"}' }
        @{ Reason = 'an invalid string validation set'; Json = '{"Name":"test","RequiredValue":"value","Choice":"Other"}' }
        @{ Reason = 'an invalid numeric validation set'; Json = '{"Name":"test","RequiredValue":"value","Selection":3}' }
        @{ Reason = 'a non-object credential'; Json = '{"Name":"test","RequiredValue":"value","Credential":"secret"}' }
        @{ Reason = 'an incorrectly typed case-insensitive property'; Json = '{"name":"test","requiredvalue":"value","count":"bad"}' }
    ) {
        Test-Json -Json $Json -Schema $schema -ErrorAction SilentlyContinue | Should -BeFalse
    }

    It 'Returns the same schema when listing cached resources' {
        $cached = & $scriptPath -Operation List -ResourceType Single | ConvertFrom-Json |
            Where-Object -Property type -EQ 'TestSchemaResource/TestSchemaResource'
        Test-Json -Json '{"Name":"test","RequiredValue":"value","Count":"bad"}' -Schema ($cached.schema | ConvertTo-Json -Depth 100) -ErrorAction SilentlyContinue |
            Should -BeFalse
        $cached.schema.properties.Ensure.anyOf[0].enum | Should -Be @('Present', 'Absent')
    }

    It 'Embeds schemas for both deprecated and single-resource adapters' {
        $legacy = & $scriptPath -Operation List | ConvertFrom-Json |
            Where-Object -Property type -EQ 'TestSchemaResource/TestSchemaResource'
        $legacy.requireAdapter | Should -BeExactly 'Microsoft.DSC/PowerShell'
        Test-Json -Json '{"Name":"test","RequiredValue":"value","Count":"bad"}' -Schema ($legacy.schema | ConvertTo-Json -Depth 100) -ErrorAction SilentlyContinue |
            Should -BeFalse
    }

    It 'Produces usable Windows PowerShell schemas from live and cached metadata' -Skip:(!$IsWindows) {
        try {
            $env:PSModulePath += [IO.Path]::PathSeparator + $PSScriptRoot
            foreach ($iteration in 1..2) {
                $windowsResources = & powershell -NoProfile -ExecutionPolicy Bypass -File $scriptPath -Operation List -ResourceType Single 2> (Join-Path $TestDrive 'windows-schema-error.log')
                $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw (Join-Path $TestDrive 'windows-schema-error.log'))
                $windowsResource = $windowsResources | ConvertFrom-Json |
                    Where-Object -Property type -EQ 'TestSchemaResource/TestSchemaResource'
                $windowsResource.requireAdapter | Should -BeExactly 'Microsoft.Adapter/WindowsPowerShell'
                $windowsResource.schema.properties.Count.type | Should -BeExactly 'integer'
                $windowsResource.schema.properties.Names.items.type | Should -Contain 'string'
                $windowsSchema = $windowsResource.schema | ConvertTo-Json -Depth 100
                Test-Json -Json '{"name":"test","requiredvalue":"value","Ensure":"present"}' -Schema $windowsSchema -ErrorAction Stop |
                    Should -BeTrue
                Test-Json -Json '{"Name":"test","RequiredValue":"value","Count":"bad"}' -Schema $windowsSchema -ErrorAction SilentlyContinue |
                    Should -BeFalse
                $scriptResource = $windowsResources | ConvertFrom-Json |
                    Where-Object -Property type -EQ 'TestScriptBaseDSC/CredentialValidation'
                $scriptResource.schema.properties.Credential.type | Should -Contain 'object'
                $scriptSchema = $scriptResource.schema | ConvertTo-Json -Depth 100
                Test-Json -Json '{"Name":"test","Credential":{"username":"user","password":"secret"}}' -Schema $scriptSchema -ErrorAction Stop |
                    Should -BeTrue
                Test-Json -Json '{"Name":"test"}' -Schema $scriptSchema -ErrorAction SilentlyContinue |
                    Should -BeFalse
            }
        } finally {
            $env:PSModulePath = $TestDrive
        }
    }

    It 'Provides the embedded schema through the DSC CLI' {
        $out = dsc resource schema -r 'TestSchemaResource/TestSchemaResource' 2> (Join-Path $TestDrive 'schema-error.log')
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw (Join-Path $TestDrive 'schema-error.log'))
        ($out | ConvertFrom-Json).properties.Count.type | Should -BeExactly 'integer'
    }

    It 'Rejects invalid resource input through the DSC CLI' {
        $null = dsc resource get -r 'TestSchemaResource/TestSchemaResource' -i '{"Name":"test","RequiredValue":"value","Count":"bad"}' 2> (Join-Path $TestDrive 'invalid-error.log')
        $LASTEXITCODE | Should -Not -Be 0
        Get-Content -Raw (Join-Path $TestDrive 'invalid-error.log') | Should -Match 'integer'
    }
}
