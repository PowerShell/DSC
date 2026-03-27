# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

$ErrorActionPreference = 'Stop'

$script:AdaptedResourceSchemaUri = 'https://aka.ms/dsc/schemas/v3/bundled/adaptedresource/manifest.json'
$script:ResourceManifestSchemaUri = 'https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.json'
$script:JsonSchemaUri = 'https://json-schema.org/draft/2020-12/schema'
$script:DefaultAdapter = 'Microsoft.Adapter/PowerShell'

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

    [hashtable] ToHashtable() {
        return [ordered]@{
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
    }
}

class DscResourceManifestList {
    [System.Collections.Generic.List[hashtable]] $AdaptedResources
    [System.Collections.Generic.List[hashtable]] $Resources
    [System.Collections.Generic.List[hashtable]] $Extensions

    DscResourceManifestList() {
        $this.AdaptedResources = [System.Collections.Generic.List[hashtable]]::new()
        $this.Resources = [System.Collections.Generic.List[hashtable]]::new()
        $this.Extensions = [System.Collections.Generic.List[hashtable]]::new()
    }

    [void] AddAdaptedResource([DscAdaptedResourceManifest]$Manifest) {
        $this.AdaptedResources.Add($Manifest.ToHashtable())
    }

    [void] AddResource([hashtable]$Resource) {
        $this.Resources.Add($Resource)
    }

    [void] AddExtension([hashtable]$Extension) {
        $this.Extensions.Add($Extension)
    }

