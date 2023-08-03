#requires -Modules yayaml
using namespace System.Collections

<#
    .SYNOPSIS
#>

[cmdletbinding(DefaultParameterSetName='ByConfig')]
param(
    [string]
    $OutputDirectory = "$PSScriptRoot",
    
    [Parameter(ParameterSetName='ByPath')]
    [string[]]
    $ConfigFilePath,
    
    [string[]]
    [ValidateSet('Json', 'JsonVSCode', 'Yaml', 'YamlVSCode')]
    $OutputFormat = @(
        'Json'
        'JsonVSCode'
        'Yaml'
        'YamlVSCode'
    )
)

begin {
    class LocalJsonSchemaRegistry {
        [Specialized.OrderedDictionary]
        $Map
        
        [Generic.List[Specialized.OrderedDictionary]]
        $List

        [string]$SchemaHost
        [string]$SchemaPrefix
        [string]$SchemaVersion

        LocalJsonSchemaRegistry() {
            $this.Map = [Specialized.OrderedDictionary]::new()
            $this.List = [Generic.List[Specialized.OrderedDictionary]]::new()
        }

        LocalJsonSchemaRegistry(
            [string]$SchemaHost,
            [string]$SchemaPrefix,
            [string]$SchemaVersion
        ) {
            $this.SchemaHost    = $SchemaHost
            $this.SchemaPrefix  = $SchemaPrefix
            $this.SchemaVersion = $SchemaVersion
            $this.Map           = [Specialized.OrderedDictionary]::new()
            $this.List          = [Generic.List[Specialized.OrderedDictionary]]::new()
        }

    }

    function Remove-JsonSchemaKey {
        [cmdletbinding(DefaultParameterSetName='SchemaObject')]
        [OutputType([Specialized.OrderedDictionary])]
        [OutputType([Object[]])]
        param(
            [parameter(ParameterSetName='SchemaObject')]
            [Specialized.OrderedDictionary]
            $Schema,

            [parameter(ParameterSetName='SchemaList', DontShow)]
            [Object[]]
            $SchemaList,

            [string]$KeyName
        )

        process {
            if ($PSCmdlet.ParameterSetName -eq 'SchemaObject') {
                $MungedSchema = [Specialized.OrderedDictionary]::new()

                $Schema.GetEnumerator().Where({$_.Key -ne $KeyName}).ForEach({
                    if ($_.Value -is [Object[]]) {
                        $MungedKeyValue = Remove-JsonSchemaKey -KeyName $KeyName -SchemaList $_.Value
                        $MungedSchema.Add($_.Key, $MungedKeyValue)
                    } elseif ($_.Value -is [Specialized.OrderedDictionary]) {
                        $MungedKeyValue = Remove-JsonSchemaKey -KeyName $KeyName -Schema $_.Value
                        $MungedSchema.Add($_.Key, $MungedKeyValue)
                    } else {
                        $MungedSchema.Add($_.Key, $_.Value)
                    }
                })

                return $MungedSchema
            }

            if ($PSCmdlet.ParameterSetName -eq 'SchemaList') {
                [Object[]]$MungedArrayValue = @()

                $SchemaList.ForEach({
                    if ($_ -is [Object[]]) {
                        $MungedArrayValue += Remove-JsonSchemaKey -KeyName $KeyName -SchemaList $_
                    } elseif ($_ -is [Specialized.OrderedDictionary]) {
                        $MungedArrayValue += Remove-JsonSchemaKey -KeyName $KeyName -Schema $_
                    } else {
                        $MungedArrayValue += $_
                    }
                })

                return $MungedArrayValue
            }
        }
    }

    function Get-LocalJsonSchemaRegistry {
        [CmdletBinding()]
        [OutputType([LocalJsonSchemaRegistry])]
        param(
            [switch]$WithoutExamples,
            [switch]$WithoutComments,
            [string[]]$SchemaDirectories = @(),
            [string]$SchemaHost          = 'https://raw.githubusercontent.com',
            [string]$SchemaPrefix        = 'PowerShell/DSC/main',
            [string]$SchemaVersion       = '2023/08'
        )

        begin {
            $Info = [LocalJsonSchemaRegistry]::new($SchemaHost, $SchemaPrefix, $SchemaVersion)
        }

        process {
            Get-ChildItem -Path $SchemaDirectories -Recurse -File
            | Where-Object -Property Extension -in -Value @('.json', '.yaml', '.yml')
            | ForEach-Object -process {
                $Schema = Get-Content $_ -Raw | yayaml\ConvertFrom-Yaml
                if ($AddDocsUrl) {
                    Write-Warning 'Not implemented yet'
                }
                if ($MakeStrict) {
                    Write-Warning 'Not implemented yet'
                }
                if ($WithoutComments) {
                    $Schema = Remove-JsonSchemaKey -Schema $Schema -KeyName '$comment'
                }
                if ($WithoutExamples) {
                    $Schema = Remove-JsonSchemaKey -Schema $Schema -KeyName 'examples'
                }
                if ($SchemaID = $Schema.'$id') {
                    $SchemaRefID = $SchemaID -replace $SchemaHost, ''
                    $Info.List.Add($Schema)
                    $Info.Map.Add($SchemaID, $Schema)
                    $Info.Map.Add($SchemaRefID, $Schema)
                }
            }

            $Info
        }
    }

    function Get-JsonSchemaReference {
        <#
        #>
        [cmdletbinding(DefaultParameterSetName='SchemaObject')]
        [OutputType([Generic.List[string]])]
        param(
            [parameter(ParameterSetName='SchemaObject')]
            [Specialized.OrderedDictionary]
            $Schema,

            [parameter(ParameterSetName='SchemaList', DontShow)]
            [Object[]]
            $SchemaList,

            [LocalJsonSchemaRegistry] $SchemaRegistry
        )

        begin {
            $References = [Generic.List[string]]::new()
            $AddNestedReference = {
                if ($_ -notin $References) {
                    $References.Add($_)
                }
            }
        }

        process {
            if ($PSCmdlet.ParameterSetName -eq 'SchemaObject') {
                $Schema.GetEnumerator().ForEach({
                    if ($_.Key -eq '$ref' -and $_.Value -notin $References) {
                        $References.Add($_.Value)
                    } elseif ($_.Value -is [Object[]]) {
                        $NestedReferences = Get-JsonSchemaReference -SchemaList $_.Value
                        $NestedReferences.ForEach($AddNestedReference)
                    } elseif ($_.Value -is [Specialized.OrderedDictionary]) {
                        $NestedReferences = Get-JsonSchemaReference -Schema $_.Value
                        $NestedReferences.ForEach($AddNestedReference)
                    }
                })
            }

            if ($PSCmdlet.ParameterSetName -eq 'SchemaList') {
                $SchemaList.ForEach({
                    if ($_ -is [Object[]]) {
                        $NestedReferences = Get-JsonSchemaReference -SchemaList $_
                        $NestedReferences.ForEach($AddNestedReference)
                    } elseif ($_ -is [Specialized.OrderedDictionary]) {
                        $NestedReferences = Get-JsonSchemaReference -Schema $_
                        $NestedReferences.ForEach($AddNestedReference)
                    }
                })
            }

            if ($null -ne $SchemaRegistry -and $References.Count -gt 0) {
                foreach ($Reference in $References.Clone()) {
                    if ($Reference -in $SchemaRegistry.Map.Keys) {
                        $Resolving = @{
                            Schema = $SchemaRegistry.Map.$Reference
                            SchemaRegistry = $SchemaRegistry
                        }
                        $NestedReferences = Get-JsonSchemaReference @Resolving
                        $NestedReferences.ForEach($AddNestedReference)
                    }
                }
            }

            $References
        }
    }

    function Merge-JsonSchema {
        <#
        #>
        [cmdletbinding(DefaultParameterSetName='FromPreset')]
        [OutputType([Specialized.OrderedDictionary])]
        param(
            [Parameter(ParameterSetName='FromPath', Mandatory)]
            [string]
            $Path,
            
            [Parameter(ParameterSetName='FromSchema', Mandatory)]
            [Specialized.OrderedDictionary]
            $Schema,
            
            [Parameter(ParameterSetName='FromPreset', Mandatory)]
            [ValidateSet('ConfigDocument', 'ResourceManifest')]
            [string]
            $Preset,

            [LocalJsonSchemaRegistry] $SchemaRegistry,
            
            [switch]$ForVSCode,
            [switch]$WithoutComments,
            [switch]$WithoutExamples
        )

        begin {
            if ($null -eq $SchemaRegistry) {
                $SchemaRegistry = Get-LocalJsonSchemaRegistry
            }

            $Schema = [Specialized.OrderedDictionary]::new()
            $References = [Generic.List[string]]::new()
            $RelativeUriReferencePattern = @(
                '(?m)'
                '^'
                '(?<Prefix>\s+\$ref:\s+)'
                '(?<Reference>/.+)'
                '$'
            ) -join ''
        }

        process {
            if ($PSCmdlet.ParameterSetName -eq 'FromPath') {
                $Schema = Get-Content -Path $Path -Raw | yayaml\ConvertFrom-Yaml
            }
            if ($PSCmdlet.ParameterSetName -eq 'FromPreset') {
                switch ($Preset) {
                    'ConfigDocument'    {
                        $Schema = $SchemaRegistry.Map.'/dsc/2023/07/config/document.yaml'
                    }
                    'ResourceManifest'  {
                        $Schema = $SchemaRegistry.Map.'/dsc/2023/07/resource/manifest.yaml'
                    }
                }
            }

            $ID = $Schema.'$id'

            $MergedSchema = $Schema | yayaml\ConvertTo-Yaml -Depth 99 | yayaml\ConvertFrom-Yaml

            $References = Get-JsonSchemaReference -Schema $Schema -SchemaRegistry $SchemaRegistry

            if ($ForVSCode) {
                if ('$defs' -notin $MergedSchema.Keys) {
                    $MergedSchema.Add('$defs', [Specialized.OrderedDictionary]::new())
                }

                foreach ($Reference in $References) {
                    $ReferenceSchema = $SchemaRegistry.Map.$Reference
                    if ($null -eq $ReferenceSchema -and $Reference -match '#\/') {
                        Write-Verbose "$ID`n`tSkipping local reference: '$Reference'"
                        continue
                    }

                    if ($null -eq $ReferenceSchema -or $Reference -match '^https?:\/\/') {
                        Write-Verbose "$ID`n`tSkipping apparent remote reference: '$Reference'"
                        continue
                    }

                    $ReferenceSegments = $Reference.Trim('/') -split '/'
                    $Working = $MergedSchema.'$defs'
                    
                    for ($i = 0; $i -lt $ReferenceSegments.Count; $i++) {
                        $Segment = $ReferenceSegments[$i]

                        # Segment dictionary already exists
                        if ($Segment -in $Working.Keys) {
                            $Working = $Working.$Segment
                            continue
                        }
                        
                        # Add an empty dictionary for non-final segments
                        if ($i -ne ($ReferenceSegments.Count - 1)) {
                            $Working.Add($Segment, [Specialized.OrderedDictionary]::new())
                            $Working = $Working.$Segment
                            continue
                        }

                        # Add the referenced schema
                        $Working.Add($Segment, $ReferenceSchema)
                    }

                }

                $MungingSchema = $MergedSchema | yayaml\ConvertTo-Yaml -Depth 99
                $MungingSchema
                | Select-String -Pattern $RelativeUriReferencePattern -AllMatches
                | Select-Object -ExpandProperty Matches
                | ForEach-Object -Process {
                    $Whole = $_.Groups
                    | Where-Object { $_.Name -eq '0' }
                    | Select-Object -ExpandProperty Value
                    $Prefix = $_.Groups
                    | Where-Object { $_.Name -eq 'Prefix' }
                    | Select-Object -ExpandProperty Value
                    $RefUri = $_.Groups
                    | Where-Object { $_.Name -eq 'Reference' }
                    | Select-Object -ExpandProperty Value
                    $NewValue = @(
                        $Prefix
                        "'"
                        '#/$defs'
                        $RefUri.Trim()
                        "'"
                    ) -join ''
                    $MungingSchema = $MungingSchema -replace [regex]::Escape($Whole), $NewValue
                }
                $MergedSchema = $MungingSchema | yayaml\ConvertFrom-Yaml
            } else {
                if ('$defs' -notin $MergedSchema.Keys) {
                    $MergedSchema.Add('$defs', [Specialized.OrderedDictionary]::new())
                }

                foreach ($Reference in $References) {
                    $ReferenceSchema = $SchemaRegistry.Map.$Reference
                    if ($null -eq $ReferenceSchema -and $Reference -match '^#\/') {
                        Write-Verbose "$ID`n`tSkipping local reference: '$Reference'"
                        continue
                    }

                    if ($null -eq $ReferenceSchema) {
                        Write-Verbose "$ID`n`tSkipping apparent remote reference: '$Reference'"
                        continue
                    }

                    if ($Reference -notin $Schema.'$defs'.Keys) {
                        Write-Verbose "$ID`n`tAdding reference to `$defs: '$Reference'"
                        $MergedSchema.'$defs'.Add($ReferenceSchema.'$id', $ReferenceSchema)
                    }
                }
            }

            if ($WithoutComments) {
                $MergedSchema = Remove-JsonSchemaKey -Schema $MergedSchema -KeyName '$comment'
            }
            if ($WithoutExamples) {
                $MergedSchema = Remove-JsonSchemaKey -Schema $MergedSchema -KeyName 'examples'
            }

            $MergedSchema
        }
    }

    function ConvertTo-MergedAndMungedJson {
        [CmdletBinding()]
        [OutputType([string])]
        param(
            [Parameter(Mandatory, ValueFromPipeline)]
            [Specialized.OrderedDictionary]
            $InputObject
        )

        process {
            ($InputObject | ConvertTo-Json -Depth 99) -replace '\b(\w+\.)yaml', '$1json'
        }
    }

    function Set-BundledSchemaID {
        [CmdletBinding()]
        [OutputType([Specialized.OrderedDictionary])]
        param(
            [Parameter(Mandatory, ValueFromPipeline)]
            [Specialized.OrderedDictionary]
            $InputObject,
            [Parameter(Mandatory)]
            [string]
            $BundledName,
            [string]
            $SchemaHost = 'https://raw.githubusercontent.com',
            [string]
            $SchemaPrefix = 'PowerShell/DSC/main'
        )

        begin {
            $ReplaceIDPattern = @(
                '^'
                '('
                    [regex]::Escape("$SchemaHost/$SchemaPrefix")
                    '\d+\/\d+\/'
                ')'
                '.+\.yaml'
                '$'
            ) -join ''
            $ReplaceIDValue = "`$1bundled/$BundledName.yaml"
        }
        process {
            $ID = $InputObject.'$id' -replace $ReplaceIDPattern, $ReplaceIDValue
            $InputObject.'$id' = $ID
            $InputObject
        }
    }

    function Export-MergedJsonSchema {
        param (
            [Parameter(Mandatory)]
            [string]
            $ConfigFilePath,

            [string]
            $Name,

            [string]
            $OutputDirectory = $PWD,
            
            [string[]]
            [ValidateSet('Json', 'JsonVSCode', 'Yaml', 'YamlVSCode')]
            $OutputFormat = @(
                'Json'
            ),

            [LocalJsonSchemaRegistry] $SchemaRegistry
        )

        begin {
            $MergeForNormal = $OutputFormat.Where({$_ -notmatch 'VSCode'}).Count -gt 0
            $MergeForVSCode = $OutputFormat.Where({$_ -match 'VSCode'}).Count -gt 0
            if (-not (Test-Path -Path $OutputDirectory)) {
                $null = New-Item -Path $OutputDirectory -ItemType Directory -Force
            }
        }

        process {
            if ([string]::IsNullOrEmpty($Name)) {
                $ConfigFileInfo = Get-Item -Path $ConfigFilePath
                $Name = $ConfigFileInfo.BaseName
            }

            $OutputPathPrefix = "$OutputDirectory/$Name"

            $SharedMergeParams = @{
                Path           = $ConfigFilePath
                SchemaRegistry = $SchemaRegistry
            }
            
            if ($MergeForNormal) {
                $Bundled = Merge-JsonSchema @SharedMergeParams
                | Set-BundledSchemaID -BundledName $Name

                if ($OutputFormat -contains 'json') {
                    $Bundled
                    | ConvertTo-MergedAndMungedJson
                    | ForEach-Object { $_ -replace '\r\n', "`n" }
                    | Out-File -FilePath "$OutputPathPrefix.json"
                }

                if ($OutputFormat -contains 'yaml') {
                    $Bundled
                    | yayaml\ConvertTo-Yaml -Depth 99
                    | ForEach-Object { $_ -replace '\r\n', "`n" }
                    | Out-File -FilePath "$OutputPathPrefix.yaml"
                }
            }
            if ($MergeForVSCode) {
                $Bundled = Merge-JsonSchema @SharedMergeParams -ForVSCode
                | Set-BundledSchemaID -BundledName $Name

                if ($OutputFormat -contains 'jsonVSCode') {
                    $Bundled
                    | ConvertTo-MergedAndMungedJson
                    | ForEach-Object { $_ -replace '\r\n', "`n" }
                    | Out-File -FilePath "$OutputPathPrefix.vscode.json"
                }

                if ($OutputFormat -contains 'yamlVSCode') {
                    $Bundled
                    | yayaml\ConvertTo-Yaml -Depth 99
                    | ForEach-Object { $_ -replace '\r\n', "`n" }
                    | Out-File -FilePath "$OutputPathPrefix.vscode.yaml" -Force:$Force
                }
            }
        }
    }
}

