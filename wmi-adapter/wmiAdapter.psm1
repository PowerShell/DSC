# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

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

function Get-DscResourceObject {
    param(
        [Parameter(Mandatory = $true, ValueFromPipeline = $true)]
        $jsonInput
    )
    # normalize the INPUT object to an array of dscResourceObject objects
    $inputObj = $jsonInput | ConvertFrom-Json
    $desiredState = [System.Collections.Generic.List[Object]]::new()

    $inputObj.resources | ForEach-Object -Process {
        $desiredState += [dscResourceObject]@{
            name       = $_.name
            type       = $_.type
            properties = $_.properties
        }
    }

    return $desiredState
}

function GetValidCimProperties {
    [CmdletBinding()]
    param 
    (
        [Parameter(Mandatory = $true, ValueFromPipeline = $true)]
        [Microsoft.Management.Infrastructure.CimClass]$CimClass,

        [Parameter(Mandatory = $true)]
        $ClassName,

        [Parameter()]
        [object]$Properties,

        [Parameter()]
        [switch] $SkipReadOnly,

        [Parameter()]
        [switch] $ValidateKeyProperty
    )

    $availableProperties = $CimClass.CimClassProperties | Where-Object -Property Name -in $Properties.psobject.Properties.name
    $validatedProperties = [System.Collections.Generic.List[Array]]::new()

    $keyProperties = $availableProperties | Where-Object {$_.Flags.Hasflag([Microsoft.Management.Infrastructure.CimFlags]::Key)}
    

    if ($null -eq $availableProperties) {
        "No valid properties found in the CIM class '$ClassName' for the provided properties." | Write-DscTrace -Operation Error
        exit 1
    }

    if ($ValidateKeyProperty.IsPresent) {
        # Check if any key property is also read-only
        if ($keyProperties.Count -eq 0) {
            "No key properties found in the CIM class '$ClassName'." | Write-DscTrace -Operation Error
            exit 1
        }
        $readOnlyKeyProps = $keyProperties | Where-Object { $_.Flags.HasFlag([Microsoft.Management.Infrastructure.CimFlags]::ReadOnly) }

        if ($readOnlyKeyProps.Count -eq $keyProperties.Count) {
            "All properties specified in the CIM class '$ClassName' are read-only, which is not supported." | Write-DscTrace -Operation Error
            exit 1
        }
    }

    # Check if the provided properties match the available properties in the CIM class
    # If the count of provided properties does not match the available properties, we log a warning but continue
    if ($properties.psobject.Properties.name.count -ne $availableProperties.Count) {
        $inputPropertyNames = $properties.psobject.Properties.Name
        $availablePropertyNames = $availableProperties.Name

        $missingProperties = $inputPropertyNames | Where-Object { $_ -notin $availablePropertyNames }
        if ($missingProperties) {
            foreach ($missing in $missingProperties) {
                "Property '$missing' was provided but not found in the CIM class '$($CimClass.ClassName)'." | Write-DscTrace -Operation Warn
            }
        }
    }

    $validatedProperties.Add($availableProperties)

    if ($SkipReadOnly.IsPresent) {   
        $availableProperties = foreach ($prop in $availableProperties) {
            [string[]]$flags = $prop.Flags.ToString().Split(",").Trim()
            if ($null -ne $properties.$($prop.Name)) {
                # Filter out read-only properties if SkipReadOnly is specified
                if ($flags -notcontains 'ReadOnly') {
                    $prop
                }
            } else {
                # Return $prop as if there is an empty value provided as property, we are not going to a WHERE clause
                $prop
            }
        }

        return $availableProperties
    }

    return $validatedProperties
}

