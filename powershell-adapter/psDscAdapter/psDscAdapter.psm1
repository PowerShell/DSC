# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

#region functions from PSDesiredStateConfiguration, ported to integrate with DSC.exe
# for versions of PowerShell that do not ship in Windows, eliminate the dependency on installing PSDesiredStateConfiguration modules
# for Windows PowerShell, use the PSDesiredStateConfiguration module that ships in Windows
if ($PSVersionTable.PSVersion.Major -gt 5) {
    data LocalizedData {
        # culture="en-US"
        ConvertFrom-StringData -StringData @'
    InvalidResourceSpecification = Found more than one resource named '{0}'. Please use the module specification to be more specific.
    UnsupportedResourceImplementation = The resource '{0}' implemented as '{1}' is not supported by Invoke-DscResource.
    FileReadError=Error Reading file {0}.
    ResourceNotFound=The term '{0}' is not recognized as the name of a {1}.
    GetDscResourceInputName=The Get-DscResource input '{0}' parameter value is '{1}'.
    ResourceNotMatched=Skipping resource '{0}' as it does not match the requested name.
    LoadingDefaultCimKeywords=Loading default CIM keywords
    GettingModuleList=Getting module list
    CreatingResourceList=Creating resource list
    CreatingResource=Creating resource '{0}'.
    SchemaFileForResource=Schema file name for resource {0}
    NoModulesPresent=There are no modules present in the system with the given module specification.
    PsDscRunAsCredentialNotSupport=The 'PsDscRunAsCredential' property is not currently support when using Invoke-DscResource.
'@
    }
    Set-StrictMode -Off

    # if these files are missing, it is difficult to troubleshoot why the module is not working as expected
    $requiredFileCount = 0
    @(
        "$PSScriptRoot/Configuration/BaseRegistration/BaseResource.Schema.mof"
        "$PSScriptRoot/Configuration/BaseRegistration/MSFT_MetaConfigurationExtensionClasses.Schema.mof"
        "$PSScriptRoot/Configuration/BaseRegistration/en-us/BaseResource.Schema.mfl"
        "$PSScriptRoot/Configuration/BaseRegistration/en-us/MSFT_MetaConfigurationExtensionClasses.Schema.mfl"
    ) | ForEach-Object { if (Test-Path $_) { $requiredFileCount++ } }
    if (4 -ne $requiredFileCount) {
        $trace = @{'Debug' = 'ERROR: The psDscAdapter module is missing required files. Re-install DSC.' } | ConvertTo-Json -Compress
        $host.ui.WriteErrorLine($trace)
    }

    # In case localized resource is not available we revert back to English as defined in LocalizedData section so ignore the error instead of showing it to user.
    Import-LocalizedData -BindingVariable LocalizedData -FileName psDscAdapter.Resource.psd1 -ErrorAction Ignore

    Import-Module $PSScriptRoot/helpers/DscResourceInfo.psm1

    # Set DSC HOME environment variable.
    $env:DSC_HOME = "$PSScriptRoot/Configuration"

    $script:V1MetaConfigPropertyList = @('ConfigurationModeFrequencyMins', 'RebootNodeIfNeeded', 'ConfigurationMode', 'ActionAfterReboot', 'RefreshMode', 'CertificateID', 'ConfigurationID', 'DownloadManagerName', 'DownloadManagerCustomData', 'RefreshFrequencyMins', 'AllowModuleOverwrite', 'DebugMode', 'Credential')
    $script:DirectAccessMetaConfigPropertyList = @('AllowModuleOverWrite', 'CertificateID', 'ConfigurationDownloadManagers', 'ResourceModuleManagers', 'DebugMode', 'RebootNodeIfNeeded', 'RefreshMode', 'ConfigurationAgent')

    # Checks to see if a module defining composite resources should be reloaded
    # based the last write time of the schema file. Returns true if the file exists
    # and the last modified time was either not recorded or has change.
    function Test-ModuleReloadRequired {
        [OutputType([bool])]
        param (
            [Parameter(Mandatory)]
            [string]
            $SchemaFilePath
        )

        if (-not $SchemaFilePath -or $SchemaFilePath -notmatch '\.schema\.psm1$') {
            # not a composite res
            return $false
        }

        # If the path doesn't exist, then we can't reload it.
        # Note: this condition is explicitly not an error for this function.
        if ( -not (Test-Path $SchemaFilePath)) {
            if ($schemaFileLastUpdate.ContainsKey($SchemaFilePath)) {
                $schemaFileLastUpdate.Remove($SchemaFilePath)
            }
            return $false
        }

        # If we have a modified date, then return it.
        if ($schemaFileLastUpdate.ContainsKey($SchemaFilePath)) {
            if ( (Get-Item $SchemaFilePath).LastWriteTime -eq $schemaFileLastUpdate[$SchemaFilePath] ) {
                return $false
            }
            else {
                return $true
            }
        }

        # Otherwise, record the last write time and return true.
        $script:schemaFileLastUpdate[$SchemaFilePath] = (Get-Item $SchemaFilePath).LastWriteTime
        $true
    }

    # Holds the schema file to lastwritetime mapping.
    [System.Collections.Generic.Dictionary[string, DateTime]] $script:schemaFileLastUpdate =
    New-Object -TypeName 'System.Collections.Generic.Dictionary[string,datetime]'

    # Import class resources from module
    function ImportClassResourcesFromModule {
        param (
            [Parameter(Mandatory)]
            [PSModuleInfo]
            $Module,

            [Parameter(Mandatory)]
            [System.Collections.Generic.List[string]]
            $Resources,

            [System.Collections.Generic.Dictionary[string, scriptblock]]
            $functionsToDefine
        )

        $resourcesFound = [Microsoft.PowerShell.DesiredStateConfiguration.Internal.DscClassCache]::ImportClassResourcesFromModule($Module, $Resources, $functionsToDefine)
        return , $resourcesFound
    }

    # Import CIM and Script keywords from a module
    function ImportCimAndScriptKeywordsFromModule {
        param (
            [Parameter(Mandatory)]
            $Module,

            [Parameter(Mandatory)]
            $resource,

            $functionsToDefine
        )

        trap {
            continue
        }

        $SchemaFilePath = $null
        $oldCount = $functionsToDefine.Count

        $keywordErrors = New-Object -TypeName 'System.Collections.ObjectModel.Collection[System.Exception]'

        $foundCimSchema = [Microsoft.PowerShell.DesiredStateConfiguration.Internal.DscClassCache]::ImportCimKeywordsFromModule(
            $Module, $resource, [ref] $SchemaFilePath, $functionsToDefine, $keywordErrors)

        foreach ($ex in $keywordErrors) {
            $trace = @{'Debug' = 'ERROR: ' + $ex } | ConvertTo-Json -Compress
            $host.ui.WriteErrorLine($trace)
            if ($ex.InnerException) {
                $trace = @{'Debug' = 'ERROR: ' + $ex.InnerException } | ConvertTo-Json -Compress
                $host.ui.WriteErrorLine($trace)
            }
        }

        $functionsAdded = $functionsToDefine.Count - $oldCount
        Write-Debug -Message "  $Name : PROCESSING RESOURCE FILE: Added $functionsAdded type handler functions from  '$SchemaFilePath'"

        $SchemaFilePath = $null
        $oldCount = $functionsToDefine.Count

        $foundScriptSchema = [Microsoft.PowerShell.DesiredStateConfiguration.Internal.DscClassCache]::ImportScriptKeywordsFromModule(
            $Module, $resource, [ref] $SchemaFilePath, $functionsToDefine )

        $functionsAdded = $functionsToDefine.Count - $oldCount
        Write-Debug -Message "  $Name : PROCESSING RESOURCE FILE: Added $functionsAdded type handler functions from  '$SchemaFilePath'"

        if ($foundScriptSchema -and $SchemaFilePath) {
            $resourceDirectory = Split-Path $SchemaFilePath
            if ($null -ne $resourceDirectory) {
                Import-Module -Force: (Test-ModuleReloadRequired $SchemaFilePath) -Verbose:$false -Name $resourceDirectory -Global -ErrorAction SilentlyContinue
            }
        }

        return $foundCimSchema -or $foundScriptSchema
    }

    # Utility to throw an error/exception
    function ThrowError {
        param
        (
            [parameter(Mandatory = $true)]
            [ValidateNotNullOrEmpty()]
            [System.String]
            $ExceptionName,

            [parameter(Mandatory = $true)]
            [ValidateNotNullOrEmpty()]
            [System.String]
            $ExceptionMessage,

            [System.Object]
            $ExceptionObject,

            [parameter(Mandatory = $true)]
            [ValidateNotNullOrEmpty()]
            [System.String]
            $errorId,

            [parameter(Mandatory = $true)]
            [ValidateNotNull()]
            [System.Management.Automation.ErrorCategory]
            $errorCategory
        )

        $exception = New-Object $ExceptionName $ExceptionMessage
        $ErrorRecord = New-Object -TypeName System.Management.Automation.ErrorRecord -ArgumentList $exception, $errorId, $errorCategory, $ExceptionObject
        throw $ErrorRecord
    }

    # Gets the list of DSC resource modules on the machine
    function Get-DSCResourceModules {
        $listPSModuleFolders = $env:PSModulePath.Split([IO.Path]::PathSeparator)
        $dscModuleFolderList = [System.Collections.Generic.HashSet[System.String]]::new()

        foreach ($folder in $listPSModuleFolders) {
            if (!(Test-Path $folder)) {
                continue
            }

            foreach ($moduleFolder in Get-ChildItem $folder -Directory) {
                $addModule = $false

                $dscFolders = Get-ChildItem "$($moduleFolder.FullName)\DscResources", "$($moduleFolder.FullName)\*\DscResources" -ErrorAction Ignore
                if ($null -ne $dscFolders) {
                    $addModule = $true
                }

                if (-not $addModule) {
                    foreach ($psd1 in Get-ChildItem -Recurse -Filter "$($moduleFolder.Name).psd1" -Path $moduleFolder.fullname -Depth 2) {
                        $containsDSCResource = Select-String -LiteralPath $psd1 -Pattern '^[^#]*\bDscResourcesToExport\b.*'
                        if ($null -ne $containsDSCResource) {
                            $addModule = $true
                        }
                    }
                }

                if ($addModule) {
                    $dscModuleFolderList.Add($moduleFolder.Name) | Out-Null
                }
            }
        }

        $dscModuleFolderList
    }

    <# public function Get-DscResouce
    .SYNOPSIS
        This function retrieves Desired State Configuration (DSC) resources.

    .DESCRIPTION
        The Get-DscResource function retrieves DSC resources based on the provided parameters. 
        It can retrieve resources by name, module, or syntax. It first loads the default Inbox providers in cache, 
        then retrieves the specified module (if any), and imports class resources from the module. 
        It also handles errors and warnings, and provides verbose output for debugging purposes.

    .PARAMETERS
        - Name: The name of the DSC resource to retrieve.
        - Module: The module that the DSC resource belongs to.
        - Syntax: A switch parameter that, when used, returns the syntax of the DSC resource.

    .EXAMPLE
        Get-DscResource -Name "WindowsFeature" -Module "PSDesiredStateConfiguration"
    #>
    function Get-DscResource {
        [Diagnostics.CodeAnalysis.SuppressMessageAttribute('PSProvideCommentHelp', '', Scope = 'Function', Target = '*')]
        [Diagnostics.CodeAnalysis.SuppressMessageAttribute('PSAvoidUsingPositionalParameters', '', Scope = 'Function', Target = '*')]
        [CmdletBinding(HelpUri = 'http://go.microsoft.com/fwlink/?LinkId=403985')]
        [OutputType('Microsoft.PowerShell.DesiredStateConfiguration.DscResourceInfo[]')]
        [OutputType('string[]')]
        param (
            [Parameter(ValueFromPipeline = $true, ValueFromPipelineByPropertyName = $true)]
            [ValidateNotNullOrEmpty()]
            [string[]]
            $Name,
            [Parameter(ValueFromPipeline = $true, ValueFromPipelineByPropertyName = $true)]
            [ValidateNotNullOrEmpty()]
            [Object]
            $Module,
            [Parameter()]
            [switch]
            $Syntax
        )

        Begin {
            $initialized = $false
            $ModuleString = $null
            Write-Progress -Id 1 -Activity $LocalizedData.LoadingDefaultCimKeywords

            $keywordErrors = New-Object -TypeName 'System.Collections.ObjectModel.Collection[System.Exception]'

            # Load the default Inbox providers (keyword) in cache, also allow caching the resources from multiple versions of modules.
            [Microsoft.PowerShell.DesiredStateConfiguration.Internal.DscClassCache]::LoadDefaultCimKeywords($keywordErrors, $true)

            foreach ($ex in $keywordErrors) {
                $trace = @{'Debug' = 'ERROR: ' + $ex } | ConvertTo-Json -Compress
                $host.ui.WriteErrorLine($trace)
                if ($ex.InnerException) {
                    $trace = @{'Debug' = 'ERROR: ' + $ex.InnerException } | ConvertTo-Json -Compress
                    $host.ui.WriteErrorLine($trace)
                }
            }

            Write-Progress -Id 2 -Activity $LocalizedData.GettingModuleList

            $initialized = $true

            if ($Module) {
                #Pick from the specified module if there's one
                $moduleSpecificName = [System.Management.Automation.LanguagePrimitives]::ConvertTo($Module, [Microsoft.PowerShell.Commands.ModuleSpecification])
                $modules = Get-Module -ListAvailable -FullyQualifiedName $moduleSpecificName

                if ($Module -is [System.Collections.Hashtable]) {
                    $ModuleString = $Module.ModuleName
                }
                elseif ($Module -is [Microsoft.PowerShell.Commands.ModuleSpecification]) {
                    $ModuleString = $Module.Name
                }
                else {
                    $ModuleString = $Module
                }
            }
            else {
                $dscResourceModules = Get-DSCResourceModules
                if ($null -ne $dscResourceModules) {
                    $modules = Get-Module -ListAvailable -Name ($dscResourceModules)
                }
            }

            foreach ($mod in $modules) {
                if ($mod.ExportedDscResources.Count -gt 0) {
                    $null = ImportClassResourcesFromModule -Module $mod -Resources * -functionsToDefine $functionsToDefine
                }

                $dscResources = Join-Path -Path $mod.ModuleBase -ChildPath 'DscResources'
                if (Test-Path $dscResources) {
                    foreach ($resource in Get-ChildItem -Path $dscResources -Directory -Name) {
                        $null = ImportCimAndScriptKeywordsFromModule -Module $mod -Resource $resource -functionsToDefine $functionsToDefine
                    }
                }
            }

            $Resources = @()
        }

        Process {
            try {
                if ($null -ne $Name) {
                    $nameMessage = $LocalizedData.GetDscResourceInputName -f @('Name', [system.string]::Join(', ', $Name))
                    Write-Verbose -Message $nameMessage
                }

                if (!$modules) {
                    #Return if no modules were found with the required specification
                    $trace = @{'Debug' = $LocalizedData.NoModulesPresent } | ConvertTo-Json -Compress
                    $host.ui.WriteErrorLine($trace)
                    return
                }

                $ignoreResourceParameters = @('InstanceName', 'OutputPath', 'ConfigurationData') + [System.Management.Automation.Cmdlet]::CommonParameters + [System.Management.Automation.Cmdlet]::OptionalCommonParameters

                $patterns = GetPatterns $Name

                Write-Progress -Id 3 -Activity $LocalizedData.CreatingResourceList

                # Get resources for CIM cache
                $keywords = [Microsoft.PowerShell.DesiredStateConfiguration.Internal.DscClassCache]::GetCachedKeywords() | Where-Object -FilterScript {
                (!$_.IsReservedKeyword) -and ($null -ne $_.ResourceName) -and !(IsHiddenResource $_.ResourceName) -and (![bool]$Module -or ($_.ImplementingModule -like $ModuleString))
                }

                $dscResourceNames = $keywords.keyword

                $Resources += $keywords |
                ForEach-Object -Process {
                    GetResourceFromKeyword -keyword $_ -patterns $patterns -modules $modules -dscResourceNames $dscResourceNames
                } |
                Where-Object -FilterScript {
                    $_ -ne $null
                }

                # Get composite resources
                $Resources += Get-Command -CommandType Configuration |
                ForEach-Object -Process {
                    GetCompositeResource $patterns $_ $ignoreResourceParameters -modules $modules
                } |
                Where-Object -FilterScript {
                    $_ -ne $null -and (![bool]$ModuleString -or ($_.Module -like $ModuleString)) -and
                ($_.Path -and ((Split-Path -Leaf $_.Path) -eq "$($_.Name).schema.psm1"))
                }

                # check whether all resources are found
                CheckResourceFound $Name $Resources
            }
            catch {
                if ($initialized) {
                    [System.Management.Automation.Language.DynamicKeyword]::Reset()
                    [Microsoft.PowerShell.DesiredStateConfiguration.Internal.DscClassCache]::ClearCache()

                    $initialized = $false
                }

                throw $_
            }
        }

        End {
            $Resources = $Resources | Sort-Object -Property Module, Name -Unique
            foreach ($resource in $Resources) {
                # return formatted string if required
                if ($Syntax) {
                    GetSyntax $resource | Write-Output
                }
                else {
                    Write-Output -InputObject $resource
                }
            }

            if ($initialized) {
                [System.Management.Automation.Language.DynamicKeyword]::Reset()
                [Microsoft.PowerShell.DesiredStateConfiguration.Internal.DscClassCache]::ClearCache()

                $initialized = $false
            }
        }
    }

    # Get DSC resoruce for a dynamic keyword
    function GetResourceFromKeyword {
        [OutputType('Microsoft.PowerShell.DesiredStateConfiguration.DscResourceInfo')]
        param (
            [Parameter(Mandatory)]
            [System.Management.Automation.Language.DynamicKeyword]
            $keyword,
            [System.Management.Automation.WildcardPattern[]]
            $patterns,
            [Parameter(Mandatory)]
            [System.Management.Automation.PSModuleInfo[]]
            $modules,
            [Parameter(Mandatory)]
            [Object[]]
            $dscResourceNames
        )
        $implementationDetail = 'ScriptBased'

        # Find whether $name follows the pattern
        $matched = (IsPatternMatched $patterns $keyword.ResourceName) -or (IsPatternMatched $patterns $keyword.Keyword)
        if ($matched -eq $false) {
            $message = $LocalizedData.ResourceNotMatched -f @($keyword.Keyword)
            Write-Verbose -Message ($message)
            return
        }
        else {
            $message = $LocalizedData.CreatingResource -f @($keyword.Keyword)
            Write-Verbose -Message $message
        }

        $resource = New-Object -TypeName Microsoft.PowerShell.DesiredStateConfiguration.DscResourceInfo

        $resource.ResourceType = $keyword.ResourceName

        if ($keyword.ResourceName -ne $keyword.Keyword) {
            $resource.FriendlyName = $keyword.Keyword
        }

        $resource.Name = $keyword.Keyword

        $schemaFiles = [Microsoft.PowerShell.DesiredStateConfiguration.Internal.DscClassCache]::GetFileDefiningClass($keyword.ResourceName)

        if ($schemaFiles.Count) {
            # Find the correct schema file that matches module name and version
            # if same module/version is installed in multiple locations, then pick the first schema file.
            foreach ($schemaFileName in $schemaFiles) {
                $moduleInfo = GetModule $modules $schemaFileName
                if ($moduleInfo.Name -eq $keyword.ImplementingModule -and $moduleInfo.Version -eq $keyword.ImplementingModuleVersion) {
                    break
                }
            }

            # if the class is not a resource we will ignore it except if it is DSC inbox resource.
            if (-not $schemaFileName.StartsWith("$env:windir\system32\configuration", [stringComparison]::OrdinalIgnoreCase)) {
                $classesFromSchema = [Microsoft.PowerShell.DesiredStateConfiguration.Internal.DscClassCache]::GetCachedClassByFileName($schemaFileName)
                if ($null -ne $classesFromSchema) {
                    # check if the resource is proper DSC resource that always derives from OMI_BaseResource.
                    $schemaToProcess = $classesFromSchema | ForEach-Object -Process {
                        if (($_.CimSystemProperties.ClassName -ieq $keyword.ResourceName) -and ($_.CimSuperClassName -ieq 'OMI_BaseResource')) {
                            $member = Get-Member -InputObject $_ -MemberType NoteProperty -Name 'ImplementationDetail'
                            if ($null -eq $member) {
                                $_ | Add-Member -MemberType NoteProperty -Name 'ImplementationDetail' -Value $implementationDetail -PassThru
                            }
                            else {
                                $_
                            }
                        }
                    }
                    if ($null -eq $schemaToProcess) {
                        return
                    }
                }
            }

            $message = $LocalizedData.SchemaFileForResource -f @($schemaFileName)
            Write-Verbose -Message $message

            $resource.Module = $moduleInfo
            $resource.Path = GetImplementingModulePath $schemaFileName
            $resource.ParentPath = Split-Path $schemaFileName
        }
        else {
            $implementationDetail = 'ClassBased'
            $Module = $modules | Where-Object -FilterScript {
                $_.Name -eq $keyword.ImplementingModule -and
                $_.Version -eq $keyword.ImplementingModuleVersion
            }

            if ($Module -and $Module.ExportedDscResources -contains $keyword.Keyword) {
                $implementationDetail = 'ClassBased'
                $resource.Module = $Module
                $resource.Path = $Module.Path
                $resource.ParentPath = Split-Path -Path $Module.Path
            }
        }

        if ([system.string]::IsNullOrEmpty($resource.Path) -eq $false) {
            $resource.ImplementedAs = [Microsoft.PowerShell.DesiredStateConfiguration.ImplementedAsType]::PowerShell
        }
        else {
            $implementationDetail = 'Binary'
            $resource.ImplementedAs = [Microsoft.PowerShell.DesiredStateConfiguration.ImplementedAsType]::Binary
        }

        if ($null -ne $resource.Module) {
            $resource.CompanyName = $resource.Module.CompanyName
        }

        # add properties
        $keyword.Properties.Values | ForEach-Object -Process {
            AddDscResourceProperty $resource $_ $dscResourceNames
        }

        # sort properties
        $updatedProperties = $resource.Properties | Sort-Object -Property @{
            expression = 'IsMandatory'
            Descending = $true
        }, @{
            expression = 'Name'
            Ascending  = $true
        }
        $resource.UpdateProperties($updatedProperties)
    
        $resource | Add-Member -MemberType NoteProperty -Name 'ImplementationDetail' -Value $implementationDetail

        return $resource
    }

    # Gets composite resource
    function GetCompositeResource {
        [Diagnostics.CodeAnalysis.SuppressMessageAttribute('PSAvoidUsingPositionalParameters', '', Scope = 'Function', Target = '*')]
        [OutputType('Microsoft.PowerShell.DesiredStateConfiguration.DscResourceInfo')]
        param (
            [System.Management.Automation.WildcardPattern[]]
            $patterns,
            [Parameter(Mandatory)]
            [System.Management.Automation.ConfigurationInfo]
            $configInfo,
            $ignoreParameters,
            [Parameter(Mandatory)]
            [System.Management.Automation.PSModuleInfo[]]
            $modules
        )

        # Find whether $name follows the pattern
        $matched = IsPatternMatched $patterns $configInfo.Name
        if ($matched -eq $false) {
            $message = $LocalizedData.ResourceNotMatched -f @($configInfo.Name)
            Write-Verbose -Message ($message)

            return $null
        }
        else {
            $message = $LocalizedData.CreatingResource -f @($configInfo.Name)
            Write-Verbose -Message $message
        }

        $resource = New-Object -TypeName Microsoft.PowerShell.DesiredStateConfiguration.DscResourceInfo

        $resource.ResourceType = $configInfo.Name
        $resource.FriendlyName = $null
        $resource.Name = $configInfo.Name
        $resource.ImplementedAs = [Microsoft.PowerShell.DesiredStateConfiguration.ImplementedAsType]::Composite

        if ($null -ne $configInfo.Module) {
            $resource.Module = GetModule $modules $configInfo.Module.Path
            if ($null -eq $resource.Module) {
                $resource.Module = $configInfo.Module
            }
            $resource.CompanyName = $configInfo.Module.CompanyName
            $resource.Path = $configInfo.Module.Path
            $resource.ParentPath = Split-Path -Path $resource.Path
        }

        # add properties
        $configInfo.Parameters.Values | ForEach-Object -Process {
            AddDscResourcePropertyFromMetadata $resource $_ $ignoreParameters
        }

        $resource | Add-Member -MemberType NoteProperty -Name 'ImplementationDetail' -Value 'Composite'
        return $resource
    }

    # Adds property to a DSC resource
    function AddDscResourceProperty {
        param (
            [Parameter(Mandatory)]
            [Microsoft.PowerShell.DesiredStateConfiguration.DscResourceInfo]
            $dscresource,
            [Parameter(Mandatory)]
            $property,
            [Parameter(Mandatory)]
            $dscResourceNames
        )

        $convertTypeMap = @{
            'MSFT_Credential'     = '[PSCredential]'
            'MSFT_KeyValuePair'   = '[HashTable]'
            'MSFT_KeyValuePair[]' = '[HashTable]'
        }

        $ignoreProperties = @('ResourceId', 'ConfigurationName')
        if ($ignoreProperties -contains $property.Name) {
            return
        }

        $dscProperty = New-Object -TypeName Microsoft.PowerShell.DesiredStateConfiguration.DscResourcePropertyInfo
        $dscProperty.Name = $property.Name
        if ($convertTypeMap.ContainsKey($property.TypeConstraint)) {
            $type = $convertTypeMap[$property.TypeConstraint]
        }
        else {
            $Type = [System.Management.Automation.LanguagePrimitives]::ConvertTypeNameToPSTypeName($property.TypeConstraint)
            if ([string]::IsNullOrEmpty($Type)) {
                $dscResourceNames | ForEach-Object -Process {
                    if (($property.TypeConstraint -eq $_) -or ($property.TypeConstraint -eq ($_ + '[]'))) { $Type = "[$($property.TypeConstraint)]" }
                }
            }
        }

        if ($null -ne $property.ValueMap) {
            $property.ValueMap.Keys |
            Sort-Object |
            ForEach-Object -Process {
                $dscProperty.Values.Add($_)
            }
        }

        $dscProperty.PropertyType = $Type
        $dscProperty.IsMandatory = $property.Mandatory

        $dscresource.Properties.Add($dscProperty)
    }

    # Adds property to a DSC resource
    function AddDscResourcePropertyFromMetadata {
        param (
            [Parameter(Mandatory)]
            [Microsoft.PowerShell.DesiredStateConfiguration.DscResourceInfo]
            $dscresource,
            [Parameter(Mandatory)]
            [System.Management.Automation.ParameterMetadata]
            $parameter,
            $ignoreParameters
        )

        if ($ignoreParameters -contains $parameter.Name) {
            return
        }

        $dscProperty = New-Object -TypeName Microsoft.PowerShell.DesiredStateConfiguration.DscResourcePropertyInfo
        $dscProperty.Name = $parameter.Name

        # adding [] in Type name to keep it in sync with the name returned from LanguagePrimitives.ConvertTypeNameToPSTypeName
        $dscProperty.PropertyType = '[' + $parameter.ParameterType.Name + ']'
        $dscProperty.IsMandatory = $parameter.Attributes.Mandatory

        $dscresource.Properties.Add($dscProperty)
    }

    # Gets syntax for a DSC resource
    function GetSyntax {
        [OutputType('string')]
        param (
            [Parameter(Mandatory)]
            [Microsoft.PowerShell.DesiredStateConfiguration.DscResourceInfo]
            $dscresource
        )

        $output = $dscresource.Name + " [String] #ResourceName`n"
        $output += "{`n"
        foreach ($property in $dscresource.Properties) {
            $output += '    '
            if ($property.IsMandatory -eq $false) {
                $output += '['
            }

            $output += $property.Name

            $output += ' = ' + $property.PropertyType + ''

            # Add possible values
            if ($property.Values.Count -gt 0) {
                $output += '{ ' + [system.string]::Join(' | ', $property.Values) + ' }'
            }

            if ($property.IsMandatory -eq $false) {
                $output += ']'
            }

            $output += "`n"
        }

        $output += "}`n"

        return $output
    }

    # Checks whether a resource is found or not
    function CheckResourceFound($names, $Resources) {
        if ($null -eq $names) {
            return
        }

        $namesWithoutWildcards = $names | Where-Object -FilterScript {
            [System.Management.Automation.WildcardPattern]::ContainsWildcardCharacters($_) -eq $false
        }

        foreach ($Name in $namesWithoutWildcards) {
            $foundResources = $Resources | Where-Object -FilterScript {
            ($_.Name -eq $Name) -or ($_.ResourceType -eq $Name)
            }
            if ($foundResources.Count -eq 0) {
                $errorMessage = $LocalizedData.ResourceNotFound -f @($Name, 'Resource')
                $trace = @{'Debug' = 'ERROR: ' + $errorMessage } | ConvertTo-Json -Compress
                $host.ui.WriteErrorLine($trace)
            }
        }
    }

    # Get implementing module path
    function GetImplementingModulePath {
        param (
            [Parameter(Mandatory)]
            [string]
            $schemaFileName
        )

        $moduleFileName = ($schemaFileName -replace '.schema.mof$', '') + '.psd1'
        if (Test-Path $moduleFileName) {
            return $moduleFileName
        }

        $moduleFileName = ($schemaFileName -replace '.schema.mof$', '') + '.psm1'
        if (Test-Path $moduleFileName) {
            return $moduleFileName
        }

        return
    }

    # Gets module for a DSC resource
    function GetModule {
        [OutputType('System.Management.Automation.PSModuleInfo')]
        param (
            [Parameter(Mandatory)]
            [System.Management.Automation.PSModuleInfo[]]
            $modules,
            [Parameter(Mandatory)]
            [string]
            $schemaFileName
        )

        if ($null -eq $schemaFileName) {
            return $null
        }

        $schemaFileExt = $null
        if ($schemaFileName -match '.schema.mof') {
            $schemaFileExt = '.schema.mof$'
        }

        if ($schemaFileName -match '.schema.psm1') {
            $schemaFileExt = '.schema.psm1$'
        }

        if (!$schemaFileExt) {
            return $null
        }

        # get module from parent directory.
        # Desired structure is : <Module-directory>/DscResources/<schema file directory>/schema.File
        $validResource = $false
        $schemaDirectory = Split-Path $schemaFileName
        if ($schemaDirectory) {
            $subDirectory = [System.IO.Directory]::GetParent($schemaDirectory)

            if ($subDirectory -and ($subDirectory.Name -eq 'DscResources') -and $subDirectory.Parent) {
                $results = $modules | Where-Object -FilterScript {
                    $_.ModuleBase -eq $subDirectory.Parent.FullName
                }

                if ($results) {
                    # Log Resource is internally handled by the CA. There is no formal provider for it.
                    if ($schemaFileName -match 'MSFT_LogResource') {
                        $validResource = $true
                    }
                    else {
                        # check for proper resource module
                        foreach ($ext in @('.psd1', '.psm1', '.dll', '.cdxml')) {
                            $resModuleFileName = ($schemaFileName -replace $schemaFileExt, '') + $ext
                            if (Test-Path($resModuleFileName)) {
                                $validResource = $true
                                break
                            }
                        }
                    }
                }
            }
        }

        if ($results -and $validResource) {
            return $results[0]
        }
        else {
            return $null
        }
    }

    # Checks whether a resource is hidden or not
    function IsHiddenResource {
        param (
            [Parameter(Mandatory)]
            [string]
            $ResourceName
        )

        $hiddenResources = @(
            'OMI_BaseResource',
            'MSFT_KeyValuePair',
            'MSFT_BaseConfigurationProviderRegistration',
            'MSFT_CimConfigurationProviderRegistration',
            'MSFT_PSConfigurationProviderRegistration',
            'OMI_ConfigurationDocument',
            'MSFT_Credential',
            'MSFT_DSCMetaConfiguration',
            'OMI_ConfigurationDownloadManager',
            'OMI_ResourceModuleManager',
            'OMI_ReportManager',
            'MSFT_FileDownloadManager',
            'MSFT_WebDownloadManager',
            'MSFT_FileResourceManager',
            'MSFT_WebResourceManager',
            'MSFT_WebReportManager',
            'OMI_MetaConfigurationResource',
            'MSFT_PartialConfiguration',
            'MSFT_DSCMetaConfigurationV2'
        )

        return $hiddenResources -contains $ResourceName
    }

    # Gets patterns for names
    function GetPatterns {
        [OutputType('System.Management.Automation.WildcardPattern[]')]
        param (
            [string[]]
            $names
        )

        $patterns = @()

        if ($null -eq $names) {
            return $patterns
        }

        foreach ($Name in $names) {
            $patterns += New-Object -TypeName System.Management.Automation.WildcardPattern -ArgumentList @($Name, [System.Management.Automation.WildcardOptions]::IgnoreCase)
        }

        return $patterns
    }

    # Checks whether an input name matches one of the patterns
    # $pattern is not expected to have an empty or null values
    function IsPatternMatched {
        [OutputType('bool')]
        param (
            [System.Management.Automation.WildcardPattern[]]
            $patterns,
            [Parameter(Mandatory)]
            [string]
            $Name
        )

        if ($null -eq $patterns) {
            return $true
        }

        foreach ($pattern in $patterns) {
            if ($pattern.IsMatch($Name)) {
                return $true
            }
        }

        return $false
    }

    <# public function Invoke-DscResource
    .SYNOPSIS
        This function is used to invoke a Desired State Configuration (DSC) resource.

    .DESCRIPTION
        The Invoke-DscResource function takes in a DSC resource name, module name, method, and properties as parameters. 
        It first checks if the 'PsDscRunAsCredential' property is present, and if so, throws an error as it's not supported.
        It then retrieves the DSC resource using the Get-DscResource function. 
        If no resources are found, or more than one resource is found, it throws an error.
        It checks if the DSC resource is implemented as 'PowerShell', and if not, throws an error.
        Finally, it invokes the DSC resource. If the resource is class-based, it uses the Invoke-DscClassBasedResource function. 
        Otherwise, it uses the Invoke-DscScriptBasedResource function.

    .PARAMETERS
        - Name: The name of the DSC resource to invoke.
        - ModuleName: The module that the DSC resource belongs to.
        - Method: The method to invoke on the DSC resource. Must be one of 'Get', 'Set', 'Test'.
        - Property: A hashtable of properties to pass to the DSC resource.

    .EXAMPLE
        Invoke-DscResource -Name "WindowsFeature" -Method "Set" -Property @{ Name = "Web-Server"; Ensure = "Present" }
    #>
    function Invoke-DscResource {
        [CmdletBinding(HelpUri = '')]
        param (
            [Parameter(ValueFromPipeline = $true, ValueFromPipelineByPropertyName = $true, Mandatory)]
            [ValidateNotNullOrEmpty()]
            [string]
            $Name,
            [Parameter(ValueFromPipeline = $true, ValueFromPipelineByPropertyName = $true)]
            [ValidateNotNullOrEmpty()]
            [Microsoft.PowerShell.Commands.ModuleSpecification]
            $ModuleName,
            [Parameter(Mandatory)]
            [ValidateSet('Get', 'Set', 'Test')]
            [string]
            $Method,
            [Parameter(Mandatory)]
            [Hashtable]
            $Property
        )

        $getArguments = @{
            Name = $Name
        }

        if ($Property.ContainsKey('PsDscRunAsCredential')) {
            $errorMessage = $LocalizedData.PsDscRunAsCredentialNotSupport -f $name
            $exception = [System.ArgumentException]::new($errorMessage, 'Name')
            ThrowError -ExceptionName 'System.ArgumentException' -ExceptionMessage $errorMessage -ExceptionObject $exception -ErrorId 'PsDscRunAsCredentialNotSupport,Invoke-DscResource' -ErrorCategory InvalidArgument
        }

        if ($ModuleName) {
            $getArguments.Add('Module', $ModuleName)
        }

        Write-Debug -Message "Getting DSC Resource $Name"
        $resource = @(Get-DscResource @getArguments -ErrorAction stop)

        if ($resource.Count -eq 0) {
            throw 'unexpected state - no resources found - get-dscresource should have thrown'
        }

        if ($resource.Count -ne 1) {
            $errorMessage = $LocalizedData.InvalidResourceSpecification -f $name
            $exception = [System.ArgumentException]::new($errorMessage, 'Name')
            ThrowError -ExceptionName 'System.ArgumentException' -ExceptionMessage $errorMessage -ExceptionObject $exception -ErrorId 'InvalidResourceSpecification,Invoke-DscResource' -ErrorCategory InvalidArgument
        }

        [Microsoft.PowerShell.DesiredStateConfiguration.DscResourceInfo] $resource = $resource[0]
        if ($resource.ImplementedAs -ne 'PowerShell') {
            $errorMessage = $LocalizedData.UnsupportedResourceImplementation -f $name, $resource.ImplementedAs
            $exception = [System.InvalidOperationException]::new($errorMessage)
            ThrowError -ExceptionName 'System.InvalidOperationException' -ExceptionMessage $errorMessage -ExceptionObject $exception -ErrorId 'UnsupportedResourceImplementation,Invoke-DscResource' -ErrorCategory InvalidOperation
        }

        $resourceInfo = $resource | Out-String
        Write-Debug $resourceInfo

        if ($resource.ImplementationDetail -eq 'ClassBased') {
            Invoke-DscClassBasedResource -Resource $resource -Method $Method -Property $Property
        }
        else {
            Invoke-DscScriptBasedResource -Resource $resource -Method $Method -Property $Property
        }
    }

    # Class to return Test method results for Invoke-DscResource
    class InvokeDscResourceTestResult {
        [bool] $InDesiredState
    }

    # Class to return Set method results for Invoke-DscResource
    class InvokeDscResourceSetResult {
        [bool] $RebootRequired
    }

    # Run methods from class-based DSC resources
    function Invoke-DscClassBasedResource {
        [Diagnostics.CodeAnalysis.SuppressMessageAttribute('PSAvoidGlobalVars', '', Scope = 'Function')]
        [Diagnostics.CodeAnalysis.SuppressMessageAttribute('PSUseDeclaredVarsMoreThanAssignments', '', Scope = 'Function')]
        param(
            [Parameter(Mandatory)]
            [Microsoft.PowerShell.DesiredStateConfiguration.DscResourceInfo] $resource,
            [Parameter(Mandatory)]
            [ValidateSet('Get', 'Set', 'Test')]
            [string]
            $Method,
            [Hashtable]
            $Property
        )

        $path = $resource.Path
        $type = $resource.ResourceType

        Write-Debug "Importing $path ..."
        $iss = [System.Management.Automation.Runspaces.InitialSessionState]::CreateDefault2()
        $powershell = [PowerShell]::Create($iss)
        $script = @"
using module "$path"

Write-Host -Message ([$type]::new | out-string)
return [$type]::new()
"@


        $null = $powershell.AddScript($script)
        $dscType = $powershell.Invoke() | Select-Object -First 1
        foreach ($key in $Property.Keys) {
            $value = $Property.$key
            Write-Debug "Setting $key to $value"
            $dscType.$key = $value
        }
        $info = $dscType | Out-String
        Write-Debug $info

        Write-Debug "calling $type.$Method() ..."
        $global:DSCMachineStatus = $null
        $output = $dscType.$Method()
        return Get-InvokeDscResourceResult -Output $output -Method $Method
    }

    # Run private functions from class-based DSC resources
    function Invoke-DscScriptBasedResource {
        [Diagnostics.CodeAnalysis.SuppressMessageAttribute('PSAvoidGlobalVars', '', Scope = 'Function')]
        [Diagnostics.CodeAnalysis.SuppressMessageAttribute('PSUseDeclaredVarsMoreThanAssignments', '', Scope = 'Function')]
        param(
            [Parameter(Mandatory)]
            [Microsoft.PowerShell.DesiredStateConfiguration.DscResourceInfo] $resource,
            [Parameter(Mandatory)]
            [ValidateSet('Get', 'Set', 'Test')]
            [string]
            $Method,
            [Hashtable]
            $Property
        )

        $path = $resource.Path
        $type = $resource.ResourceType

        Write-Debug "Importing $path ..."
        Import-Module -Scope Local -Name $path -Force -ErrorAction stop

        $functionName = "$Method-TargetResource"

        Write-Debug "calling $name\$functionName ..."
        $global:DSCMachineStatus = $null
        $output = & $type\$functionName @Property
        return Get-InvokeDscResourceResult -Output $output -Method $Method
    }

    # Format output of Invoke-DscResource
    function Get-InvokeDscResourceResult {
        [Diagnostics.CodeAnalysis.SuppressMessageAttribute('PSAvoidGlobalVars', '', Scope = 'Function')]
        param(
            $Output,
            $Method
        )

        switch ($Method) {
            'Set' {
                $Output | ForEach-Object -Process {
                    Write-Verbose -Message ('output: ' + $_)
                }
                $rebootRequired = if ($global:DSCMachineStatus -eq 1) { $true } else { $false }
                return [InvokeDscResourceSetResult]@{
                    RebootRequired = $rebootRequired
                }
            }
            'Test' {
                return [InvokeDscResourceTestResult]@{
                    InDesiredState = $Output
                }
            }
            default {
                return $Output
            }
        }
    }
}
#endregion

