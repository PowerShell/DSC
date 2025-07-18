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
    [OutputType()]
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

    $keyProperty = ($availableProperties.flags -like "Property, Key*")

    if ($null -eq $availableProperties) {
        "No valid properties found in the CIM class '$ClassName' for the provided properties." | Write-DscTrace -Operation Error
        exit 1
    }

    if ($null -eq $keyProperty) {
        "Key property not provided for CIM class '$ClassName'." | Write-DscTrace -Operation Error
        exit 1
    }

    if ($ValidateKeyProperty.IsPresent) {
        # Check if any key property is also read-only
        $keyProps = $availableProperties | Where-Object { $_.Flags.ToString() -like "*Key*" }
        $readOnlyKeyProps = $keyProps | Where-Object { $_.Flags.ToString() -like "*ReadOnly*" }

        if ($readOnlyKeyProps.Count -eq $keyProps.Count) {
            "All key properties in the CIM class '$ClassName' are read-only, which is not supported." | Write-DscTrace -Operation Error
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

    # if ($SkipReadOnly.IsPresent) {
    #     # For 'Set', we need to validate that the provided properties match the CIM class
    #     $availableProperties = $cimClass.CimClassProperties | ForEach-Object {
    #         [string[]]$flags = $_.Flags.ToString().Split(",").Trim()
    #         if ($flags -notcontains 'ReadOnly' -or $flags -contains 'Key') {
    #             $_
    #         }
    #     }

    #     # Reset the validated properties list as we only want to capture non-readonly properties for 'Set'
    #     $validatedProperties = [System.Collections.Generic.List[Array]]::new()
    #     foreach ($property in $availableProperties) {
    #         $propName = $property.Name
    #         $isKey = $property.IsKey

    #         if ($isKey) {
    #             # Still check here if the key property is passed as we continue 
    #             if ($Properties.psobject.properties.name -notcontains $propName -or $null -eq $properties.$propName -or $Properties.$propName -eq '') {
    #                 "Key property '$propName' is required but not provided or is empty." | Write-DscTrace -Operation Error
    #                 exit 1
    #             } else {
    #                 $validatedProperties.Add($property)
    #             }
    #         } elseif ($Properties.psobject.Properties.name -contains $propName) {
    #             $validatedProperties.Add($property)
    #         } else {
    #             "Property '$propName' is not provided in the resource object." | Write-DscTrace -Operation Trace
    #         }
    #     }
    # }

    return $validatedProperties
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

        $query = "SELECT $($properties.Name -join ',') FROM $wmi_classname"
        $where = " WHERE "
        $useWhere = $false
        $first = $true
        foreach ($property in $properties) {
            # TODO: validate property against the CIM class to give better error message
            if ($null -ne $DesiredState.properties.$($property.Name)) {
                $useWhere = $true
                if ($first) {
                    $first = $false
                } else {
                    $where += " AND "
                }

                if ($property.CimType -eq "String") {
                    $where += "$($property.Name) = '$($DesiredState.properties.$($property.Name))'"
                } else {
                    $where += "$($property.Name) = $($DesiredState.properties.$($property.Name))"
                }
            }
        }
        if ($useWhere) {
            $query += $where
        }
        "Query: $query" | Write-DscTrace -Operation Debug
        $wmi_instances = Get-CimInstance -Namespace $wmi_namespace -Query $query -ErrorAction Ignore -ErrorVariable err
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
                }

            }
            'Set' {
                $wmi_instance = ValidateCimMethodAndArguments -DesiredState $r
                $properties = @{}

                $wmi_instance.Properties | ForEach-Object {
                    if ($r.properties.psobject.properties.name -contains $_.Name) {
                        $properties[$_.Name] = $r.properties.$($_.Name)
                    }
                }

                $readOnlyProperties = $wmi_instance.Properties | Where-Object -Property Flags -like "*ReadOnly*"

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

function ValidateCimMethodAndArguments {
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

$out = [dscResourceObject]@{
    name       = 'root.cimv2/Win32_Environment'
    type       = 'root.cimv2/Win32_Environment'
    properties = [PSCustomObject]@{
        UserName = "{0}\{1}" -f $env:USERDOMAIN, $env:USERNAME
        VariableValue = 'update'
        Name = 'test'
    }
}