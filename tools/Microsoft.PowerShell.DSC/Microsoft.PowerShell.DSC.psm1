# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

$ErrorActionPreference = 'Stop'

$script:AdaptedResourceSchemaUri = 'https://aka.ms/dsc/schemas/v3/bundled/adaptedresource/manifest.json'
$script:JsonSchemaUri = 'https://json-schema.org/draft/2020-12/schema'
$script:DefaultAdapter = 'Microsoft.DSC/PowerShell'

#region Classes

class DscAdaptedResourceManifestSchema {
    [hashtable] $Embedded
}

class DscAdaptedResourceManifest {
    [string] $Schema
    [string] $Type
    [string] $Kind
    [string] $Version
    [string[]] $Capabilities
    [string] $Description
    [string] $Author
    [string] $RequireAdapter
    [string] $Path
    [DscAdaptedResourceManifestSchema] $ManifestSchema

    [string] ToJson() {
        $manifest = [ordered]@{
            '$schema'      = $this.Schema
            type           = $this.Type
            kind           = $this.Kind
            version        = $this.Version
            capabilities   = $this.Capabilities
            description    = $this.Description
            author         = $this.Author
            requireAdapter = $this.RequireAdapter
            path           = $this.Path
            schema         = [ordered]@{
                embedded = $this.ManifestSchema.Embedded
            }
        }
        return $manifest | ConvertTo-Json -Depth 10
    }
}

#endregion Classes

#region Private functions

function GetDscResourceTypeDefinition {
    [CmdletBinding()]
    [OutputType([System.Collections.Generic.List[hashtable]])]
    param(
        [Parameter(Mandatory)]
        [string]$Path
    )

    [System.Management.Automation.Language.Token[]] $tokens = $null
    [System.Management.Automation.Language.ParseError[]] $errors = $null
    $ast = [System.Management.Automation.Language.Parser]::ParseFile($Path, [ref]$tokens, [ref]$errors)

    foreach ($e in $errors) {
        Write-Error "Parse error in '$Path': $($e.Message)"
    }

    $allTypeDefinitions = $ast.FindAll(
        {
            $typeAst = $args[0] -as [System.Management.Automation.Language.TypeDefinitionAst]
            return $null -ne $typeAst
        },
        $false
    )

    $results = [System.Collections.Generic.List[hashtable]]::new()

    foreach ($typeDefinition in $allTypeDefinitions) {
        foreach ($attribute in $typeDefinition.Attributes) {
            if ($attribute.TypeName.Name -eq 'DscResource') {
                $results.Add(@{
                    TypeDefinitionAst  = $typeDefinition
                    AllTypeDefinitions = $allTypeDefinitions
                })
                break
            }
        }
    }

    return $results
}

function GetDscResourceCapability {
    [CmdletBinding()]
    [OutputType([string[]])]
    param(
        [Parameter(Mandatory)]
        [System.Management.Automation.Language.MemberAst[]]$MemberAst
    )

    $capabilities = [System.Collections.Generic.List[string]]::new()
    $availableMethods = @('get', 'set', 'setHandlesExist', 'whatIf', 'test', 'delete', 'export')
    $methods = $MemberAst | Where-Object {
        $_ -is [System.Management.Automation.Language.FunctionMemberAst] -and $_.Name -in $availableMethods
    }

    foreach ($method in $methods.Name) {
        switch ($method) {
            'Get' { $capabilities.Add('get') }
            'Set' { $capabilities.Add('set') }
            'Test' { $capabilities.Add('test') }
            'WhatIf' { $capabilities.Add('whatIf') }
            'SetHandlesExist' { $capabilities.Add('setHandlesExist') }
            'Delete' { $capabilities.Add('delete') }
            'Export' { $capabilities.Add('export') }
        }
    }

    return ($capabilities | Select-Object -Unique)
}