<# public function Invoke-DscCacheRefresh
.SYNOPSIS
    This function caches the results of the Get-DscResource call to optimize performance.

.DESCRIPTION
    This function is designed to improve the performance of DSC operations by caching the results of the Get-DscResource call. 
    By storing the results, subsequent calls to Get-DscResource can retrieve the cached data instead of making a new call each time. 
    This can significantly speed up operations that need to repeatedly access DSC resources.

.EXAMPLE
    Invoke-DscCacheRefresh -Module "PSDesiredStateConfiguration"
#>
function Invoke-DscCacheRefresh {
    [CmdletBinding(HelpUri = '')]
    param(
        [Parameter(ValueFromPipeline = $true, ValueFromPipelineByPropertyName = $true)]
        [Object[]]
        $Module
    )
    # for the WindowsPowerShell adapter, always use the version of PSDesiredStateConfiguration that ships in Windows
    if ($PSVersionTable.PSVersion.Major -le 5) {
        $psdscWindowsPath = "$env:windir\System32\WindowsPowerShell\v1.0\Modules\PSDesiredStateConfiguration\PSDesiredStateConfiguration.psd1"
        Import-Module $psdscWindowsPath -Force -ErrorAction stop -ErrorVariable $importModuleError
        if (-not [string]::IsNullOrEmpty($importModuleError)) {
            $trace = @{'Debug' = 'ERROR: Could not import PSDesiredStateConfiguration 1.1 in Windows PowerShell. ' + $importModuleError } | ConvertTo-Json -Compress
            $host.ui.WriteErrorLine($trace)
        }
        $DSCVersion = [version]'1.1.0'
    }
        
    # cache the results of Get-DscResource
    [dscResourceCache[]]$dscResourceCache = @()

    # improve by performance by having the option to only get details for named modules
    # workaround for File and SignatureValidation resources that ship in Windows
    if ($null -ne $module -and 'Windows' -ne $module) {
        if ($module.gettype().name -eq 'string') {
            $module = @($module)
        }
        $DscResources = @()
        $Modules = @()
        foreach ($m in $module) {
            $DscResources += Get-DscResource -Module $m
            $Modules += Get-Module -Name $m -ListAvailable
        }
    }
    elseif ('Windows' -eq $module) {
        $DscResources = Get-DscResource | Where-Object { $_.modulename -eq $null -and $_.parentpath -like "$env:windir\System32\Configuration\*" }
    }
    else {
        $DscResources = Get-DscResource
        $Modules = Get-Module -ListAvailable
    }

    foreach ($dscResource in $DscResources) {
        # resources that shipped in Windows should only be used with Windows PowerShell
        if ($dscResource.ParentPath -like "$env:windir\System32\*" -and $PSVersionTable.PSVersion.Major -gt 5) {
            continue
        }

        # we can't run this check in PSDesiredStateConfiguration 1.1 because the property doesn't exist
        if ( $DSCVersion -ge [version]'2.0.0' ) {
            # only support known dscResourceType
            if ([dscResourceType].GetEnumNames() -notcontains $dscResource.ImplementationDetail) {
                $trace = @{'Debug' = 'WARNING: implementation detail not found: ' + $dscResource.ImplementationDetail } | ConvertTo-Json -Compress
                $host.ui.WriteErrorLine($trace)
                continue
            }
        }

        # workaround: if the resource does not have a module name, get it from parent path
        # workaround: modulename is not settable, so clone the object without being read-only
        # workaround: we have to special case File and SignatureValidation resources that ship in Windows
        $binaryBuiltInModulePaths = @(
            "$env:windir\system32\Configuration\Schema\MSFT_FileDirectoryConfiguration"
            "$env:windir\system32\Configuration\BaseRegistration"
        )
        $DscResourceInfo = [DscResourceInfo]::new()
        $dscResource.PSObject.Properties | ForEach-Object -Process { $DscResourceInfo.$($_.Name) = $_.Value }
        if ($dscResource.ModuleName) {
            $moduleName = $dscResource.ModuleName
        }
        elseif ($binaryBuiltInModulePaths -contains $dscResource.ParentPath) {
            $moduleName = 'Windows'
            $DscResourceInfo.Module = 'Windows'
            $DscResourceInfo.ModuleName = 'Windows'
            $DscResourceInfo.CompanyName = 'Microsoft Corporation'
            $DscResourceInfo.Version = '1.0.0'
        }
        elseif ($dscResource.ParentPath) {
            # workaround: populate module name from parent path that is three levels up
            $moduleName = Split-Path $dscResource.ParentPath | Split-Path | Split-Path -Leaf
            $DscResourceInfo.Module = $moduleName
            $DscResourceInfo.ModuleName = $moduleName
            # workaround: populate module version from psmoduleinfo if available
            if ($moduleInfo = $Modules | Where-Object { $_.Name -eq $moduleName }) {
                $moduleInfo = $moduleInfo | Sort-Object -Property Version -Descending | Select-Object -First 1
                $DscResourceInfo.Version = $moduleInfo.Version.ToString()
            }
        }

        $dscResourceCache += [dscResourceCache]@{
            Type            = "$moduleName/$($dscResource.Name)"
            DscResourceInfo = $DscResourceInfo
        }
    }
    return $dscResourceCache
}

