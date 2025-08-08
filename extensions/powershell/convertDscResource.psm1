function Write-DscTrace {
    param(
        [Parameter(Mandatory = $false)]
        [ValidateSet('Error', 'Warn', 'Info', 'Debug', 'Trace')]
        [string]$Operation = 'Debug',
        [Parameter(Mandatory = $true, ValueFromPipeline = $true)]
        [string]$Message
    )

    $trace = @{$Operation.ToLower() = $Message } | ConvertTo-Json -Compress
    $host.ui.WriteErrorLine($trace)
}

function Build-DscConfigDocument
{
    [CmdletBinding()]
    [OutputType([System.Collections.Specialized.OrderedDictionary])]
    param
    (
        [Parameter(Mandatory = $true)]
        [System.String]
        $Content
    )

    # declare configuration document
    $configurationDocument = [ordered]@{
        "`$schema" = "https://aka.ms/dsc/schemas/v3/bundled/config/document.json"
        resources  = @()
    }

    # convert object to hashtable(s)
    $dscObjects = ConvertTo-DscObject @PSBoundParameters -ErrorAction SilentlyContinue

    if (-not $dscObjects -or $dscObjects.Count -eq 0)
    {
        "No DSC objects found in the provided content." | Write-DscTrace -Operation Error
        exit 1
    }

    # store all resources in variables
    $resources = [System.Collections.Generic.List[object]]::new()

    foreach ($dscObject in $dscObjects)
    {
        $resource = [PSCustomObject]@{
            name       = $dscObject.ResourceInstanceName
            type       = ("{0}/{1}" -f $dscObject.ModuleName, $dscObject.ResourceName)
            properties = @()
        }

        $properties = [ordered]@{}

        foreach ($dscObjectProperty in $dscObject.GetEnumerator())
        {
            if ($dscObjectProperty.Key -notin @('ResourceInstanceName', 'ResourceName', 'ModuleName', 'DependsOn', 'ConfigurationName', 'Type'))
            {
                $properties.Add($dscObjectProperty.Key, $dscObjectProperty.Value)
            }
        }

        # add properties
        $resource.properties = $properties

        if ($dscObject.ContainsKey('DependsOn') -and $dscObject.DependsOn)
        {
            $dependsOnKeys = $dscObject.DependsOn.Split("]").Replace("[", "")

            $previousGroupHash = $dscObjects | Where-Object { $_.ResourceName -eq $dependsOnKeys[0] -and $_.ResourceInstanceName -eq $dependsOnKeys[1] }
            if ($previousGroupHash)
            {
                $dependsOnString = "[resourceId('$("{0}/{1}" -f $previousGroupHash.ModuleName, $previousGroupHash.ResourceName)','$($previousGroupHash.ResourceInstanceName)')]"

                Write-Verbose -Message "Found '$dependsOnstring' for resource: $($dscObject.ResourceInstanceName)"
                # add it to the object
                $resource | Add-Member -MemberType NoteProperty -Name 'dependsOn' -Value @($dependsOnString)
            }
        }

        $resources.Add($resource)
    }

    $configurationDocument.resources = $resources

    return $configurationDocument
}