    [string] ToJson() {
        $result = [ordered]@{}

        if ($this.AdaptedResources.Count -gt 0) {
            $result['adaptedResources'] = @($this.AdaptedResources)
        }

        if ($this.Resources.Count -gt 0) {
            $result['resources'] = @($this.Resources)
        }

        if ($this.Extensions.Count -gt 0) {
            $result['extensions'] = @($this.Extensions)
        }

        return $result | ConvertTo-Json -Depth 15
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

    $resolvedPath = Resolve-Path -LiteralPath $Path
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

    $fileName = [System.IO.Path]::GetFileName($resolvedPath)

    return @{
        ModuleName  = $moduleName
        Version     = '0.0.1'
        Author      = ''
        Description = ''
        ScriptPath  = [string]$resolvedPath
        Psd1Path    = "$moduleName/$fileName"
        Directory   = $directory
    }
}

function ConvertPSObjectToHashtable {
    [CmdletBinding()]
    [OutputType([hashtable])]
    param(
        [Parameter(Mandatory)]
        [object]$InputObject
    )

    if ($InputObject -is [System.Collections.IDictionary]) {
        $result = [ordered]@{}
        foreach ($key in $InputObject.Keys) {
            $result[$key] = ConvertPSObjectToHashtable -InputObject $InputObject[$key]
        }
        return $result
    }

    if ($InputObject -is [PSCustomObject]) {
        $result = [ordered]@{}
        foreach ($property in $InputObject.PSObject.Properties) {
            $result[$property.Name] = ConvertPSObjectToHashtable -InputObject $property.Value
        }
        return $result
    }

    if ($InputObject -is [System.Collections.IList]) {
        $items = [System.Collections.Generic.List[object]]::new()
        foreach ($item in $InputObject) {
            $items.Add((ConvertPSObjectToHashtable -InputObject $item))
        }
        return @($items)
    }

    return $InputObject
}

function ConvertToAdaptedResourceManifest {
    [CmdletBinding()]
    [OutputType([DscAdaptedResourceManifest])]
    param(
        [Parameter(Mandatory)]
        [hashtable]$Hashtable
    )

    $manifest = [DscAdaptedResourceManifest]::new()
    $manifest.Schema = $Hashtable['$schema']
    $manifest.Type = $Hashtable['type']
    $manifest.Kind = if ($Hashtable.Contains('kind')) { $Hashtable['kind'] } else { 'resource' }
    $manifest.Version = $Hashtable['version']
    $manifest.Capabilities = if ($Hashtable.Contains('capabilities') -and $null -ne $Hashtable['capabilities']) { @($Hashtable['capabilities']) } else { [string[]]::new(0) }
    $manifest.Description = if ($Hashtable.Contains('description')) { [string]$Hashtable['description'] } else { '' }
    $manifest.Author = if ($Hashtable.Contains('author')) { [string]$Hashtable['author'] } else { '' }
    $manifest.RequireAdapter = $Hashtable['requireAdapter']
    $manifest.Path = if ($Hashtable.Contains('path')) { [string]$Hashtable['path'] } else { '' }

    $schemaData = $Hashtable['schema']
    if ($schemaData) {
        $embeddedSchema = if ($schemaData.Contains('embedded')) { $schemaData['embedded'] } else { $schemaData }
        $manifest.ManifestSchema = [DscAdaptedResourceManifestSchema]@{
            Embedded = $embeddedSchema
        }
    }

    return $manifest
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

<#
    .SYNOPSIS
        Creates a DSC resource manifests list for bundling multiple resources in a single file.

    .DESCRIPTION
        Builds a DscResourceManifestList object that can contain both adapted resources and
        command-based resources. The resulting object can be serialized to JSON and written
        to a `.dsc.manifests.json` file, which DSCv3 discovers and loads as a bundle.

        Adapted resources can be added by piping DscAdaptedResourceManifest objects from
        New-DscAdaptedResourceManifest. Command-based resources can be added via the
        -Resource parameter as hashtables matching the DSCv3 resource manifest schema.

    .PARAMETER AdaptedResource
        One or more DscAdaptedResourceManifest objects to include in the manifests list.
        These are typically produced by New-DscAdaptedResourceManifest.

    .PARAMETER Resource
        One or more hashtables representing command-based DSC resource manifests. Each
        hashtable should conform to the DSCv3 resource manifest schema with keys such as
        `$schema`, `type`, `version`, `get`, `set`, `test`, `schema`, etc.

    .EXAMPLE
        $adapted = New-DscAdaptedResourceManifest -Path ./MyModule/MyModule.psd1
        New-DscResourceManifest -AdaptedResource $adapted

        Creates a manifests list from adapted resource manifests generated from a module.

    .EXAMPLE
        $resource = @{
            '$schema'  = 'https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.json'
            type       = 'MyCompany/MyTool'
            version    = '1.0.0'
            get        = @{ executable = 'mytool'; args = @('get') }
            set        = @{ executable = 'mytool'; args = @('set'); implementsPretest = $false; return = 'state' }
            test       = @{ executable = 'mytool'; args = @('test'); return = 'state' }
            exitCodes  = @{ '0' = 'Success'; '1' = 'Error' }
            schema     = @{ command = @{ executable = 'mytool'; args = @('schema') } }
        }
        New-DscResourceManifest -Resource $resource

        Creates a manifests list containing a single command-based resource.

    .EXAMPLE
        $adapted = New-DscAdaptedResourceManifest -Path ./MyModule/MyModule.psd1
        $resource = @{
            '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.json'
            type      = 'MyCompany/MyTool'
            version   = '1.0.0'
            get       = @{ executable = 'mytool'; args = @('get') }
        }
        New-DscResourceManifest -AdaptedResource $adapted -Resource $resource

        Creates a manifests list combining both adapted and command-based resources.

    .EXAMPLE
        New-DscAdaptedResourceManifest -Path ./MyModule/MyModule.psd1 |
            New-DscResourceManifest

        Pipes adapted resource manifests directly into the function via the pipeline.

    .OUTPUTS
        Returns a DscResourceManifestList object with a .ToJson() method for serialization
        to the `.dsc.manifests.json` format.
#>
function New-DscResourceManifest {
    [CmdletBinding()]
    [OutputType([DscResourceManifestList])]
    param(
        [Parameter(ValueFromPipeline)]
        [DscAdaptedResourceManifest[]]$AdaptedResource,

        [Parameter()]
        [hashtable[]]$Resource
    )

    begin {
        $manifestList = [DscResourceManifestList]::new()

        if ($Resource) {
            foreach ($res in $Resource) {
                $manifestList.AddResource($res)
            }
        }
    }

    process {
        if ($AdaptedResource) {
            foreach ($adapted in $AdaptedResource) {
                $manifestList.AddAdaptedResource($adapted)
            }
        }
    }

    end {
        Write-Output $manifestList
    }
}

<#
    .SYNOPSIS
        Imports adapted resource manifest objects from `.dsc.adaptedResource.json` files.

    .DESCRIPTION
        Reads one or more `.dsc.adaptedResource.json` files and returns DscAdaptedResourceManifest
        objects. This is the inverse of serializing a manifest with `.ToJson()` — it allows you
        to load existing adapted resource manifests for inspection, modification, or inclusion
        in a resource manifest list via New-DscResourceManifest.

    .PARAMETER Path
        The path to a `.dsc.adaptedResource.json` file. Accepts pipeline input.

    .EXAMPLE
        Import-DscAdaptedResourceManifest -Path ./MyResource.dsc.adaptedResource.json

        Imports a single adapted resource manifest and returns a DscAdaptedResourceManifest object.

    .EXAMPLE
        Get-ChildItem -Filter *.dsc.adaptedResource.json | Import-DscAdaptedResourceManifest

        Imports all adapted resource manifest files in the current directory.

    .EXAMPLE
        Import-DscAdaptedResourceManifest -Path ./MyResource.dsc.adaptedResource.json |
            New-DscResourceManifest

        Imports an adapted resource manifest and bundles it into a resource manifest list.

    .OUTPUTS
        Returns a DscAdaptedResourceManifest object for each file. The object has .ToJson()
        and .ToHashtable() methods for serialization.
#>
function Import-DscAdaptedResourceManifest {
    [CmdletBinding()]
    [OutputType([DscAdaptedResourceManifest])]
    param(
        [Parameter(Mandatory, ValueFromPipeline, ValueFromPipelineByPropertyName)]
        [ValidateScript({
            if (-not (Test-Path -LiteralPath $_)) {
                throw "Path '$_' does not exist."
            }
            return $true
        })]
        [Alias('FullName')]
        [string]$Path
    )

    process {
        $resolvedPath = Resolve-Path -LiteralPath $Path
        Write-Verbose "Importing adapted resource manifest from '$resolvedPath'"

        $jsonContent = Get-Content -LiteralPath $resolvedPath -Raw
        $parsed = ConvertFrom-Json -InputObject $jsonContent
        $hashtable = ConvertPSObjectToHashtable -InputObject $parsed

        $manifest = ConvertToAdaptedResourceManifest -Hashtable $hashtable
        Write-Output $manifest
    }
}

<#
    .SYNOPSIS
        Imports a DSC resource manifest list from a `.dsc.manifests.json` file.

    .DESCRIPTION
        Reads a `.dsc.manifests.json` file and returns a DscResourceManifestList object
        containing the adapted resources, command-based resources, and extensions defined
        in the file. This is the inverse of serializing a manifest list with `.ToJson()`.

        The adapted resources in the returned list are hydrated into DscAdaptedResourceManifest
        objects and stored via AddAdaptedResource. Resources and extensions are stored as
        hashtables.

    .PARAMETER Path
        The path to a `.dsc.manifests.json` file. Accepts pipeline input.

    .EXAMPLE
        Import-DscResourceManifest -Path ./MyModule.dsc.manifests.json

        Imports a manifest list file and returns a DscResourceManifestList object.

    .EXAMPLE
        Get-ChildItem -Filter *.dsc.manifests.json | Import-DscResourceManifest

        Imports all manifest list files in the current directory.

    .EXAMPLE
        $list = Import-DscResourceManifest -Path ./existing.dsc.manifests.json
        $list.AdaptedResources.Count

        Imports a manifest list and inspects the number of adapted resources.

    .OUTPUTS
        Returns a DscResourceManifestList object with .ToJson() for serialization.
#>
function Import-DscResourceManifest {
    [CmdletBinding()]
    [OutputType([DscResourceManifestList])]
    param(
        [Parameter(Mandatory, ValueFromPipeline, ValueFromPipelineByPropertyName)]
        [ValidateScript({
            if (-not (Test-Path -LiteralPath $_)) {
                throw "Path '$_' does not exist."
            }
            return $true
        })]
        [Alias('FullName')]
        [string]$Path
    )

    process {
        $resolvedPath = Resolve-Path -LiteralPath $Path
        Write-Verbose "Importing resource manifest list from '$resolvedPath'"

        $jsonContent = Get-Content -LiteralPath $resolvedPath -Raw
        $parsed = ConvertFrom-Json -InputObject $jsonContent
        $hashtable = ConvertPSObjectToHashtable -InputObject $parsed

        $manifestList = [DscResourceManifestList]::new()

        if ($hashtable.Contains('adaptedResources')) {
            foreach ($ar in $hashtable['adaptedResources']) {
                $manifest = ConvertToAdaptedResourceManifest -Hashtable $ar
                $manifestList.AddAdaptedResource($manifest)
            }
        }

        if ($hashtable.Contains('resources')) {
            foreach ($res in $hashtable['resources']) {
                $manifestList.AddResource($res)
            }
        }

        if ($hashtable.Contains('extensions')) {
            foreach ($ext in $hashtable['extensions']) {
                $manifestList.AddExtension($ext)
            }
        }

        Write-Output $manifestList
    }
}

#endregion Public functions