# Convert the INPUT to a dscResourceObject object so configuration and resource are standardized as moch as possible
function Get-DscResourceObject {
    param(
        [Parameter(Mandatory = $true, ValueFromPipeline = $true)]
        $jsonInput
    )
    # normalize the INPUT object to an array of dscResourceObject objects
    $inputObj = $jsonInput | ConvertFrom-Json
    $desiredState = [System.Collections.Generic.List[Object]]::new()

    # catch potential for improperly formatted configuration input
    if ($inputObj.resources -and -not $inputObj.metadata.'Microsoft.DSC'.context -eq 'configuration') {
        $trace = @{'Debug' = 'WARNING: The input has a top level property named "resources" but is not a configuration. If the input should be a configuration, include the property: "metadata": {"Microsoft.DSC": {"context": "Configuration"}}' } | ConvertTo-Json -Compress
        $host.ui.WriteErrorLine($trace)
    }

    # match adapter to version of powershell
    if ($PSVersionTable.PSVersion.Major -le 5) {
        $adapterName = 'Microsoft.DSC/WindowsPowerShell'
    }
    else {
        $adapterName = 'Microsoft.DSC/PowerShell'
    }

    if ($null -ne $inputObj.metadata -and $null -ne $inputObj.metadata.'Microsoft.DSC' -and $inputObj.metadata.'Microsoft.DSC'.context -eq 'configuration') {
        # change the type from pscustomobject to dscResourceObject
        $inputObj.resources | ForEach-Object -Process {
            $desiredState += [dscResourceObject]@{
                name       = $_.name
                type       = $_.type
                properties = $_.properties
            }
        }
    }
    else {
        # mimic a config object with a single resource
        $type = $inputObj.type
        $inputObj.psobject.properties.Remove('type')
        $desiredState += [dscResourceObject]@{
            name       = $adapterName
            type       = $type
            properties = $inputObj
        }
    }
    return $desiredState
}

