# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.
#  Module adapted from 'PSDesiredStateConfiguration'

data LocalizedData
{
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

# In case localized resource is not available we revert back to English as defined in LocalizedData section so ignore the error instead of showing it to user.
Import-LocalizedData  -BindingVariable LocalizedData -FileName psDscAdapter.Resource.psd1 -ErrorAction Ignore

Import-Module $PSScriptRoot/helpers/DscResourceInfo.psm1

# Set DSC HOME environment variable.
$env:DSC_HOME = "$PSScriptRoot/Configuration"

$script:V1MetaConfigPropertyList = @('ConfigurationModeFrequencyMins', 'RebootNodeIfNeeded', 'ConfigurationMode', 'ActionAfterReboot', 'RefreshMode', 'CertificateID', 'ConfigurationID', 'DownloadManagerName', 'DownloadManagerCustomData', 'RefreshFrequencyMins', 'AllowModuleOverwrite', 'DebugMode', 'Credential')
$script:DirectAccessMetaConfigPropertyList = @('AllowModuleOverWrite', 'CertificateID', 'ConfigurationDownloadManagers', 'ResourceModuleManagers', 'DebugMode', 'RebootNodeIfNeeded', 'RefreshMode', 'ConfigurationAgent')

##############################################################
#
# Checks to see if a module defining composite resources should be reloaded
# based the last write time of the schema file. Returns true if the file exists
# and the last modified time was either not recorded or has change.
#
function Test-ModuleReloadRequired
{
    [OutputType([bool])]
    param (
        [Parameter(Mandatory)]
        [string]
        $SchemaFilePath
    )

    if (-not $SchemaFilePath -or  $SchemaFilePath -notmatch '\.schema\.psm1$')
    {
        # not a composite res
        return $false
    }

    # If the path doesn't exist, then we can't reload it.
    # Note: this condition is explicitly not an error for this function.
    if ( -not (Test-Path $SchemaFilePath))
    {
        if ($schemaFileLastUpdate.ContainsKey($SchemaFilePath))
        {
            $schemaFileLastUpdate.Remove($SchemaFilePath)
        }
        return $false
    }

    # If we have a modified date, then return it.
    if ($schemaFileLastUpdate.ContainsKey($SchemaFilePath))
    {
        if ( (Get-Item $SchemaFilePath).LastWriteTime -eq $schemaFileLastUpdate[$SchemaFilePath] )
        {
            return $false
        }
        else
        {
            return $true
        }
    }

    # Otherwise, record the last write time and return true.
    $script:schemaFileLastUpdate[$SchemaFilePath] = (Get-Item $SchemaFilePath).LastWriteTime
    $true
}
# Holds the schema file to lastwritetime mapping.
[System.Collections.Generic.Dictionary[string,DateTime]] $script:schemaFileLastUpdate =
New-Object -TypeName 'System.Collections.Generic.Dictionary[string,datetime]'

function ImportClassResourcesFromModule
{
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
    return ,$resourcesFound
}

function ImportCimAndScriptKeywordsFromModule
{
    param (
        [Parameter(Mandatory)]
        $Module,

        [Parameter(Mandatory)]
        $resource,

        $functionsToDefine
    )

    trap
    {
        continue
    }

    $SchemaFilePath = $null
    $oldCount = $functionsToDefine.Count

    $keywordErrors = New-Object -TypeName 'System.Collections.ObjectModel.Collection[System.Exception]'

    $foundCimSchema = [Microsoft.PowerShell.DesiredStateConfiguration.Internal.DscClassCache]::ImportCimKeywordsFromModule(
    $Module, $resource, [ref] $SchemaFilePath, $functionsToDefine, $keywordErrors)

    foreach($ex in $keywordErrors)
    {
        Write-Error -Exception $ex
        if($ex.InnerException)
        {
            Write-Error -Exception $ex.InnerException
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

    if ($foundScriptSchema -and $SchemaFilePath)
    {
        $resourceDirectory = Split-Path $SchemaFilePath
        if($null -ne $resourceDirectory)
        {
            Import-Module -Force: (Test-ModuleReloadRequired $SchemaFilePath) -Verbose:$false -Name $resourceDirectory -Global -ErrorAction SilentlyContinue
        }
    }

    return $foundCimSchema -or $foundScriptSchema
}


#------------------------------------
# Utility to throw an error/exception
#------------------------------------
function ThrowError
{
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

function Get-DSCResourceModules
{
    $listPSModuleFolders = $env:PSModulePath.Split([IO.Path]::PathSeparator)
    $dscModuleFolderList = [System.Collections.Generic.HashSet[System.String]]::new()

    foreach ($folder in $listPSModuleFolders)
    {
        if (!(Test-Path $folder))
        {
            continue
        }

        foreach($moduleFolder in Get-ChildItem $folder -Directory)
        {
            $addModule = $false

            $dscFolders = Get-ChildItem "$($moduleFolder.FullName)\DscResources","$($moduleFolder.FullName)\*\DscResources" -ErrorAction Ignore
            if($null -ne $dscFolders)
            {
                $addModule = $true
            }

            if(-not $addModule)
            {
                foreach($psd1 in Get-ChildItem -Recurse -Filter "$($moduleFolder.Name).psd1" -Path $moduleFolder.fullname -Depth 2)
                {
                    $containsDSCResource = Select-String -LiteralPath $psd1 -pattern '^[^#]*\bDscResourcesToExport\b.*'
                    if($null -ne $containsDSCResource)
                    {
                        $addModule = $true
                    }
                }
            }

            if($addModule)
            {
                $dscModuleFolderList.Add($moduleFolder.Name) | Out-Null
            }
        }
    }

    $dscModuleFolderList
}

###########################################################
#  Get-DSCResource
###########################################################

#
# Gets DSC resources on the machine. Allows to filter on a particular resource.
# It parses all the resources defined in the schema.mof file and also the composite
# resources defined or imported from PowerShell modules
#
function Get-DscResource
{
    [Diagnostics.CodeAnalysis.SuppressMessageAttribute("PSProvideCommentHelp", "", Scope="Function", Target="*")]
    [Diagnostics.CodeAnalysis.SuppressMessageAttribute("PSAvoidUsingPositionalParameters", "", Scope="Function", Target="*")]
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

    Begin
    {
        $initialized = $false
        $ModuleString = $null
        Write-Progress -Id 1 -Activity $LocalizedData.LoadingDefaultCimKeywords

        $keywordErrors = New-Object -TypeName 'System.Collections.ObjectModel.Collection[System.Exception]'

        # Load the default Inbox providers (keyword) in cache, also allow caching the resources from multiple versions of modules.
        [Microsoft.PowerShell.DesiredStateConfiguration.Internal.DscClassCache]::LoadDefaultCimKeywords($keywordErrors, $true)

        foreach($ex in $keywordErrors)
        {
            Write-Error -Exception $ex
            if($ex.InnerException)
            {
                Write-Error -Exception $ex.InnerException
            }
        }

        Write-Progress -Id 2 -Activity $LocalizedData.GettingModuleList

        $initialized = $true

        if($Module) #Pick from the specified module if there's one
        {
            $moduleSpecificName = [System.Management.Automation.LanguagePrimitives]::ConvertTo($Module,[Microsoft.PowerShell.Commands.ModuleSpecification])
            $modules = Get-Module -ListAvailable -FullyQualifiedName $moduleSpecificName

            if($Module -is [System.Collections.Hashtable])
            {
                $ModuleString = $Module.ModuleName
            }
            elseif($Module -is [Microsoft.PowerShell.Commands.ModuleSpecification])
            {
                $ModuleString = $Module.Name
            }
            else
            {
                $ModuleString = $Module
            }
        }
        else
        {
            $dscResourceModules = Get-DSCResourceModules
            if($null -ne $dscResourceModules) {
                $modules = Get-Module -ListAvailable -Name ($dscResourceModules)
            }
        }

        foreach ($mod in $modules)
        {
            if ($mod.ExportedDscResources.Count -gt 0)
            {
                $null = ImportClassResourcesFromModule -Module $mod -Resources * -functionsToDefine $functionsToDefine
            }

            $dscResources = Join-Path -Path $mod.ModuleBase -ChildPath 'DscResources'
            if(Test-Path $dscResources)
            {
                foreach ($resource in Get-ChildItem -Path $dscResources -Directory -Name)
                {
                    $null = ImportCimAndScriptKeywordsFromModule -Module $mod -Resource $resource -functionsToDefine $functionsToDefine
                }
            }
        }

        $Resources = @()
    }

    Process
    {
        try
        {
            if ($null -ne $Name)
            {
                $nameMessage = $LocalizedData.GetDscResourceInputName -f @('Name', [system.string]::Join(', ', $Name))
                Write-Verbose -Message $nameMessage
            }

            if(!$modules)
            {
                #Return if no modules were found with the required specification
                Write-Warning -Message $LocalizedData.NoModulesPresent
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
        catch
        {
            if ($initialized)
            {
                [System.Management.Automation.Language.DynamicKeyword]::Reset()
                [Microsoft.PowerShell.DesiredStateConfiguration.Internal.DscClassCache]::ClearCache()

                $initialized = $false
            }

            throw $_
        }
    }

    End
    {
        $Resources = $Resources | Sort-Object -Property Module, Name -Unique
        foreach ($resource in $Resources)
        {
            # return formatted string if required
            if ($Syntax)
            {
                GetSyntax $resource | Write-Output
            }
            else
            {
                Write-Output -InputObject $resource
            }
        }

        if ($initialized)
        {
            [System.Management.Automation.Language.DynamicKeyword]::Reset()
            [Microsoft.PowerShell.DesiredStateConfiguration.Internal.DscClassCache]::ClearCache()

            $initialized = $false
        }
    }
}

#
# Get DSC resoruce for a dynamic keyword
#
function GetResourceFromKeyword
{
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
    if ($matched -eq $false)
    {
        $message = $LocalizedData.ResourceNotMatched -f @($keyword.Keyword)
        Write-Verbose -Message ($message)
        return
    }
    else
    {
        $message = $LocalizedData.CreatingResource -f @($keyword.Keyword)
        Write-Verbose -Message $message
    }

    $resource = New-Object -TypeName Microsoft.PowerShell.DesiredStateConfiguration.DscResourceInfo

    $resource.ResourceType = $keyword.ResourceName

    if ($keyword.ResourceName -ne $keyword.Keyword)
    {
        $resource.FriendlyName = $keyword.Keyword
    }

    $resource.Name = $keyword.Keyword

    $schemaFiles = [Microsoft.PowerShell.DesiredStateConfiguration.Internal.DscClassCache]::GetFileDefiningClass($keyword.ResourceName)

    if ($schemaFiles.Count)
    {
        # Find the correct schema file that matches module name and version
        # if same module/version is installed in multiple locations, then pick the first schema file.
        foreach ($schemaFileName in $schemaFiles){
            $moduleInfo = GetModule $modules $schemaFileName;
            if ($moduleInfo.Name -eq $keyword.ImplementingModule -and $moduleInfo.Version -eq $keyword.ImplementingModuleVersion){
                break
            }
        }

        # if the class is not a resource we will ignore it except if it is DSC inbox resource.
        if(-not $schemaFileName.StartsWith("$env:windir\system32\configuration",[stringComparison]::OrdinalIgnoreCase))
        {
            $classesFromSchema = [Microsoft.PowerShell.DesiredStateConfiguration.Internal.DscClassCache]::GetCachedClassByFileName($schemaFileName)
            if($null -ne  $classesFromSchema)
            {
                # check if the resource is proper DSC resource that always derives from OMI_BaseResource.
                $schemaToProcess = $classesFromSchema | ForEach-Object -Process {
                    if(($_.CimSystemProperties.ClassName -ieq $keyword.ResourceName) -and ($_.CimSuperClassName -ieq 'OMI_BaseResource'))
                    {
                        $member = Get-Member -InputObject $_ -MemberType NoteProperty -Name 'ImplementationDetail'
                        if ($null -eq $member)
                        {
                            $_ | Add-Member -MemberType NoteProperty -Name 'ImplementationDetail' -Value $implementationDetail -PassThru
                        }
                        else
                        {
                            $_
                        }
                    }
                }
                if($null -eq  $schemaToProcess)
                {
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
    else
    {
        $implementationDetail = 'ClassBased'
        $Module = $modules | Where-Object -FilterScript {
            $_.Name -eq $keyword.ImplementingModule -and
            $_.Version -eq $keyword.ImplementingModuleVersion
        }

        if ($Module -and $Module.ExportedDscResources -contains $keyword.Keyword)
        {
            $implementationDetail = 'ClassBased'
            $resource.Module = $Module
            $resource.Path = $Module.Path
            $resource.ParentPath = Split-Path -Path $Module.Path
        }
    }

    if ([system.string]::IsNullOrEmpty($resource.Path) -eq $false)
    {
        $resource.ImplementedAs = [Microsoft.PowerShell.DesiredStateConfiguration.ImplementedAsType]::PowerShell
    }
    else
    {
        $implementationDetail = $null
        $resource.ImplementedAs = [Microsoft.PowerShell.DesiredStateConfiguration.ImplementedAsType]::Binary
    }

    if ($null -ne $resource.Module)
    {
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

#
# Gets composite resource
#
function GetCompositeResource
{
    [Diagnostics.CodeAnalysis.SuppressMessageAttribute("PSAvoidUsingPositionalParameters", "", Scope="Function", Target="*")]
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
    if ($matched -eq $false)
    {
        $message = $LocalizedData.ResourceNotMatched -f @($configInfo.Name)
        Write-Verbose -Message ($message)

        return $null
    }
    else
    {
        $message = $LocalizedData.CreatingResource -f @($configInfo.Name)
        Write-Verbose -Message $message
    }

    $resource = New-Object -TypeName Microsoft.PowerShell.DesiredStateConfiguration.DscResourceInfo

    $resource.ResourceType = $configInfo.Name
    $resource.FriendlyName = $null
    $resource.Name = $configInfo.Name
    $resource.ImplementedAs = [Microsoft.PowerShell.DesiredStateConfiguration.ImplementedAsType]::Composite

    if ($null -ne $configInfo.Module)
    {
        $resource.Module = GetModule $modules $configInfo.Module.Path
        if($null -eq $resource.Module)
        {
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

    $resource | Add-Member -MemberType NoteProperty -Name 'ImplementationDetail' -Value $null
    return $resource
}

#
# Adds property to a DSC resource
#
function AddDscResourceProperty
{
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
        'MSFT_Credential'='[PSCredential]';
        'MSFT_KeyValuePair'='[HashTable]';
        'MSFT_KeyValuePair[]'='[HashTable]'
    }

    $ignoreProperties = @('ResourceId', 'ConfigurationName')
    if ($ignoreProperties -contains $property.Name)
    {
        return
    }

    $dscProperty = New-Object -TypeName Microsoft.PowerShell.DesiredStateConfiguration.DscResourcePropertyInfo
    $dscProperty.Name = $property.Name
    if ($convertTypeMap.ContainsKey($property.TypeConstraint))
    {
        $type = $convertTypeMap[$property.TypeConstraint]
    }
    else
    {
        $Type = [System.Management.Automation.LanguagePrimitives]::ConvertTypeNameToPSTypeName($property.TypeConstraint)
        if ([string]::IsNullOrEmpty($Type)) {
            $dscResourceNames | ForEach-Object -Process {
                if (($property.TypeConstraint -eq $_) -or ($property.TypeConstraint -eq ($_ + "[]"))) { $Type = "[$($property.TypeConstraint)]" }
            }
        }
    }

    if ($null -ne $property.ValueMap)
    {
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

#
# Adds property to a DSC resource
#
function AddDscResourcePropertyFromMetadata
{
    param (
        [Parameter(Mandatory)]
        [Microsoft.PowerShell.DesiredStateConfiguration.DscResourceInfo]
        $dscresource,
        [Parameter(Mandatory)]
        [System.Management.Automation.ParameterMetadata]
        $parameter,
        $ignoreParameters
    )

    if ($ignoreParameters -contains $parameter.Name)
    {
        return
    }

    $dscProperty = New-Object -TypeName Microsoft.PowerShell.DesiredStateConfiguration.DscResourcePropertyInfo
    $dscProperty.Name = $parameter.Name

    # adding [] in Type name to keep it in sync with the name returned from LanguagePrimitives.ConvertTypeNameToPSTypeName
    $dscProperty.PropertyType = '[' +$parameter.ParameterType.Name + ']'
    $dscProperty.IsMandatory = $parameter.Attributes.Mandatory

    $dscresource.Properties.Add($dscProperty)
}

#
# Gets syntax for a DSC resource
#
function GetSyntax
{
    [OutputType('string')]
    param (
        [Parameter(Mandatory)]
        [Microsoft.PowerShell.DesiredStateConfiguration.DscResourceInfo]
        $dscresource
    )

    $output  = $dscresource.Name + " [String] #ResourceName`n"
    $output += "{`n"
    foreach ($property in $dscresource.Properties)
    {
        $output += '    '
        if ($property.IsMandatory -eq $false)
        {
            $output += '['
        }

        $output += $property.Name

        $output += ' = ' + $property.PropertyType + ''

        # Add possible values
        if ($property.Values.Count -gt 0)
        {
            $output += '{ ' +  [system.string]::Join(' | ', $property.Values) + ' }'
        }

        if ($property.IsMandatory -eq $false)
        {
            $output += ']'
        }

        $output += "`n"
    }

    $output += "}`n"

    return $output
}

#
# Checks whether a resource is found or not
#
function CheckResourceFound($names, $Resources)
{
    if ($null -eq $names)
    {
        return
    }

    $namesWithoutWildcards = $names | Where-Object -FilterScript {
        [System.Management.Automation.WildcardPattern]::ContainsWildcardCharacters($_) -eq $false
    }

    foreach ($Name in $namesWithoutWildcards)
    {
        $foundResources = $Resources | Where-Object -FilterScript {
            ($_.Name -eq $Name) -or ($_.ResourceType -eq $Name)
        }
        if ($foundResources.Count -eq 0)
        {
            $errorMessage = $LocalizedData.ResourceNotFound -f @($Name, 'Resource')
            Write-Error -Message $errorMessage
        }
    }
}

#
# Get implementing module path
#
function GetImplementingModulePath
{
    param (
        [Parameter(Mandatory)]
        [string]
        $schemaFileName
    )

    $moduleFileName = ($schemaFileName -replace ".schema.mof$", '') + '.psd1'
    if (Test-Path $moduleFileName)
    {
        return $moduleFileName
    }

    $moduleFileName = ($schemaFileName -replace ".schema.mof$", '') + '.psm1'
    if (Test-Path $moduleFileName)
    {
        return $moduleFileName
    }

    return
}

#
# Gets module for a DSC resource
#
function GetModule
{
    [OutputType('System.Management.Automation.PSModuleInfo')]
    param (
        [Parameter(Mandatory)]
        [System.Management.Automation.PSModuleInfo[]]
        $modules,
        [Parameter(Mandatory)]
        [string]
        $schemaFileName
    )

    if($null -eq $schemaFileName)
    {
        return $null
    }

    $schemaFileExt = $null
    if ($schemaFileName -match '.schema.mof')
    {
        $schemaFileExt = ".schema.mof$"
    }

    if ($schemaFileName -match '.schema.psm1')
    {
        $schemaFileExt = ".schema.psm1$"
    }

    if(!$schemaFileExt)
    {
        return $null
    }

    # get module from parent directory.
    # Desired structure is : <Module-directory>/DscResources/<schema file directory>/schema.File
    $validResource = $false
    $schemaDirectory = Split-Path $schemaFileName
    if($schemaDirectory)
    {
        $subDirectory = [System.IO.Directory]::GetParent($schemaDirectory)

        if ($subDirectory -and ($subDirectory.Name -eq 'DscResources') -and $subDirectory.Parent)
        {
            $results = $modules | Where-Object -FilterScript {
                $_.ModuleBase -eq $subDirectory.Parent.FullName
            }

            if ($results)
            {
                # Log Resource is internally handled by the CA. There is no formal provider for it.
                if ($schemaFileName -match 'MSFT_LogResource')
                {
                    $validResource = $true
                }
                else
                {
                    # check for proper resource module
                    foreach ($ext in @('.psd1', '.psm1', '.dll', '.cdxml'))
                    {
                        $resModuleFileName = ($schemaFileName -replace $schemaFileExt, '') + $ext
                        if(Test-Path($resModuleFileName))
                        {
                            $validResource = $true
                            break
                        }
                    }
                }
            }
        }
    }

    if ($results -and $validResource)
    {
        return $results[0]
    }
    else
    {
        return $null
    }
}

#
# Checks whether a resource is hidden or not
#
function IsHiddenResource
{
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

#
# Gets patterns for names
#
function GetPatterns
{
    [OutputType('System.Management.Automation.WildcardPattern[]')]
    param (
        [string[]]
        $names
    )

    $patterns = @()

    if ($null -eq $names)
    {
        return $patterns
    }

    foreach ($Name in $names)
    {
        $patterns += New-Object -TypeName System.Management.Automation.WildcardPattern -ArgumentList @($Name, [System.Management.Automation.WildcardOptions]::IgnoreCase)
    }

    return $patterns
}

#
# Checks whether an input name matches one of the patterns
# $pattern is not expected to have an empty or null values
#
function IsPatternMatched
{
    [OutputType('bool')]
    param (
        [System.Management.Automation.WildcardPattern[]]
        $patterns,
        [Parameter(Mandatory)]
        [string]
        $Name
    )

    if ($null -eq $patterns)
    {
        return $true
    }

    foreach ($pattern in $patterns)
    {
        if ($pattern.IsMatch($Name))
        {
            return $true
        }
    }

    return $false
}
function Invoke-DscResource
{
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
        [ValidateSet('Get','Set','Test')]
        [string]
        $Method,
        [Parameter(Mandatory)]
        [Hashtable]
        $Property
    )

    $getArguments = @{
        Name = $Name
    }

    if($Property.ContainsKey('PsDscRunAsCredential'))
    {
        $errorMessage = $LocalizedData.PsDscRunAsCredentialNotSupport -f $name
        $exception = [System.ArgumentException]::new($errorMessage,'Name')
        ThrowError -ExceptionName 'System.ArgumentException' -ExceptionMessage $errorMessage -ExceptionObject $exception -ErrorId 'PsDscRunAsCredentialNotSupport,Invoke-DscResource' -ErrorCategory InvalidArgument
    }

    if($ModuleName)
    {
        $getArguments.Add('Module',$ModuleName)
    }

    Write-Debug -Message "Getting DSC Resource $Name"
    $resource = @(psDscAdapter\Get-DscResource @getArguments -ErrorAction stop)

    if($resource.Count -eq 0)
    {
        throw "unexpected state - no resources found - get-dscresource should have thrown"
    }

    if($resource.Count -ne 1)
    {
        $errorMessage = $LocalizedData.InvalidResourceSpecification -f $name
        $exception = [System.ArgumentException]::new($errorMessage,'Name')
        ThrowError -ExceptionName 'System.ArgumentException' -ExceptionMessage $errorMessage -ExceptionObject $exception -ErrorId 'InvalidResourceSpecification,Invoke-DscResource' -ErrorCategory InvalidArgument
    }

    [Microsoft.PowerShell.DesiredStateConfiguration.DscResourceInfo] $resource = $resource[0]
    if($resource.ImplementedAs -ne 'PowerShell')
    {
        $errorMessage = $LocalizedData.UnsupportedResourceImplementation -f $name, $resource.ImplementedAs
        $exception = [System.InvalidOperationException]::new($errorMessage)
        ThrowError -ExceptionName 'System.InvalidOperationException' -ExceptionMessage $errorMessage -ExceptionObject $exception -ErrorId 'UnsupportedResourceImplementation,Invoke-DscResource' -ErrorCategory InvalidOperation
    }

    $resourceInfo = $resource |out-string
    Write-Debug $resourceInfo

    if($resource.ImplementationDetail -eq 'ClassBased')
    {
        Invoke-DscClassBasedResource -Resource $resource -Method $Method -Property $Property
    }
    else
    {
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

function Invoke-DscClassBasedResource
{
    [Diagnostics.CodeAnalysis.SuppressMessageAttribute("PSAvoidGlobalVars", "", Scope="Function")]
    [Diagnostics.CodeAnalysis.SuppressMessageAttribute("PSUseDeclaredVarsMoreThanAssignments", "", Scope="Function")]
    param(
        [Parameter(Mandatory)]
        [Microsoft.PowerShell.DesiredStateConfiguration.DscResourceInfo] $resource,
        [Parameter(Mandatory)]
        [ValidateSet('Get','Set','Test')]
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


    $null= $powershell.AddScript($script)
    $dscType=$powershell.Invoke() | Select-object -First 1
    foreach($key in $Property.Keys)
    {
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

function Invoke-DscScriptBasedResource
{
    [Diagnostics.CodeAnalysis.SuppressMessageAttribute("PSAvoidGlobalVars", "", Scope="Function")]
    [Diagnostics.CodeAnalysis.SuppressMessageAttribute("PSUseDeclaredVarsMoreThanAssignments", "", Scope="Function")]
    param(
        [Parameter(Mandatory)]
        [Microsoft.PowerShell.DesiredStateConfiguration.DscResourceInfo] $resource,
        [Parameter(Mandatory)]
        [ValidateSet('Get','Set','Test')]
        [string]
        $Method,
        [Hashtable]
        $Property
    )

    $path = $resource.Path
    $type = $resource.ResourceType

    Write-Debug "Importing $path ..."
    Import-module -Scope Local -Name $path -Force -ErrorAction stop

    $functionName = "$Method-TargetResource"

    Write-Debug "calling $name\$functionName ..."
    $global:DSCMachineStatus = $null
    $output = & $type\$functionName @Property
    return Get-InvokeDscResourceResult -Output $output -Method $Method
}

function Get-InvokeDscResourceResult
{
    [Diagnostics.CodeAnalysis.SuppressMessageAttribute("PSAvoidGlobalVars", "", Scope="Function")]
    param(
        $Output,
        $Method
    )

    switch($Method)
    {
        'Set' {
            $Output | Foreach-Object -Process {
                Write-Verbose -Message ('output: ' + $_)
            }
            $rebootRequired = if($global:DSCMachineStatus -eq 1) {$true} else {$false}
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

# Cache the results of Get-DscResource to optimize performance
function Invoke-CacheRefresh {
    param(
        [Parameter(Mandatory = $false)]
        [string[]] $module
    )
    # cache the results of Get-DscResource
    [resourceCache[]]$resourceCache = @()

    # improve by performance by having the option to only get details for named modules
    if ($null -ne $module) {
        if ($module.gettype().name -eq 'string') {
            $module = @($module)
        }
        $DscResources = @()
        $Modules = @()
        foreach ($m in $module) {
            $DscResources += psDscAdapter\Get-DscResource -Module $m
            $Modules += Get-Module -Name $m -ListAvailable
        }
    }
    else {
        $DscResources = psDscAdapter\Get-DscResource
        $Modules = Get-Module -ListAvailable
    }

    foreach ($dsc in $DscResources) {
        # only support known moduleType, excluding binary
        if ([moduleType].GetEnumNames() -notcontains $dsc.ImplementationDetail) {
            continue
        }
        # workaround: if the resource does not have a module name, get it from parent path
        # workaround: modulename is not settable, so clone the object without being read-only
        $DscResourceInfo = [DscResourceInfo]::new()
        $dsc.PSObject.Properties | ForEach-Object -Process { $DscResourceInfo.$($_.Name) = $_.Value }
        if ($dsc.ModuleName) {
            $moduleName = $dsc.ModuleName
        }
        elseif ($dsc.ParentPath) {
            # workaround: populate module name from parent path that is three levels up
            $moduleName = Split-Path $dsc.ParentPath | Split-Path | Split-Path -Leaf
            $DscResourceInfo.Module = $moduleName
            $DscResourceInfo.ModuleName = $moduleName
            # workaround: populate module version from psmoduleinfo if available
            if ($moduleInfo = $Modules | Where-Object { $_.Name -eq $moduleName }) {
                $moduleInfo = $moduleInfo | Sort-Object -Property Version -Descending | Select-Object -First 1
                $DscResourceInfo.Version = $moduleInfo.Version.ToString()
            }
        }

        $resourceCache += [resourceCache]@{
            Type            = "$moduleName/$($dsc.Name)"
            DscResourceInfo = $DscResourceInfo
        }
    }
    return $resourceCache
}

# Convert the INPUT to a configFormat object so configuration and resource are standardized as moch as possible
function Get-ConfigObject {
    param(
        [Parameter(Mandatory = $true, ValueFromPipeline = $true)]
        $jsonInput
    )
    # normalize the INPUT object to an array of configFormat objects
    $inputObj = $jsonInput | ConvertFrom-Json
    $desiredState = [System.Collections.Generic.List[Object]]::new()

    # catch potential for improperly formatted configuration input
    if ($inputObj.resources -and -not $inputObj.metadata.'Microsoft.DSC'.context -eq 'configuration') {
        Write-Warning 'The input has a top level property named "resources" but is not a configuration. If the input should be a configuration, include the property: "metadata": {"Microsoft.DSC": {"context": "Configuration"}}'
    }

    if ($null -ne $inputObj.metadata -and $null -ne $inputObj.metadata.'Microsoft.DSC' -and $inputObj.metadata.'Microsoft.DSC'.context -eq 'configuration') {
        # change the type from pscustomobject to configFormat
        $inputObj.resources.properties.resources | ForEach-Object -Process {
            $desiredState += [configFormat]@{
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
        $desiredState += [configFormat]@{
            name       = 'Microsoft.Dsc/PowerShell'
            type       = $type
            properties = $inputObj
        }
    }
    return $desiredState
}

# Get-ActualState function to get the actual state of the resource
function Get-ActualState {
    param(
        [Parameter(Mandatory, ValueFromPipeline = $true)]
        [configFormat]$DesiredState,
        [Parameter(Mandatory)]
        [resourceCache[]]$ResourceCache
    )
    # get details from cache about the DSC resource, if it exists
    $cachedResourceInfo = $ResourceCache | Where-Object Type -EQ $DesiredState.type | ForEach-Object DscResourceInfo

    # if the resource is found in the cache, get the actual state
    if ($cachedResourceInfo) {

        # formated OUTPUT of each resource
        $addToActualState = [configFormat]@{}

        # set top level properties of the OUTPUT object from INPUT object
        $DesiredState.psobject.properties | ForEach-Object -Process {
            if ($_.TypeNameOfValue -EQ 'System.String') { $addToActualState.$($_.Name) = $DesiredState.($_.Name) }
        }

        # workaround: script based resources do not validate Get parameter consistency, so we need to remove any parameters the author chose not to include in Get-TargetResource
        switch ([moduleType]$cachedResourceInfo.ImplementationDetail) {
            'ScriptBased' {

                # If the OS is Windows, import the embedded psDscAdapter module. For Linux/MacOS, only class based resources are supported and are called directly.
                if (!$IsWindows) {
                    Write-Error 'Script based resources are only supported on Windows.'
                    exit 1
                }

                # imports the .psm1 file for the DSC resource as a PowerShell module and stores the list of parameters
                Import-Module -Scope Local -Name $cachedResourceInfo.path -Force -ErrorAction stop
                $validParams = (Get-Command -Module $cachedResourceInfo.ResourceType -Name 'Get-TargetResource').Parameters.Keys
                # prune any properties that are not valid parameters of Get-TargetResource
                $DesiredState.properties.psobject.properties | ForEach-Object -Process {
                    if ($validParams -notcontains $_.Name) {
                        $DesiredState.properties.psobject.properties.Remove($_.Name)
                    }
                }

                # morph the INPUT object into a hashtable named "property" for the cmdlet Invoke-DscResource
                $DesiredState.properties.psobject.properties | ForEach-Object -Begin { $property = @{} } -Process { $property[$_.Name] = $_.Value }

                # using the cmdlet from psDscAdapter module, and handle errors
                try {
                    $getResult = psDscAdapter\Invoke-DscResource -Method Get -ModuleName $cachedResourceInfo.ModuleName -Name $cachedResourceInfo.Name -Property $property

                    # set the properties of the OUTPUT object from the result of Get-TargetResource
                    $addToActualState.properties = $getResult
                }
                catch {
                    Write-Error $_.Exception.Message
                    exit 1
                }
            }
            'ClassBased' {
                try {
                    # load powershell class from external module
                    $resource = Get-TypeInstanceFromModule -modulename $cachedResourceInfo.ModuleName -classname $cachedResourceInfo.Name
                    $resourceInstance = $resource::New()

                    # set each property of $resourceInstance to the value of the property in the $desiredState INPUT object
                    $DesiredState.properties.psobject.properties | ForEach-Object -Process {
                        $resourceInstance.$($_.Name) = $_.Value
                    }
                    $getResult = $resourceInstance.Get()

                    # set the properties of the OUTPUT object from the result of Get-TargetResource
                    $addToActualState.properties = $getResult
                }
                catch {
                    Write-Error $_.Exception.Message
                    exit 1
                }
            }
            Default {
                $errmsg = 'Can not find implementation of type: "' + $cachedResourceInfo.ImplementationDetail + '". If this is a binary resource such as File, use the Microsoft.Dsc/WindowsPowerShell adapter.'
                Write-Error $errmsg
                exit 1
            }
        }

        return $addToActualState
    }
    else {
        $dsJSON = $DesiredState | ConvertTo-Json -Depth 10
        $errmsg = 'Can not find type "' + $DesiredState.type + '" for resource "' + $dsJSON + '". Please ensure that Get-DscResource returns this resource type.'
        Write-Error $errmsg
        exit 1
    }
}

# Get-TypeInstanceFromModule function to get the type instance from the module
function Get-TypeInstanceFromModule {
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
class resourceCache {
    [string] $Type
    [psobject] $DscResourceInfo
}

# format expected for configuration and resource output
class configFormat {
    [string] $name
    [string] $type
    [psobject[]] $properties
}

# module types
enum moduleType {
    ScriptBased
    ClassBased
}

# dsc resource type (settable clone)
class DscResourceInfo {
    [moduleType] $ImplementationDetail
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
