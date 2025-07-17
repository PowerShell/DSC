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

        [Parameter()]
        [object]$Properties,

        [Parameter()]
        [ValidateSet('Get', 'Set', 'Test')]
        [string]$Operation
    )

    $validatedProperties = [System.Collections.Generic.List[Object]]::new()

    switch ($Operation) {
        'Get' {
            # For 'Get', we don't need to validate properties, just return all properties
            $cimClass.CimClassProperties | ForEach-Object {
                $validatedProperties.Add([PSCustomObject]@{
                    Name       = $_.Name
                    Type       = $_.CimType.ToString()
                    IsKey      = $_.Flags -contains 'Key'
                    IsReadOnly = $_.Flags -contains 'ReadOnly'
                })
            }
        }
        'Set' {
            # For 'Set', we need to validate that the provided properties match the CIM class
            $availableProperties = $cimClass.CimClassProperties | ForEach-Object {
                [string[]]$flags = $_.Flags.ToString().Split(",").Trim()
                if ($flags -notcontains 'ReadOnly' -or $flags -contains 'Key') {
                    @{
                        Name       = $_.Name
                        Type       = $_.CimType
                        Flags      = $flags
                        IsKey      = $flags -contains 'Key'
                        IsReadOnly = $flags -contains 'ReadOnly' # This is to ensure we identify key read-only properties
                    }
                }
            }

            $validatedProperties = [System.Collections.Generic.List[Object]]::new()
            foreach ($property in $availableProperties) {
                $propName = $property.Name
                $isKey = $property.IsKey
                $isReadOnly = $property.IsReadOnly

                if ($isKey) {
                    if ($Properties.psobject.properties.name -notcontains $propName -or $null -eq $properties.$propName -or $Properties.$propName -eq '') {
                        "Key property '$propName' is required but not provided or is empty." | Write-DscTrace -Operation Error
                        exit 1
                    } else {
                        $validatedProperties.Add([PSCustomObject]@{
                                Name       = $propName
                                Value      = $Properties.$propName
                                Type       = $property.Type
                                IsReadOnly = $isReadOnly
                            })
                    }
                } elseif ($Properties.psobject.Properties.name -contains $propName) {
                    $validatedProperties.Add([PSCustomObject]@{
                            Name       = $propName
                            Value      = $Properties.$propName
                            Type       = $property.Type
                            IsReadOnly = $isReadOnly
                        })
                } else {
                    "Property '$propName' is not provided in the resource object." | Write-DscTrace -Operation Trace
                }
            }
        }
    }

    return $validatedProperties
}

function GetWmiInstance {
    [CmdletBinding()]
    param 
    (
        [Parameter(Mandatory = $true, ValueFromPipeline = $true)]
        [psobject]$DesiredState,

        [Parameter()]
        [ValidateSet('Get', 'Set', 'Test')]
        [string]$Operation = 'Get'
    )

    $type_fields = $DesiredState.type -split "/"
    $wmi_namespace = $type_fields[0].Replace('.', '\')
    $wmi_classname = $type_fields[1]

    $class = Get-CimClass -Namespace $wmi_namespace -ClassName $wmi_classname -ErrorAction Stop

    if ($DesiredState.properties) {
        $properties = GetValidCimProperties -CimClass $class -Properties $DesiredState.properties -Operation $Operation

        $query = "SELECT $($properties.Name -join ',') FROM $wmi_classname"
        $where = " WHERE "
        $useWhere = $false
        $first = $true
        foreach ($property in $properties) {
            # TODO: validate property against the CIM class to give better error message
            if ($null -ne $property.value) {
                $useWhere = $true
                if ($first) {
                    $first = $false
                } else {
                    $where += " AND "
                }

                if ($property.Type -eq "String") {
                    $where += "$($property.Name) = '$($property.Value)'"
                } else {
                    $where += "$($property.Name) = $($property.Value)"
                }
            }
        }
        if ($useWhere) {
            $query += $where
        }
        "Query: $query" | Write-DscTrace -Operation Debug
        $wmi_instances = Get-CimInstance -Namespace $wmi_namespace -Query $query -ErrorAction Stop
    } else {
        $wmi_instances = Get-CimInstance -Namespace $wmi_namespace -ClassName $wmi_classname -ErrorAction Stop
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
                # TODO: identify key properties and add WHERE clause to the query
                $wmi_instances = GetWmiInstance -DesiredState $DesiredState

                if ($wmi_instances) {
                    $instance_result = [ordered]@{}
                    # TODO: for a `Get`, they key property must be provided so a specific instance is returned rather than just the first
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
                        $properties[$_.Name] = $_.Value
                    }
                }

                $readOnlyProperties = $wmi_instance.Properties | Where-Object -Property IsReadOnly -eq $true

                if ($null -eq $wmi_instance.CimInstance) {
                    New-CimInstance -Namespace $wmi_instance.Namespace -ClassName $wmi_instance.ClassName -Property $properties -ErrorAction Stop
                } else {
                    # When calling Set-CimInstance, the read-only properties needs to be filtered out
                    if ($readOnlyProperties) {
                        foreach ($prop in $readOnlyProperties) {
                            if ($properties.ContainsKey($prop.Name)) {
                                $properties.Remove($prop.Name) | Out-Null    
                            }
                        }
                    }
                    $wmi_instance.CimInstance | Set-CimInstance -Property $properties -ErrorAction Stop
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

    $validatedProperties = GetValidCimProperties -CimClass $cimClass -Properties $DesiredState.properties -Operation Set

    $cimInstance = GetWmiInstance -DesiredState $DesiredState -Operation Set

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



# $out = [dscResourceObject]@{
#     name       = "root.cimv2/Win32_Environment"
#     type       = "root.cimv2/Win32_Environment"
#     properties = [PSCustomObject]@{
#         Name = "test"
#         VariableValue = "TestValue"
#         UserName = ("{0}\{1}" -f $env:USERDOMAIN, $env:USERNAME) # Read-only property required
#     }
# }

$out = [dscResourceObject]@{
    name       = "root.cimv2/Win32_Environment"
    type       = "root.cimv2/Win32_Environment"
    properties = [PSCustomObject]@{
        Name = 'test'
        VariableValue = 'TestValue'
        UserName = ("{0}\{1}" -f $env:USERDOMAIN, $env:USERNAME) # Read-only property required
    }
}