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
    DscResourcesToExport = @('TestSchemaResource', 'TestNullableResource')
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

    [DscProperty()]
    [SchemaEnsure] $InheritedEnsure
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
    [ValidateSet('One', 'Two')]
    [string[]] $Choices

    [DscProperty()]
    [string[]] $Names

    [DscProperty()]
    [SchemaEnsure] $Ensure

    [DscProperty()]
    [SchemaEnsure[]] $Ensures

    [DscProperty()]
    [hashtable] $Options

    [DscProperty()]
    [PSCredential] $Credential

    [DscProperty()]
    [SecureString] $Secret

    [DscProperty()]
    [string] $IPAddress

    [DscProperty()]
    [string] ${Resource-Name}

    [string] $NotDscProperty

    [TestSchemaResource] Get() { return $this }
    [void] Set() {}
    [bool] Test() { return $true }
}

[DscResource()]
class TestNullableResource {
    [DscProperty(Key)]
    [string] $Name

    [DscProperty(Mandatory)]
    [ValidateSet(1, 2)]
    [Nullable[int]] $RequiredCount

    [DscProperty(Mandatory)]
    [Nullable[SchemaEnsure]] $RequiredEnsure

    [DscProperty()]
    [System.Nullable[bool]] $Enabled

    [DscProperty()]
    [ValidateSet(1, 2)]
    [Nullable[int]] $Selection

    [DscProperty()]
    [Nullable[SchemaEnsure]] $Ensure

    [DscProperty()]
    [AllowNull()]
    [ValidateSet('One', 'Two')]
    [string] $Choice

    [DscProperty()]
    [System.Management.Automation.AllowNullAttribute()]
    [ValidateSet('One', 'Two')]
    [string[]] $Choices