function GetDscResourceProperty {
    [CmdletBinding()]
    [OutputType([System.Collections.Generic.List[hashtable]])]
    param(
        [Parameter(Mandatory)]
        [System.Management.Automation.Language.TypeDefinitionAst[]]$AllTypeDefinitions,

        [Parameter(Mandatory)]
        [System.Management.Automation.Language.TypeDefinitionAst]$TypeDefinitionAst
    )

    $properties = [System.Collections.Generic.List[hashtable]]::new()
    CollectAstProperty -AllTypeDefinitions $AllTypeDefinitions -TypeAst $TypeDefinitionAst -Properties $properties
    return , $properties
}

function CollectAstProperty {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]
        [System.Management.Automation.Language.TypeDefinitionAst[]]$AllTypeDefinitions,

        [Parameter(Mandatory)]
        [System.Management.Automation.Language.TypeDefinitionAst]$TypeAst,

        [Parameter(Mandatory)]
        [AllowEmptyCollection()]
        [System.Collections.Generic.List[hashtable]]$Properties
    )

    foreach ($typeConstraint in $TypeAst.BaseTypes) {
        $baseType = $AllTypeDefinitions | Where-Object { $_.Name -eq $typeConstraint.TypeName.Name }
        if ($baseType) {
            CollectAstProperty -AllTypeDefinitions $AllTypeDefinitions -TypeAst $baseType -Properties $Properties
        }
    }

    foreach ($member in $TypeAst.Members) {
        $propertyAst = $member -as [System.Management.Automation.Language.PropertyMemberAst]
        if (($null -eq $propertyAst) -or ($propertyAst.IsStatic)) {
            continue
        }

        $isDscProperty = $false
        $isKey = $false
        $isMandatory = $false
        foreach ($attr in $propertyAst.Attributes) {
            if ($attr.TypeName.Name -eq 'DscProperty') {
                $isDscProperty = $true
                foreach ($namedArg in $attr.NamedArguments) {
                    switch ($namedArg.ArgumentName) {
                        'Key' { $isKey = $true }
                        'Mandatory' { $isMandatory = $true }
                    }
                }
            }
        }

        if (-not $isDscProperty) {
            continue
        }

        $typeName = if ($propertyAst.PropertyType) {
            $propertyAst.PropertyType.TypeName.Name
        } else {
            'string'
        }

        # check if the type is an enum defined in the same file
        $enumValues = $null
        $enumAst = $AllTypeDefinitions | Where-Object {
            $_.Name -eq $typeName -and $_.IsEnum
        }
        if ($enumAst) {
            $enumValues = @($enumAst.Members | ForEach-Object { $_.Name })
        }

        $Properties.Add(@{
            Name        = $propertyAst.Name
            TypeName    = $typeName
            IsKey       = $isKey
            IsMandatory = $isMandatory -or $isKey
            EnumValues  = $enumValues
        })
    }
}

function ConvertToJsonSchemaType {
    [CmdletBinding()]
    [OutputType([hashtable])]
    param(
        [Parameter(Mandatory)]
        [string]$TypeName
    )

    switch ($TypeName) {
        'string'   { return @{ type = 'string' } }
        'int'      { return @{ type = 'integer' } }
        'int32'    { return @{ type = 'integer' } }
        'int64'    { return @{ type = 'integer' } }
        'long'     { return @{ type = 'integer' } }
        'double'   { return @{ type = 'number' } }
        'float'    { return @{ type = 'number' } }
        'single'   { return @{ type = 'number' } }
        'decimal'  { return @{ type = 'number' } }
        'bool'     { return @{ type = 'boolean' } }
        'boolean'  { return @{ type = 'boolean' } }
        'switch'   { return @{ type = 'boolean' } }
        'hashtable' { return @{ type = 'object' } }
        'datetime' { return @{ type = 'string'; format = 'date-time' } }
        default {
            # arrays like string[] or int[]
            if ($TypeName -match '^(.+)\[\]$') {
                $innerType = ConvertToJsonSchemaType -TypeName $Matches[1]
                return @{ type = 'array'; items = $innerType }
            }
            # default to string for unknown types
            return @{ type = 'string' }
        }
    }
}