function BuildWmiQuery {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)]
        [string]$ClassName,
        
        [Parameter(Mandatory = $true)]
        [array]$Properties,
        
        [Parameter(Mandatory = $true)]
        [psobject]$DesiredStateProperties,
        
        [Parameter()]
        [switch]$KeyPropertiesOnly
    )
    
    $targetProperties = if ($KeyPropertiesOnly.IsPresent) {
        $Properties | Where-Object {$_.Flags.HasFlag([Microsoft.Management.Infrastructure.CimFlags]::Key)}
    } else {
        $Properties
    }
    
    if ($targetProperties.Count -eq 0) {
        return $null
    }
    
    $query = "SELECT $($targetProperties.Name -join ',') FROM $ClassName"
    $whereClause = " WHERE "
    $useWhere = $false
    $isFirst = $true
    
    foreach ($property in $targetProperties) {
        if ($null -ne $DesiredStateProperties.$($property.Name)) {
            $useWhere = $true
            if ($isFirst) {
                $isFirst = $false
            } else {
                $whereClause += " AND "
            }
            
            if ($property.CimType -eq "String") {
                $whereClause += "$($property.Name) = '$($DesiredStateProperties.$($property.Name))'"
            } else {
                $whereClause += "$($property.Name) = $($DesiredStateProperties.$($property.Name))"
            }
        }
    }
    
    if ($useWhere) {
        $query += $whereClause
    }
    
    return $query
}

