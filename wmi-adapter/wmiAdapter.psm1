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

        $type_fields = $r.type -split "/"
        $wmi_namespace = $type_fields[0].Replace('.', '\')
        $wmi_classname = $type_fields[1]

        switch ($Operation) {
            'Get' {
                # TODO: identify key properties and add WHERE clause to the query
                if ($r.properties) {
                    $query = "SELECT $($r.properties.psobject.properties.name -join ',') FROM $wmi_classname"
                    $where = " WHERE "
                    $useWhere = $false
                    $first = $true
                    foreach ($property in $r.properties.psobject.properties) {
                        # TODO: validate property against the CIM class to give better error message
                        if ($null -ne $property.value) {
                            $useWhere = $true
                            if ($first) {
                                $first = $false
                            } else {
                                $where += " AND "
                            }

                            if ($property.TypeNameOfValue -eq "System.String") {
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
                # TODO: implement set

            }
            'Test' {
                # TODO: implement test
            }
        }
    }

    return $result
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

    switch ($Operation) {
        'Get' {
            $addToActualState = GetCimSpace -Operation $Operation -DesiredState $DesiredState
        }
        'Set' {
            # TODO: Implement Set operation
        }
        'Test' {
            # TODO: Implement Test operation
        }
    }

    return $addToActualState
}

class dscResourceObject {
    [string] $name
    [string] $type
    [PSCustomObject] $properties
}