process {
    $Config  = Get-Content -Path $PSScriptRoot/schemas.config.yaml | yayaml\ConvertFrom-Yaml

    if (-not $PSBoundParameters.ContainsKey('OutputDirectory')) {
        $OutputDirectory = "$PSScriptRoot/$($Config.version)"
    }

    if (-not (Test-Path -Path $OutputDirectory)) {
        $null = New-Item -Path $OutputDirectory -ItemType Directory -Force
    }

    Get-ChildItem -Path $PSScriptRoot/src -Filter *.yaml -Recurse | ForEach-Object -Process {
        $SchemaContent = Get-Content -Path $_.FullName -Raw
        $SchemaContent = $SchemaContent -replace '<HOST>',          $Config.host
        $SchemaContent = $SchemaContent -replace '<PREFIX>',        $Config.prefix
        $SchemaContent = $SchemaContent -replace '<VERSION>',       $Config.version
        $SchemaContent = $SchemaContent -replace '(?m)\.yaml$"?,?', '.json'
        $SchemaPath    = $_.FullName    -replace 'src',             $Config.version
        $SchemaFolder  = Split-Path -Parent $SchemaPath
        if (-not (Test-Path -Path ($SchemaFolder))) {
            $null = New-Item -Path $SchemaFolder -ItemType Directory -Force
        }

        $SchemaContent
        | yayaml\ConvertFrom-Yaml
        | ConvertTo-Json -Depth 99
        | ForEach-Object { $_ -replace '\r\n', "`n" }
        | Out-File -FilePath ($SchemaPath -replace '\.yaml$', '.json') -Force
    }

    $RegistryParameters = @{
        SchemaDirectories = @(
            "$OutputDirectory/config"
            "$OutputDirectory/definitions"
            "$OutputDirectory/outputs"
            "$OutputDirectory/resource"
        )
        SchemaHost       = $Config.host
        SchemaPrefix     = $Config.prefix
        SchemaVersion    = $Config.version
        WithoutExamples  = $true
        WithoutComments  = $true
    }
    $SchemaRegistry = Get-LocalJsonSchemaRegistry @RegistryParameters

    $SchemaRegistry

    $Bundles = $Config.bundle_schemas | ForEach-Object -Process {
        [hashtable]$Bundle     = $_
        $Bundle.ConfigFilePath = "$OutputDirectory/$($Bundle.ConfigFilePath)"
        $Bundle
    }

    if ($Bundles.Count -eq 0) {
        $Bundles = @(
            { ConfigFilePath = "$OutputDirectory/config/document.json" }
            { ConfigFilePath = "$OutputDirectory/resource/manifest.json" }
        )
    }

    foreach ($BundleToExport in $Bundles) {
        if ($null -eq $BundleToExport.OutputDirectory) {
            $BundleToExport.OutputDirectory = "$OutputDirectory/bundled"
        } else {
            $BundleToExport.OutputDirectory = "$OutputDirectory/$($BundleToExport.OutputDirectory)"
        }
        if ($null -eq $BundleToExport.OutputFormat) {
            $BundleToExport.OutputFormat = $OutputFormat
        }
        Write-Verbose "Exporting: $($BundleToExport | ConvertTo-Json)"
        Export-MergedJsonSchema @BundleToExport -SchemaRegistry $SchemaRegistry -ErrorAction Stop
    }
}

end {

}