function BuildEmbeddedJsonSchema {
    [CmdletBinding()]
    [OutputType([ordered])]
    param(
        [Parameter(Mandatory)]
        [string]$ResourceName,

        [Parameter(Mandatory)]
        [AllowEmptyCollection()]
        [System.Collections.Generic.List[hashtable]]$Properties,

        [Parameter()]
        [string]$Description
    )

    $schemaProperties = [ordered]@{}
    $requiredList = [System.Collections.Generic.List[string]]::new()

    foreach ($prop in $Properties) {
        $schemaProp = [ordered]@{}

        if ($prop.EnumValues) {
            $schemaProp['type'] = 'string'
            $schemaProp['enum'] = $prop.EnumValues
        } else {
            $jsonType = ConvertToJsonSchemaType -TypeName $prop.TypeName
            foreach ($key in $jsonType.Keys) {
                $schemaProp[$key] = $jsonType[$key]
            }
        }

        $schemaProp['title'] = $prop.Name
        $schemaProp['description'] = "The $($prop.Name) property."
        $schemaProperties[$prop.Name] = $schemaProp

        if ($prop.IsMandatory) {
            $requiredList.Add($prop.Name)
        }
    }

    $schema = [ordered]@{
        '$schema'            = $script:JsonSchemaUri
        title                = $ResourceName
        type                 = 'object'
        required             = @($requiredList)
        additionalProperties = $false
        properties           = $schemaProperties
    }

    if (-not [string]::IsNullOrEmpty($Description)) {
        $schema['description'] = $Description
    }

    return $schema
}

function ResolveModuleInfo {
    [CmdletBinding()]
    [OutputType([hashtable])]
    param(
        [Parameter(Mandatory)]
        [string]$Path
    )

    $resolvedPath = Resolve-Path -Path $Path
    $extension = [System.IO.Path]::GetExtension($resolvedPath)
    $directory = [System.IO.Path]::GetDirectoryName($resolvedPath)

    if ($extension -eq '.psd1') {
        $manifestData = Import-PowerShellDataFile -Path $resolvedPath
        $moduleName = [System.IO.Path]::GetFileNameWithoutExtension($resolvedPath)
        $version = if ($manifestData.ModuleVersion) { $manifestData.ModuleVersion } else { '0.0.1' }
        $author = if ($manifestData.Author) { $manifestData.Author } else { '' }
        $description = if ($manifestData.Description) { $manifestData.Description } else { '' }

        $rootModule = $manifestData.RootModule
        if ([string]::IsNullOrEmpty($rootModule)) {
            $rootModule = "$moduleName.psm1"
        }
        $scriptPath = Join-Path $directory $rootModule
        $psd1RelativePath = "$moduleName/$([System.IO.Path]::GetFileName($resolvedPath))"

        return @{
            ModuleName  = $moduleName
            Version     = $version
            Author      = $author
            Description = $description
            ScriptPath  = $scriptPath
            Psd1Path    = $psd1RelativePath
            Directory   = $directory
        }
    }

    # derive fileName from .ps1 or .psm1
    $moduleName = [System.IO.Path]::GetFileNameWithoutExtension($resolvedPath)

    # validate if .psd1 is there and use that
    $psd1Path = Join-Path $directory "$moduleName.psd1"
    if (Test-Path -LiteralPath $psd1Path) {
        return ResolveModuleInfo -Path $psd1Path
    }

    return @{
        ModuleName  = $moduleName
        Version     = '0.0.1'
        Author      = ''
        Description = ''
        ScriptPath  = [string]$resolvedPath
        Psd1Path    = "$moduleName/$moduleName.psd1"
        Directory   = $directory
    }
}

#endregion Private functions

#region Public functions