    [TestNullableResource] Get() { return $this }
    [void] Set() {}
    [bool] Test() { return $true }
}
'@ | Set-Content -Path (Join-Path $modulePath 'TestSchemaResource.psm1')

        $resources = & $scriptPath -Operation List -ResourceType Single | ConvertFrom-Json
        $resource = $resources | Where-Object -Property type -EQ 'TestSchemaResource/TestSchemaResource'
        $schema = $resource.schema | ConvertTo-Json -Depth 100 -Compress
        $nullableResource = $resources | Where-Object -Property type -EQ 'TestSchemaResource/TestNullableResource'
        $nullableSchema = $nullableResource.schema | ConvertTo-Json -Depth 100 -Compress
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
        $resource.schema.properties.PSObject.Properties.Name | Should -Contain 'inherited'
        $resource.schema.properties.PSObject.Properties.Name | Should -Not -Contain 'notDscProperty'
        $resource.schema.properties.Ensure.anyOf[0].enum | Should -Be @('Present', 'Absent')
        $resource.schema.properties.InheritedEnsure.anyOf[0].enum | Should -Be @('Present', 'Absent')
        $resource.schema.properties.Ensures.items.anyOf[0].enum | Should -Be @('Present', 'Absent')
    }

    It 'Accepts valid resource data including complex properties' {
        $json = '{"name":"test","requiredValue":"value","enabled":true,"count":2,"ratio":1.5,"names":["one"],"ensure":"Present","options":{"nested":{"value":1}},"credential":{"username":"user","password":"secret"},"secret":"secret","inherited":"value"}'
        Test-Json -Json $json -Schema $schema -ErrorAction Stop | Should -BeTrue
    }

    It 'Canonicalizes alphanumeric names without duplicating property schemas' {
        $resource.properties | Should -Contain 'ipAddress'
        $resource.properties | Should -Contain 'Resource-Name'
        $propertyNames = $resource.schema.properties.PSObject.Properties.Name
        $propertyNames -ccontains 'requiredValue' | Should -BeTrue
        $propertyNames -ccontains 'ipAddress' | Should -BeTrue
        $propertyNames -ccontains 'Resource-Name' | Should -BeTrue
        $propertyNames -ccontains 'RequiredValue' | Should -BeFalse
        $resource.schema.required | Should -Be @('name', 'requiredValue')
        $resource.schema.PSObject.Properties.Name | Should -Not -Contain 'patternProperties'
        $resource.schema.PSObject.Properties.Name | Should -Not -Contain 'allOf'
    }

    It 'Preserves case-insensitive enumeration values' {
        Test-Json -Json '{"name":"test","requiredValue":"value","ensure":"present","choice":"one","selection":1}' -Schema $schema -ErrorAction Stop |
            Should -BeTrue
    }

    It 'Accepts explicit nullability for <Case>' -ForEach @(
        @{ Case = 'required nullable value types'; Json = '{"name":"test","requiredCount":null,"requiredEnsure":null}' }
        @{ Case = 'an AllowNull scalar'; Json = '{"name":"test","requiredCount":1,"requiredEnsure":"present","choice":null}' }
        @{ Case = 'an AllowNull array'; Json = '{"name":"test","requiredCount":1,"requiredEnsure":"Present","choices":null}' }
        @{ Case = 'non-null optional nullable values'; Json = '{"name":"test","requiredCount":2,"requiredEnsure":"Absent","enabled":true,"selection":1,"ensure":"present","choices":["one","Two"]}' }
    ) {
        Test-Json -Json $Json -Schema $nullableSchema -ErrorAction Stop | Should -BeTrue
    }

    It 'Rejects nullable metadata misuse for <Reason>' -ForEach @(
        @{ Reason = 'a missing required nullable property'; Json = '{"name":"test","requiredEnsure":null}' }
        @{ Reason = 'a null optional nullable boolean'; Json = '{"name":"test","requiredCount":1,"requiredEnsure":null,"enabled":null}' }
        @{ Reason = 'a null optional nullable integer'; Json = '{"name":"test","requiredCount":1,"requiredEnsure":null,"selection":null}' }
        @{ Reason = 'a null optional nullable enum'; Json = '{"name":"test","requiredCount":1,"requiredEnsure":null,"ensure":null}' }
        @{ Reason = 'a null item in an AllowNull array'; Json = '{"name":"test","requiredCount":1,"requiredEnsure":null,"choices":["One",null]}' }
        @{ Reason = 'an invalid required nullable numeric validation set'; Json = '{"name":"test","requiredCount":3,"requiredEnsure":null}' }
        @{ Reason = 'an invalid required nullable enum'; Json = '{"name":"test","requiredCount":1,"requiredEnsure":"Other"}' }
        @{ Reason = 'an invalid optional nullable numeric validation set'; Json = '{"name":"test","requiredCount":1,"requiredEnsure":null,"selection":3}' }
    ) {
        Test-Json -Json $Json -Schema $nullableSchema -ErrorAction Ignore | Should -BeFalse
    }

    It 'Preserves nested array and enumeration schemas during listing' {
        $json = '{"name":"test","requiredValue":"value","ensures":["Present","absent"],"inheritedEnsure":"present"}'
        Test-Json -Json $json -Schema $schema -ErrorAction Stop | Should -BeTrue
    }

    It 'Accepts the serialized representation of a SecureString in resource output' {
        Test-Json -Json '{"name":"test","requiredValue":"value","secret":{"Length":6}}' -Schema $schema -ErrorAction Stop |
            Should -BeTrue
    }

    It 'Rejects <Reason>' -ForEach @(
        @{ Reason = 'a missing key'; Json = '{"requiredValue":"value"}' }
        @{ Reason = 'a missing mandatory property'; Json = '{"name":"test"}' }
        @{ Reason = 'a non-string key'; Json = '{"name":42,"requiredValue":"value"}' }
        @{ Reason = 'a non-boolean value'; Json = '{"name":"test","requiredValue":"value","enabled":"yes"}' }
        @{ Reason = 'a fractional integer'; Json = '{"name":"test","requiredValue":"value","count":1.5}' }
        @{ Reason = 'a non-numeric value'; Json = '{"name":"test","requiredValue":"value","ratio":"bad"}' }
        @{ Reason = 'an invalid array item'; Json = '{"name":"test","requiredValue":"value","names":[1]}' }
        @{ Reason = 'an invalid enumeration'; Json = '{"name":"test","requiredValue":"value","ensure":"Other"}' }
        @{ Reason = 'an invalid string validation set'; Json = '{"name":"test","requiredValue":"value","choice":"Other"}' }
        @{ Reason = 'an invalid numeric validation set'; Json = '{"name":"test","requiredValue":"value","selection":3}' }
        @{ Reason = 'an invalid array validation-set item'; Json = '{"name":"test","requiredValue":"value","choices":["Other"]}' }
        @{ Reason = 'an invalid enum-array item'; Json = '{"name":"test","requiredValue":"value","ensures":["Other"]}' }
        @{ Reason = 'a null value for a non-nullable enum'; Json = '{"name":"test","requiredValue":"value","ensure":null}' }
        @{ Reason = 'a null numeric validation set'; Json = '{"name":"test","requiredValue":"value","selection":null}' }
        @{ Reason = 'a null string'; Json = '{"name":"test","requiredValue":"value","inherited":null}' }
        @{ Reason = 'a null string validation set'; Json = '{"name":"test","requiredValue":"value","choice":null}' }
        @{ Reason = 'a null array'; Json = '{"name":"test","requiredValue":"value","names":null}' }
        @{ Reason = 'a null array item'; Json = '{"name":"test","requiredValue":"value","choices":["One",null]}' }
        @{ Reason = 'a null object'; Json = '{"name":"test","requiredValue":"value","options":null}' }
        @{ Reason = 'a null SecureString'; Json = '{"name":"test","requiredValue":"value","secret":null}' }
        @{ Reason = 'a non-object credential'; Json = '{"name":"test","requiredValue":"value","credential":"secret"}' }
        @{ Reason = 'noncanonical required property casing'; Json = '{"name":"test","RequiredValue":"value"}' }
    ) {
        Test-Json -Json $Json -Schema $schema -ErrorAction Ignore | Should -BeFalse
    }

    It 'Returns the same schema when listing cached resources' {
        $cachedResources = & $scriptPath -Operation List -ResourceType Single | ConvertFrom-Json
        $cached = $cachedResources | Where-Object -Property type -EQ 'TestSchemaResource/TestSchemaResource'
        Test-Json -Json '{"name":"test","requiredValue":"value","count":"bad"}' -Schema ($cached.schema | ConvertTo-Json -Depth 100) -ErrorAction Ignore |
            Should -BeFalse
        $cachedNullable = $cachedResources | Where-Object -Property type -EQ 'TestSchemaResource/TestNullableResource'
        $cachedNullableSchema = $cachedNullable.schema | ConvertTo-Json -Depth 100
        Test-Json -Json '{"name":"test","requiredCount":null,"requiredEnsure":null,"choice":null,"choices":null}' -Schema $cachedNullableSchema -ErrorAction Stop |
            Should -BeTrue
        Test-Json -Json '{"name":"test","requiredCount":1,"requiredEnsure":null,"enabled":null}' -Schema $cachedNullableSchema -ErrorAction Ignore |
            Should -BeFalse
        $cached.schema.properties.Ensure.anyOf[0].enum | Should -Be @('Present', 'Absent')
    }

    It 'Embeds schemas for both deprecated and single-resource adapters' {
        $legacy = & $scriptPath -Operation List | ConvertFrom-Json |
            Where-Object -Property type -EQ 'TestSchemaResource/TestSchemaResource'
        $legacy.requireAdapter | Should -BeExactly 'Microsoft.DSC/PowerShell'
        Test-Json -Json '{"name":"test","requiredValue":"value","count":"bad"}' -Schema ($legacy.schema | ConvertTo-Json -Depth 100) -ErrorAction Ignore |
            Should -BeFalse
    }

    It 'Produces usable Windows PowerShell schemas from live and cached metadata' -Skip:(!$IsWindows) {
        try {
            $windowsModuleRoot = Join-Path $TestDrive 'WindowsModules'
            $windowsModulePath = Join-Path $windowsModuleRoot 'TestSchemaResource'
            New-Item -Path $windowsModuleRoot -ItemType Directory | Out-Null
            Copy-Item -Path $modulePath -Destination $windowsModulePath -Recurse
            # Windows PowerShell's MOF parser cannot represent hyphenated property names.
            $windowsModuleFile = Join-Path $windowsModulePath 'TestSchemaResource.psm1'
            (Get-Content -Raw $windowsModuleFile).Replace('${Resource-Name}', '$ResourceName') |
                Set-Content -Path $windowsModuleFile
            $env:PSModulePath = $windowsModuleRoot + [IO.Path]::PathSeparator + $PSScriptRoot
            foreach ($iteration in 1..2) {
                $windowsResources = & powershell -NoProfile -ExecutionPolicy Bypass -File $scriptPath -Operation List -ResourceType Single 2> (Join-Path $TestDrive 'windows-schema-error.log')
                $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw (Join-Path $TestDrive 'windows-schema-error.log'))
                $windowsResource = $windowsResources | ConvertFrom-Json |
                    Where-Object -Property type -EQ 'TestSchemaResource/TestSchemaResource'
                $windowsResource.requireAdapter | Should -BeExactly 'Microsoft.Adapter/WindowsPowerShell'
                $windowsResource.schema.properties.Count.type | Should -BeExactly 'integer'
                $windowsResource.schema.properties.Names.items.type | Should -Contain 'string'
                $windowsSchema = $windowsResource.schema | ConvertTo-Json -Depth 100
                Test-Json -Json '{"name":"test","requiredValue":"value","ensure":"present"}' -Schema $windowsSchema -ErrorAction Stop |
                    Should -BeTrue
                $windowsNullable = $windowsResources | ConvertFrom-Json |
                    Where-Object -Property type -EQ 'TestSchemaResource/TestNullableResource'
                $windowsNullableSchema = $windowsNullable.schema | ConvertTo-Json -Depth 100
                Test-Json -Json '{"name":"test","requiredCount":null,"requiredEnsure":null,"choice":null,"choices":null}' -Schema $windowsNullableSchema -ErrorAction Stop |
                    Should -BeTrue
                Test-Json -Json '{"name":"test","requiredCount":1,"requiredEnsure":null,"enabled":null}' -Schema $windowsNullableSchema -ErrorAction Ignore |
                    Should -BeFalse
                Test-Json -Json '{"name":"test","requiredValue":"value","count":"bad"}' -Schema $windowsSchema -ErrorAction Ignore |
                    Should -BeFalse
                $scriptResource = $windowsResources | ConvertFrom-Json |
                    Where-Object -Property type -EQ 'TestScriptBaseDSC/CredentialValidation'
                $scriptResource.schema.properties.Credential.type | Should -Contain 'object'
                $scriptSchema = $scriptResource.schema | ConvertTo-Json -Depth 100
                Test-Json -Json '{"name":"test","credential":{"username":"user","password":"secret"}}' -Schema $scriptSchema -ErrorAction Stop |
                    Should -BeTrue
                Test-Json -Json '{"name":"test"}' -Schema $scriptSchema -ErrorAction Ignore |
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
        $null = dsc resource get -r 'TestSchemaResource/TestSchemaResource' -i '{"name":"test","requiredValue":"value","count":"bad"}' 2> (Join-Path $TestDrive 'invalid-error.log')
        $LASTEXITCODE | Should -Not -Be 0
        Get-Content -Raw (Join-Path $TestDrive 'invalid-error.log') | Should -Match 'integer'
    }

    It 'Returns schema-conforming canonical state for <Operation>' -ForEach @(
        @{ Operation = 'get'; State = 'actualState' }
        @{ Operation = 'test'; State = 'actualState' }
        @{ Operation = 'set'; State = 'afterState' }
    ) {
        $json = '{"name":"test","requiredValue":"value","selection":1,"choice":"One","ipAddress":"192.0.2.1","Resource-Name":"unchanged"}'
        $out = dsc resource $Operation -r 'TestSchemaResource/TestSchemaResource' -i $json 2> (Join-Path $TestDrive 'operation-error.log')
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw (Join-Path $TestDrive 'operation-error.log'))
        $actualState = ($out | ConvertFrom-Json).$State
        $actualState.PSObject.Properties.Name -ccontains 'name' | Should -BeTrue
        $actualState.PSObject.Properties.Name -ccontains 'ipAddress' | Should -BeTrue
        $actualState.'Resource-Name' | Should -BeExactly 'unchanged'
        $actualState.PSObject.Properties.Name | Should -Not -Contain 'options'
        Test-Json -Json ($actualState | ConvertTo-Json -Depth 100) -Schema $schema -ErrorAction Stop | Should -BeTrue
        if ($Operation -eq 'test') {
            $actualState._inDesiredState | Should -BeTrue
        }
    }

    It 'Preserves explicitly allowed nulls and omits unset optional nullable values in output' {
        $json = '{"name":"test","requiredCount":2,"requiredEnsure":null,"choice":"One"}'
        $out = dsc resource test -r 'TestSchemaResource/TestNullableResource' -i $json 2> (Join-Path $TestDrive 'nullable-output-error.log')
        $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw (Join-Path $TestDrive 'nullable-output-error.log'))
        $actualState = ($out | ConvertFrom-Json).actualState
        $propertyNames = $actualState.PSObject.Properties.Name
        $actualState.requiredCount | Should -Be 2
        foreach ($name in 'requiredEnsure', 'choices') {
            $propertyNames -ccontains $name | Should -BeTrue
            $actualState.$name | Should -BeNullOrEmpty
        }
        foreach ($name in 'enabled', 'selection', 'ensure') {
            $propertyNames | Should -Not -Contain $name
        }
        Test-Json -Json ($actualState | ConvertTo-Json -Depth 100) -Schema $nullableSchema -ErrorAction Stop | Should -BeTrue
    }
}