# Get the actual state using DSC Get method from any type of DSC resource
function Get-ActualState {
    param(
        [Parameter(Mandatory, ValueFromPipeline = $true)]
        [dscResourceObject]$DesiredState,
        [Parameter(Mandatory)]
        [dscResourceCache[]]$dscResourceCache
    )

    # for the WindowsPowerShell adapter, always use the version of PSDesiredStateConfiguration that ships in Windows
    if ($PSVersionTable.PSVersion.Major -le 5) {
        $psdscWindowsPath = "$env:windir\System32\WindowsPowerShell\v1.0\Modules\PSDesiredStateConfiguration\PSDesiredStateConfiguration.psd1"
        Import-Module $psdscWindowsPath -Force -ErrorAction stop -ErrorVariable $importModuleError
        if (-not [string]::IsNullOrEmpty($importModuleError)) {
            $trace = @{'Debug' = 'ERROR: Could not import PSDesiredStateConfiguration 1.1 in Windows PowerShell. ' + $importModuleError } | ConvertTo-Json -Compress
            $host.ui.WriteErrorLine($trace)
        }
    }

    # get details from cache about the DSC resource, if it exists
    $cachedDscResourceInfo = $dscResourceCache | Where-Object Type -EQ $DesiredState.type | ForEach-Object DscResourceInfo

    # if the resource is found in the cache, get the actual state
    if ($cachedDscResourceInfo) {

        # formated OUTPUT of each resource
        $addToActualState = [dscResourceObject]@{}

        # set top level properties of the OUTPUT object from INPUT object
        $DesiredState.psobject.properties | ForEach-Object -Process {
            if ($_.TypeNameOfValue -EQ 'System.String') { $addToActualState.$($_.Name) = $DesiredState.($_.Name) }
        }

        # workaround: script based resources do not validate Get parameter consistency, so we need to remove any parameters the author chose not to include in Get-TargetResource
        switch ([dscResourceType]$cachedDscResourceInfo.ImplementationDetail) {
            'ScriptBased' {

                # For Linux/MacOS, only class based resources are supported and are called directly.
                if (!$IsWindows) {
                    $trace = @{'Debug' = 'ERROR: Script based resources are only supported on Windows.' } | ConvertTo-Json -Compress
                    $host.ui.WriteErrorLine($trace)
                    exit 1
                }

                # imports the .psm1 file for the DSC resource as a PowerShell module and stores the list of parameters
                Import-Module -Scope Local -Name $cachedDscResourceInfo.path -Force -ErrorAction stop
                $validParams = (Get-Command -Module $cachedDscResourceInfo.ResourceType -Name 'Get-TargetResource').Parameters.Keys
                # prune any properties that are not valid parameters of Get-TargetResource
                $DesiredState.properties.psobject.properties | ForEach-Object -Process {
                    if ($validParams -notcontains $_.Name) {
                        $DesiredState.properties.psobject.properties.Remove($_.Name)
                    }
                }

                # morph the INPUT object into a hashtable named "property" for the cmdlet Invoke-DscResource
                $DesiredState.properties.psobject.properties | ForEach-Object -Begin { $property = @{} } -Process { $property[$_.Name] = $_.Value }

                # using the cmdlet the appropriate dsc module, and handle errors
                try {
                    $getResult = Invoke-DscResource -Method Get -ModuleName $cachedDscResourceInfo.ModuleName -Name $cachedDscResourceInfo.Name -Property $property

                    # set the properties of the OUTPUT object from the result of Get-TargetResource
                    $addToActualState.properties = $getResult
                }
                catch {
                    $trace = @{'Debug' = 'ERROR: ' + $_.Exception.Message } | ConvertTo-Json -Compress
                    $host.ui.WriteErrorLine($trace)
                    exit 1
                }
            }
            'ClassBased' {
                try {
                    # load powershell class from external module
                    $resource = GetTypeInstanceFromModule -modulename $cachedDscResourceInfo.ModuleName -classname $cachedDscResourceInfo.Name
                    $dscResourceInstance = $resource::New()

                    # set each property of $dscResourceInstance to the value of the property in the $desiredState INPUT object
                    $DesiredState.properties.psobject.properties | ForEach-Object -Process {
                        $dscResourceInstance.$($_.Name) = $_.Value
                    }
                    $getResult = $dscResourceInstance.Get()

                    # set the properties of the OUTPUT object from the result of Get-TargetResource
                    $addToActualState.properties = $getResult
                }
                catch {
                    
                    $trace = @{'Debug' = 'ERROR: ' + $_.Exception.Message } | ConvertTo-Json -Compress
                    $host.ui.WriteErrorLine($trace)
                    exit 1
                }
            }
            'Binary' {
                if (-not ($PSVersionTable.PSVersion.Major -lt 6)) {
                    $trace = @{'Debug' = 'To use a binary resource such as File, use the Microsoft.DSC/WindowsPowerShell adapter.' } | ConvertTo-Json -Compress
                    $host.ui.WriteErrorLine($trace)
                    exit 1
                }

                if (-not (($cachedDscResourceInfo.ModuleName -eq 'Windows') -and ('File', 'Log', 'SignatureValidation' -contains $cachedDscResourceInfo.Name))) {
                    $trace = @{'Debug' = 'Only File, Log, and SignatureValidation are supported as Binary resources.' } | ConvertTo-Json -Compress
                    $host.ui.WriteErrorLine($trace)
                    exit 1
                }

                # morph the INPUT object into a hashtable named "property" for the cmdlet Invoke-DscResource
                $DesiredState.properties.psobject.properties | ForEach-Object -Begin { $property = @{} } -Process { $property[$_.Name] = $_.Value }

                # using the cmdlet from PSDesiredStateConfiguration module in Windows
                try {
                    $getResult = PSDesiredStateConfiguration\Invoke-DscResource -Method Get -ModuleName 'PSDesiredStateConfiguration' -Name $cachedDscResourceInfo.Name -Property $property

                    # only return DSC properties from the Cim instance
                    $cachedDscResourceInfo.Properties.Name | ForEach-Object -Begin { $getDscResult = @{} } -Process { $getDscResult[$_] = $getResult.$_ }

                    # set the properties of the OUTPUT object from the result of Get-TargetResource
                    $addToActualState.properties = $getDscResult
                }
                catch {
                    $trace = @{'Debug' = 'ERROR: ' + $_.Exception.Message } | ConvertTo-Json -Compress
                    $host.ui.WriteErrorLine($trace)
                    exit 1
                }
            }
            Default {
                $trace = @{'Debug' = 'Can not find implementation of type: ' + $cachedDscResourceInfo.ImplementationDetail } | ConvertTo-Json -Compress
                $host.ui.WriteErrorLine($trace)
                exit 1
            }
        }

        return $addToActualState
    }
    else {
        $dsJSON = $DesiredState | ConvertTo-Json -Depth 10
        $errmsg = 'Can not find type "' + $DesiredState.type + '" for resource "' + $dsJSON + '". Please ensure that Get-DscResource returns this resource type.'
        $trace = @{'Debug' = 'ERROR: ' + $errmsg } | ConvertTo-Json -Compress
        $host.ui.WriteErrorLine($trace)
        exit 1
    }
}

# GetTypeInstanceFromModule function to get the type instance from the module
function GetTypeInstanceFromModule {
    param(
        [Parameter(Mandatory = $true)]
        [string] $modulename,
        [Parameter(Mandatory = $true)]
        [string] $classname
    )
    $instance = & (Import-Module $modulename -PassThru) ([scriptblock]::Create("'$classname' -as 'type'"))
    return $instance
}

# cached resource
class dscResourceCache {
    [string] $Type
    [psobject] $DscResourceInfo
}

# format expected for configuration and resource output
class dscResourceObject {
    [string] $name
    [string] $type
    [psobject] $properties
}

# dsc resource types
enum dscResourceType {
    ScriptBased
    ClassBased
    Binary
    Composite
}

# dsc resource type (settable clone)
class DscResourceInfo {
    [dscResourceType] $ImplementationDetail
    [string] $ResourceType
    [string] $Name
    [string] $FriendlyName
    [string] $Module
    [string] $ModuleName
    [string] $Version
    [string] $Path
    [string] $ParentPath
    [string] $ImplementedAs
    [string] $CompanyName
    [psobject[]] $Properties
}