<#
    .SYNOPSIS
        Creates adapted resource manifest objects from class-based PowerShell DSC resources.

    .DESCRIPTION
        Parses the AST of a PowerShell file (.ps1, .psm1, or .psd1) to find class-based DSC
        resources decorated with the [DscResource()] attribute. For each resource found, it
        returns a DscAdaptedResourceManifest object that complies with the DSCv3 adapted
        resource manifest JSON schema.

        The returned objects can be serialized to JSON using the .ToJson() method and written
        to `.dsc.adaptedResource.json` files. These manifests enable DSCv3 to discover and
        use PowerShell DSC resources without running Invoke-DscCacheRefresh.

    .PARAMETER Path
        The path to a .ps1, .psm1, or .psd1 file containing class-based DSC resources.
        When a .psd1 is provided, the RootModule is resolved and parsed automatically.

    .EXAMPLE
        New-DscAdaptedResourceManifest -Path ./MyModule/MyModule.psd1

        Returns adapted resource manifest objects for all class-based DSC resources in the module.

    .EXAMPLE
        New-DscAdaptedResourceManifest -Path ./MyResource.ps1 | ForEach-Object {
            $_.ToJson() | Set-Content "$($_.Type -replace '/', '.').dsc.adaptedResource.json"
        }

        Generates manifest objects and writes each to a JSON file.

    .EXAMPLE
        Get-ChildItem -Path ./MyModules -Filter *.psd1 -Recurse | New-DscAdaptedResourceManifest

        Discovers all module manifests under `./MyModules` and pipes them into the function
        to generate adapted resource manifests for every class-based DSC resource found.

    .OUTPUTS
        Returns a DscAdaptedResourceManifest object for each class-based DSC resource found.
        The object has a .ToJson() method for serialization to the adapted resource manifest
        JSON format.
#>
function New-DscAdaptedResourceManifest {
    [CmdletBinding()]
    [OutputType([DscAdaptedResourceManifest])]
    param(
        [Parameter(Mandatory, ValueFromPipeline, ValueFromPipelineByPropertyName)]
        [ValidateScript({
            if (-not (Test-Path -LiteralPath $_)) {
                throw "Path '$_' does not exist."
            }
            $ext = [System.IO.Path]::GetExtension($_)
            if ($ext -notin '.ps1', '.psm1', '.psd1') {
                throw "Path '$_' must be a .ps1, .psm1, or .psd1 file."
            }
            return $true
        })]
        [string]$Path
    )

    process {
        $moduleInfo = ResolveModuleInfo -Path $Path

        if (-not (Test-Path -LiteralPath $moduleInfo.ScriptPath)) {
            Write-Error "Cannot find script file '$($moduleInfo.ScriptPath)' to parse."
            return
        }

        $dscTypes = GetDscResourceTypeDefinition -Path $moduleInfo.ScriptPath

        if ($dscTypes.Count -eq 0) {
            Write-Warning "No class-based DSC resources found in '$Path'."
            return
        }

        foreach ($entry in $dscTypes) {
            $typeDefinitionAst = $entry.TypeDefinitionAst
            $allTypeDefinitions = $entry.AllTypeDefinitions
            $resourceName = $typeDefinitionAst.Name
            $resourceType = "$($moduleInfo.ModuleName)/$resourceName"

            Write-Verbose "Processing DSC resource '$resourceType'"

            $capabilities = GetDscResourceCapability -MemberAst $typeDefinitionAst.Members
            $properties = GetDscResourceProperty -AllTypeDefinitions $allTypeDefinitions -TypeDefinitionAst $typeDefinitionAst
            $embeddedSchema = BuildEmbeddedJsonSchema -ResourceName $resourceType -Properties $properties -Description $moduleInfo.Description

            $manifest = [DscAdaptedResourceManifest]::new()
            $manifest.Schema = $script:AdaptedResourceSchemaUri
            $manifest.Type = $resourceType
            $manifest.Kind = 'resource'
            $manifest.Version = $moduleInfo.Version
            $manifest.Capabilities = @($capabilities)
            $manifest.Description = $moduleInfo.Description
            $manifest.Author = $moduleInfo.Author
            $manifest.RequireAdapter = $script:DefaultAdapter
            $manifest.Path = $moduleInfo.Psd1Path
            $manifest.ManifestSchema = [DscAdaptedResourceManifestSchema]@{
                Embedded = $embeddedSchema
            }

            Write-Output $manifest
        }
    }
}

#endregion Public functions