function ConvertTo-DscObject
{
    [CmdletBinding()]
    param
    (
        [Parameter(Mandatory = $true)]
        [System.String]
        $Content
    )

    $result = @()
    $Tokens = $null
    $ParseErrors = $null

    Get-ChildItem "C:\Windows\System32\WindowsPowerShell\v1.0\Modules" | Select-Object Name | Write-DscTrace -Operation Trace
    # Load the PSDesiredStateConfiguration module
    Import-Module -Name 'PSDesiredStateConfiguration' -RequiredVersion '1.1' -Force -ErrorAction stop -ErrorVariable $importModuleError
    if (-not [string]::IsNullOrEmpty($importModuleError)) {
        'Could not import PSDesiredStateConfiguration 1.1 in Windows PowerShell. ' + $importModuleError | Write-DscTrace -Operation Error
        exit 1
    }

    # Remove the module version information.
    $start = $Content.ToLower().IndexOf('import-dscresource')
    if ($start -ge 0)
    {
        $end = $Content.IndexOf("`n", $start)
        if ($end -gt $start)
        {
            $start = $Content.ToLower().IndexOf("-moduleversion", $start)
            if ($start -ge 0 -and $start -lt $end)
            {
                $Content = $Content.Remove($start, $end - $start)
            }
        }
    }

    # Rename the configuration node to ensure a valid name is used.
    $start = $Content.ToLower().IndexOf("`nconfiguration")
    if ($start -lt 0)
    {
        $start = $Content.ToLower().IndexOf(' configuration ')
    }
    if ($start -ge 0)
    {
        $end = $Content.IndexOf("`n", $start)
        if ($end -gt $start)
        {
            $start = $Content.ToLower().IndexOf(' ', $start + 1)
            if ($start -ge 0 -and $start -lt $end)
            {
                $Content = $Content.Remove($start, $end - $start)
                $Content = $Content.Insert($start, " TempDSCParserConfiguration")
            }
        }
    }

    $AST = [System.Management.Automation.Language.Parser]::ParseInput($Content, [ref]$Tokens, [ref]$ParseErrors)

    # Look up the Configuration definition ("")
    $Config = $AST.Find({ $Args[0].GetType().Name -eq 'ConfigurationDefinitionAst' }, $False)

    # Retrieve information about the DSC Modules imported in the config
    # and get the list of their associated resources.
    $ModulesToLoad = @()
    foreach ($statement in $config.body.ScriptBlock.EndBlock.Statements)
    {
        if ($null -ne $statement.CommandElements -and $null -ne $statement.CommandElements[0].Value -and `
                $statement.CommandElements[0].Value -eq 'Import-DSCResource')
        {
            $currentModule = @{}
            for ($i = 0; $i -le $statement.CommandElements.Count; $i++)
            {
                if ($statement.CommandElements[$i].ParameterName -eq 'ModuleName' -and `
                    ($i + 1) -lt $statement.CommandElements.Count)
                {
                    $moduleName = $statement.CommandElements[$i + 1].Value
                    $currentModule.Add('ModuleName', $moduleName)
                }
                elseif ($statement.CommandElements[$i].ParameterName -eq 'Module' -and `
                    ($i + 1) -lt $statement.CommandElements.Count)
                {
                    $moduleName = $statement.CommandElements[$i + 1].Value
                    $currentModule.Add('ModuleName', $moduleName)
                }
                elseif ($statement.CommandElements[$i].ParameterName -eq 'ModuleVersion' -and `
                    ($i + 1) -lt $statement.CommandElements.Count)
                {
                    $moduleVersion = $statement.CommandElements[$i + 1].Value
                    $currentModule.Add('ModuleVersion', $moduleVersion)
                }
            }
            $ModulesToLoad += $currentModule
        }
    }
    $DSCResources = @()
    foreach ($moduleToLoad in $ModulesToLoad)
    {
        $loadedModuleTest = Get-Module -Name $moduleToLoad.ModuleName -ListAvailable | Where-Object -FilterScript { $_.Version -eq $moduleToLoad.ModuleVersion }

        if ($null -eq $loadedModuleTest -and -not [System.String]::IsNullOrEmpty($moduleToLoad.ModuleVersion))
        {
            "Module {$($moduleToLoad.ModuleName)} version {$($moduleToLoad.ModuleVersion)} specified in the configuration isn't installed on the machine/agent. Install it by running: Install-Module -Name '$($moduleToLoad.ModuleName)' -RequiredVersion '$($moduleToLoad.ModuleVersion)'" | Write-DscTrace -Operation Error
            exit 1
        }
        else
        {
            if ($Script:IsPowerShellCore)
            {
                $currentResources = Get-PwshDscResource -Module $moduleToLoad.ModuleName
            }
            else
            {
                $currentResources = Get-DSCResource -Module $moduleToLoad.ModuleName
            }

            if (-not [System.String]::IsNullOrEmpty($moduleToLoad.ModuleVersion))
            {
                $currentResources = $currentResources | Where-Object -FilterScript { $_.Version -eq $moduleToLoad.ModuleVersion }
            }
            $DSCResources += $currentResources
        }
    }

    if ($DSCResources.Count -eq 0)
    {
        "No DSC resources found in the imported modules." | Write-DscTrace -Operation Error
        exit 1
    }

    # Drill down
    # Body.ScriptBlock is the part after "Configuration <InstanceName> {"
    # EndBlock is the actual code within that Configuration block
    # Find the first DynamicKeywordStatement that has a word "Node" in it, find all "NamedBlockAst" elements, these are the DSC resource definitions
    try
    {
        $resourceInstances = $Config.Body.ScriptBlock.EndBlock.Statements.Find({ $Args[0].GetType().Name -eq 'DynamicKeywordStatementAst' -and $Args[0].CommandElements[0].StringConstantType -eq 'BareWord' -and $Args[0].CommandElements[0].Value -eq 'Node' }, $False).commandElements[2].ScriptBlock.Find({ $Args[0].GetType().Name -eq 'NamedBlockAst' }, $False).Statements
    }
    catch
    {
        $resourceInstances = $Config.Body.ScriptBlock.EndBlock.Statements | Where-Object -FilterScript { $null -ne $_.CommandElements -and $_.CommandElements[0].Value -ne 'Import-DscResource' }
    }

    # Get the name of the configuration.
    $configurationName = $Config.InstanceName.Value

    $totalCount = 1
    foreach ($resource in $resourceInstances)
    {
        $currentResourceInfo = @{}

        # CommandElements
        # 0 - Resource Type
        # 1 - Resource Instance Name
        # 2 - Key/Pair Value list of parameters.
        $resourceType = $resource.CommandElements[0].Value
        $resourceInstanceName = $resource.CommandElements[1].Value

        $percent = ($totalCount / ($resourceInstances.Count) * 100)
        Write-Progress -Status "[$totalCount/$($resourceInstances.Count)] $resourceType - $resourceInstanceName" `
            -PercentComplete $percent `
            -Activity "Parsing Resources"
        $currentResourceInfo.Add("ResourceName", $resourceType)
        $currentResourceInfo.Add("ResourceInstanceName", $resourceInstanceName)
        $currentResourceInfo.Add("ModuleName", $ModulesToLoad.ModuleName)
        $currentResourceInfo.Add("ConfigurationName", $configurationName)

        # Get a reference to the current resource.
        $currentResource = $DSCResources | Where-Object -FilterScript { $_.Name -eq $resourceType }

        # Loop through all the key/pair value
        foreach ($keyValuePair in $resource.CommandElements[2].KeyValuePairs)
        {
            $isVariable = $false
            $key = $keyValuePair.Item1.Value

            if ($null -ne $keyValuePair.Item2.PipelineElements)
            {
                if ($null -eq $keyValuePair.Item2.PipelineElements.Expression.Value)
                {
                    if ($null -ne $keyValuePair.Item2.PipelineElements.Expression)
                    {
                        if ($keyValuePair.Item2.PipelineElements.Expression.StaticType.Name -eq 'Object[]')
                        {
                            $value = $keyValuePair.Item2.PipelineElements.Expression.SubExpression
                            $newValue = @()
                            foreach ($expression in $value.Statements.PipelineElements.Expression)
                            {
                                if ($null -ne $expression.Elements)
                                {
                                    foreach ($element in $expression.Elements)
                                    {
                                        if ($null -ne $element.VariablePath)
                                        {
                                            $newValue += "`$" + $element.VariablePath.ToString()
                                        }
                                        elseif ($null -ne $element.Value)
                                        {
                                            $newValue += $element.Value
                                        }
                                    }
                                }
                                else
                                {
                                    $newValue += $expression.Value
                                }
                            }
                            $value = $newValue
                        }
                        else
                        {
                            $value = $keyValuePair.Item2.PipelineElements.Expression.ToString()
                        }
                    }
                    else
                    {
                        $value = $keyValuePair.Item2.PipelineElements.Parent.ToString()
                    }

                    if ($value.GetType().Name -eq 'String' -and $value.StartsWith('$'))
                    {
                        $isVariable = $true
                    }
                }
                else
                {
                    $value = $keyValuePair.Item2.PipelineElements.Expression.Value
                }
            }

            # Retrieve the current property's type based on the resource's schema.
            $currentPropertyInResourceSchema = $currentResource.Properties | Where-Object -FilterScript { $_.Name -eq $key }
            $valueType = $currentPropertyInResourceSchema.PropertyType

            # If the value type is null, then the parameter doesn't exist
            # in the resource's schema and we throw a warning
            $propertyFound = $true
            if ($null -eq $valueType)
            {
                $propertyFound = $false
            }

            if ($propertyFound)
            {
                # If the current property is not a CIMInstance
                if (-not $valueType.StartsWith('[MSFT_') -and `
                        $valueType -ne '[string]' -and `
                        $valueType -ne '[string[]]' -and `
                        -not $isVariable)
                {
                    # Try to parse the value based on the retrieved type.
                    $scriptBlock = @"
                                    `$typeStaticMethods = $valueType | gm -static
                                    if (`$typeStaticMethods.Name.Contains('TryParse'))
                                    {
                                        $valueType::TryParse(`$value, [ref]`$value) | Out-Null
                                    }
"@
                    Invoke-Expression -Command $scriptBlock | Out-Null
                }
                elseif ($valueType -eq '[String]' -or $isVariable)
                {
                    if ($isVariable -and [Boolean]::TryParse($value.TrimStart('$'), [ref][Boolean]))
                    {
                        if ($value -eq "`$true")
                        {
                            $value = $true
                        }
                        else
                        {
                            $value = $false
                        }
                    }
                    else
                    {
                        $value = $value
                    }
                }
                elseif ($valueType -eq '[string[]]')
                {
                    # If the property is an array but there's only one value
                    # specified as a string (not specifying the @()) then
                    # we need to create the array.
                    if ($value.GetType().Name -eq 'String' -and -not $value.StartsWith('@('))
                    {
                        $value = @($value)
                    }
                }
                else
                {
                    $isArray = $false
                    if ($keyValuePair.Item2.ToString().StartsWith('@('))
                    {
                        $isArray = $true
                    }
                    if ($isArray)
                    {
                        $value = @($value)
                    }
                }
                $currentResourceInfo.Add($key, $value) | Out-Null
            }
        }

        $result += $currentResourceInfo
        $totalCount++
    }
    Write-Progress -Completed `
        -Activity "Parsing Resources"

    return [System.Array]$result
}