function GetWmiInstance {
    [CmdletBinding()]
    param 
    (
        [Parameter(Mandatory = $true, ValueFromPipeline = $true)]
        [psobject]$DesiredState
    )

    $type_fields = $DesiredState.type -split "/"
    $wmi_namespace = $type_fields[0].Replace('.', '\')
    $wmi_classname = $type_fields[1]

    $class = Get-CimClass -Namespace $wmi_namespace -ClassName $wmi_classname -ErrorAction Stop

    if ($DesiredState.properties) {
        $properties = GetValidCimProperties -CimClass $class -ClassName $wmi_classname -Properties $DesiredState.properties -SkipReadOnly

        $query = BuildWmiQuery -ClassName $wmi_classname -Properties $properties -DesiredStateProperties $DesiredState.properties

        if ($query) {
            "Query: $query" | Write-DscTrace -Operation Debug
            $wmi_instances = Get-CimInstance -Namespace $wmi_namespace -Query $query -ErrorAction Ignore -ErrorVariable err

            if ($null -eq $wmi_instances) {
                "No WMI instances found using query '$query'. Retrying with key properties only." | Write-DscTrace -Operation Debug
                $keyQuery = BuildWmiQuery -ClassName $wmi_classname -Properties $properties -DesiredStateProperties $DesiredState.properties -KeyPropertiesOnly

                if ($keyQuery) {
                    $wmi_instances = Get-CimInstance -Namespace $wmi_namespace -Query $keyQuery -ErrorAction Ignore -ErrorVariable err
                    if ($null -eq $wmi_instances) {
                        "No WMI instances found using key properties query '$keyQuery'." | Write-DscTrace -Operation Debug
                    }
                }
            }
        }
    } else {
        $wmi_instances = Get-CimInstance -Namespace $wmi_namespace -ClassName $wmi_classname -ErrorAction Ignore -ErrorVariable Err
    }

    if ($err) {
        "Error retrieving WMI instances: $($err.Exception.Message)" | Write-DscTrace -Operation Error
        exit 1
    }

    return $wmi_instances
}

function GetCimSpace {
    [CmdletBinding()]
    param 
    (
        [Parameter(Mandatory)]
        [ValidateSet('Get', 'Set', 'Test')]
        [System.String]
        $Operation,

        [Parameter(Mandatory, ValueFromPipeline = $true)]
        [psobject]
        $DesiredState
    )

    $addToActualState = [dscResourceObject]@{}
    $DesiredState.psobject.properties | ForEach-Object -Process {
        if ($_.TypeNameOfValue -EQ 'System.String') { $addToActualState.$($_.Name) = $DesiredState.($_.Name) }
    }

    $result = @()

    foreach ($r in $DesiredState) {

        switch ($Operation) {
            'Get' {
                $wmi_instances = GetWmiInstance -DesiredState $DesiredState

                if ($wmi_instances) {
                    $instance_result = [ordered]@{}
                    $wmi_instance = $wmi_instances[0] # for 'Get' we return just first matching instance; for 'export' we return all instances
                    $wmi_instance.psobject.properties | ForEach-Object {
                        if (($_.Name -ne "type") -and (-not $_.Name.StartsWith("Cim"))) {
                            if ($r.properties) {
                                if ($r.properties.psobject.properties.name -contains $_.Name) {
                                    $instance_result[$_.Name] = $_.Value
                                }
                            } else {
                                $instance_result[$_.Name] = $_.Value
                            }
                        }
                    }

                    $addToActualState.properties = $instance_result
                    $result += $addToActualState
                } else {
                    "No WMI instances found for type '$($r.type)'." | Write-DscTrace -Operation Debug
                    $addToActualState.properties = $null
                    $result += $addToActualState
                }
            }
            'Set' {
                $wmi_instance = GetCimInstanceProperties -DesiredState $r
                $properties = @{}

                $wmi_instance.Properties | ForEach-Object {
                    if ($r.properties.psobject.properties.name -contains $_.Name) {
                        $properties[$_.Name] = $r.properties.$($_.Name)
                    }
                }

                $readOnlyProperties = $wmi_instance.Properties | Where-Object {$_.Flags.HasFlag([Microsoft.Management.Infrastructure.CimFlags]::ReadOnly)}

                if ($null -eq $wmi_instance.CimInstance) {
                    $instance = New-CimInstance -Namespace $wmi_instance.Namespace -ClassName $wmi_instance.ClassName -Property $properties -ErrorAction Ignore -ErrorVariable err
                } else {
                    # When calling Set-CimInstance, the read-only properties needs to be filtered out
                    if ($readOnlyProperties) {
                        foreach ($prop in $readOnlyProperties) {
                            if ($properties.ContainsKey($prop.Name)) {
                                $properties.Remove($prop.Name) | Out-Null    
                            }
                        }
                    }
                    $wmi_instance.CimInstance | Set-CimInstance -Property $properties -ErrorAction Ignore -ErrorVariable err | Out-Null
                }

                $addToActualState = [dscResourceObject]@{
                    name       = $r.name
                    type       = $r.type
                    properties = $null
                }

                $result += $addToActualState
            }
            'Test' {
                # TODO: implement test
                "Test operation is not implemented for WMI/CIM methods." | Write-DscTrace -Operation Error 
                exit 1
            }
        }
    }

    return $result
}

function GetCimInstanceProperties {
    [CmdletBinding()]
    param (
        [Parameter(Mandatory = $true, ValueFromPipeline = $true)]
        [dscResourceObject]$DesiredState
    )

    $className = $DesiredState.type.Split("/")[-1]
    $namespace = $DesiredState.type.Split("/")[0].Replace(".", "/")

    $cimClass = Get-CimClass -Namespace $namespace -ClassName $className

    if ($null -eq $cimClass) {
        "Class '$className' not found in namespace '$namespace'." | Write-DscTrace -Operation Error 
        exit 1
    }

    $validatedProperties = GetValidCimProperties -CimClass $cimClass -ClassName $className -Properties $DesiredState.properties -ValidateKeyProperty

    $cimInstance = GetWmiInstance -DesiredState $DesiredState

    return @{
        CimInstance = $cimInstance
        Properties  = $validatedProperties
        ClassName   = $className
        Namespace   = $namespace
    }
}

function Invoke-DscWmi {
    [CmdletBinding()]
    param 
    (
        [Parameter(Mandatory)]
        [ValidateSet('Get', 'Set', 'Test', 'Export')]
        [System.String]
        $Operation,

        [Parameter(Mandatory, ValueFromPipeline = $true)]
        [dscResourceObject]
        $DesiredState
    )

    $addToActualState = GetCimSpace -Operation $Operation -DesiredState $DesiredState

    return $addToActualState
}

class dscResourceObject {
    [string] $name
    [string] $type
    [PSCustomObject] $properties
}