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

    if ($DesiredState.properties) {
        $props = $DesiredState.properties.psobject.Properties | Where-Object { $_.Name -notin @('methodName', 'parameters') }
        $query = "SELECT $($props.Name -join ',') FROM $wmi_classname"
        $where = " WHERE "
        $useWhere = $false
        $first = $true
        foreach ($property in $props) {
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
                InvokeCimMethod @wmi_instance

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

    $methodName = $DesiredState.properties.psobject.properties | Where-Object -Property Name -EQ 'methodName' | Select-Object -ExpandProperty Value
    if (-not $methodName) {
        "'methodName' property is required for invoking a WMI/CIM method." | Write-DscTrace -Operation Error
        exit 1
    }

    # This is required for invoking a WMI/CIM method with parameters even if it is empty
    if (-not ($DesiredState.properties.psobject.properties | Where-Object -Property Name -EQ 'parameters')) {
        "'parameters' property is required for invoking a WMI/CIM method." | Write-DscTrace -Operation Error
        exit 1
    }

    $className = $DesiredState.type.Split("/")[-1]
    $namespace = $DesiredState.type.Split("/")[0].Replace(".", "/")

    $cimClass = Get-CimClass -Namespace $namespace -ClassName $className -MethodName $methodName

    $arguments = @{}
    if ($cimClass) {
        $parameters = ($DesiredState.properties.psobject.properties | Where-Object -Property Name -EQ 'parameters').Value
        $cimClassParameters = $cimClass.CimClassMethods | Where-Object -Property Name -EQ $methodName | Select-Object -ExpandProperty Parameters

        foreach ($param in $parameters.psobject.Properties.name) {
            if ($cimClassParameters.Name -notcontains $param) {
                # Only warn about invalid parameters, do not exit as this allows to action to continue when calling InvokeCimMethod
                "'$param' is not a valid parameter for method '$methodName' in class '$className'." | Write-DscTrace -Operation Warn
            } else {
                $arguments += @{
                    $param = $parameters.$param
                }
            }
        }

        $cimInstance = GetWmiInstance -DesiredState $DesiredState

        return @{
            CimInstance = $cimInstance
            Arguments   = $arguments
            MethodName  = $methodName
        }
    } else {
        "'$className' class not found in namespace '$namespace'." | Write-DscTrace -Operation Error
        exit 1
    }
}

function InvokeCimMethod
{
    [CmdletBinding()]
    [OutputType([Microsoft.Management.Infrastructure.CimMethodResult])]
    param
    (

        [Parameter(Mandatory = $true)]
        [Microsoft.Management.Infrastructure.CimInstance]
        $CimInstance,

        [Parameter(Mandatory = $true)]
        [System.String]
        $MethodName,

        [Parameter()]
        [System.Collections.Hashtable]
        $Arguments
    )

    $invokeCimMethodParameters = @{
        MethodName  = $MethodName
        ErrorAction = 'Stop'
    }

    if ($PSBoundParameters.ContainsKey('Arguments') -and $null -ne [string]::IsNullOrEmpty($Arguments))
    {
        $invokeCimMethodParameters['Arguments'] = $Arguments
    }

    try
    {
        $invokeCimMethodResult = $CimInstance | Invoke-CimMethod @invokeCimMethodParameters
    }
    catch [Microsoft.Management.Infrastructure.CimException]
    {
        $errMsg = $_.Exception.Message.Trim("")
        if ($errMsg -eq 'Invalid method')
        {
            "Retrying without instance" | Write-DscTrace -Operation Trace
            $invokeCimMethodResult = Invoke-CimMethod @invokeCimMethodParameters -ClassName $CimInstance[0].CimClass.CimClassName
        }
    }
    catch 
    {
        "Could not execute 'Invoke-CimMethod' with error message: " + $_.Exception.Message | Write-DscTrace -Operation Error
        exit 1
    }

    <#
        Successfully calling the method returns $invokeCimMethodResult.HRESULT -eq 0.
        If an general error occur in the Invoke-CimMethod, like calling a method
        that does not exist, returns $null in $invokeCimMethodResult.
    #>
    if ($invokeCimMethodResult.HRESULT)
    {
        $res = $invokeCimMethodResult.HRESULT
    }
    else 
    {
        $res = $invokeCimMethodResult.ReturnValue
    }
    if ($invokeCimMethodResult -and $res -ne 0)
    {
        if ($invokeCimMethodResult | Get-Member -Name 'ExtendedErrors')
        {
            <#
                The returned object property ExtendedErrors is an array
                so that needs to be concatenated.
            #>
            $errorMessage = $invokeCimMethodResult.ExtendedErrors -join ';'
        }
        else
        {
            $errorMessage = $invokeCimMethodResult.Error
        }

        $hResult = $invokeCimMethodResult.ReturnValue

        if ($invokeCimMethodResult.HRESULT)
        {
            $hResult = $invokeCimMethodResult.HRESULT
        }

        $errmsg = 'Method {0}() failed with an error. Error: {1} (HRESULT:{2})' -f @(
            $MethodName
            $errorMessage
            $hResult
        )
        $errMsg | Write-DscTrace -Operation Error
        exit 1
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