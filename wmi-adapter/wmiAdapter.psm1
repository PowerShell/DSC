function Write-DscTrace
{
    param(
        [Parameter(Mandatory = $false)]
        [ValidateSet('Error', 'Warn', 'Info', 'Debug', 'Trace')]
        [string]$Operation = 'Debug',

        [Parameter(Mandatory = $true, ValueFromPipeline = $true)]
        [string]$Message
    )

    $trace = @{$Operation = $Message } | ConvertTo-Json -Compress
    $host.ui.WriteErrorLine($trace)
}

function Get-DscResourceObject
{
    param(
        [Parameter(Mandatory = $true, ValueFromPipeline = $true)]
        $jsonInput
    )
    # normalize the INPUT object to an array of dscResourceObject objects
    $inputObj = $jsonInput | ConvertFrom-Json -ErrorAction SilentlyContinue
    $desiredState = [System.Collections.Generic.List[Object]]::new()

    # catch potential for improperly formatted configuration input
    if ($inputObj.resources -and -not $inputObj.metadata.'Microsoft.DSC'.context -eq 'configuration')
    {
        $msg = 'The input has a top level property named "resources" but is not a configuration. If the input should be a configuration, include the property: "metadata": {"Microsoft.DSC": {"context": "Configuration"}}' 
        $msg | Write-DscTrace -Operation Warn
    }

    $adapterName = 'Microsoft.Windows/WMI'

    if ($null -ne $inputObj.metadata -and $null -ne $inputObj.metadata.'Microsoft.DSC' -and $inputObj.metadata.'Microsoft.DSC'.context -eq 'configuration')
    {
        # change the type from pscustomobject to dscResourceObject
        $inputObj.resources | ForEach-Object -Process {
            $desiredState += [dscResourceObject]@{
                name       = $_.name
                type       = $_.type
                properties = $_.properties
            }
        }
    }
    else
    {
        # mimic a config object with a single resource
        $type = $inputObj.adapted_dsc_type
        if (-not $type)
        {
            $errmsg = "Can not find " + $jsonInput + ". Please make sure the payload contains the 'adapted_dsc_type' key property."
            $errmsg | Write-DscTrace -Operation Error
            exit 1
        }

        $inputObj.psobject.properties.Remove('adapted_dsc_type')
        $desiredState += [dscResourceObject]@{
            name       = $adapterName
            type       = $type
            properties = $inputObj.properties
        }
    }
    return $desiredState
}

function GetCimSpace 
{
    [CmdletBinding()]
    param 
    (
        [Parameter(Mandatory)]
        [ValidateSet('Get', 'Set', 'Test', 'Export')]
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

    foreach ($r in $DesiredState)
    {
        $type_fields = $r.type -split "/"
        $wmi_namespace = $type_fields[0].Replace('.', '\')
        $wmi_classname = $type_fields[1]

        #TODO: add filtering based on supplied properties of $r
        $wmi_instances = Get-CimInstance -Namespace $wmi_namespace -ClassName $wmi_classname

        if ($wmi_instances)
        {
            $instance_result = @{}
            switch ($Operation)
            {
                'Get'
                {
                    $instance_result = @{}
                    $wmi_instance = $wmi_instances[0] # for 'Get' we return just first matching instance; for 'export' we return all instances
                    $wmi_instance.psobject.properties | ForEach-Object {
                        if (($_.Name -ne "type") -and (-not $_.Name.StartsWith("Cim")))
                        {
                            $instance_result[$_.Name] = $_.Value
                        }
                    }

                    # TODO: validate if we can set it to null
                    $addToActualState.CimInstance = $null
                }
                'Set'
                {
                    # TODO: with the wmi_instances now added on top, it becomes easier to apply some logic on the parameters available to Get-CimInstance
                    $wmi_instance = $wmi_instances[0]

                    # add the properties from INPUT
                    $instance_result = $r.properties

                    # return the Microsoft.Management.Infrastructure.CimInstance class
                    $addToActualState.CimInstance = $wmi_instance
                }
                'Test'
                {
                    # TODO: implement test
                }
                # TODO: validate output
                # 'Export'
                # {
                #     foreach ($wmi_instance in $wmi_instances) 
                #     {
                #         $wmi_instance.psobject.properties | ForEach-Object {
                #             if (($_.Name -ne "type") -and (-not $_.Name.StartsWith("Cim")))
                #             {
                #                 $instance_result[$_.Name] = $_.Value
                #             }
                #         }

                #         $addToActualState.properties += @($instance_result)
                #     }
                # }
            }

            return $addToActualState
        }
        else
        {
            $errmsg = "Can not find type " + $addToActualState.type + "; please ensure that Get-CimInstance returns this resource type"
            $errmsg | Write-DscTrace -Operation Error
            exit 1
        }
    }
}

function ValidateCimMethodAndArguments
{
    # TODO: whenever dsc exit codes come in add them, see: https://github.com/PowerShell/DSC/issues/421
    [CmdletBinding()]
    param 
    (
        [Parameter(Mandatory = $true, ValueFromPipeline = $true)]
        [dscResourceObject[]]
        $DesiredState

    )

    $inputObject = [System.Collections.Generic.List[hashtable]]@{}

    foreach ($r in $DesiredState)
    {
        $methodName = $r.properties.MethodName
        if (-not $methodName)
        {
            $errmsg = 'Can not find method name when calling ' + $DesiredState.type + '; Please add "MethodName" in input.' 
            'ERROR: ' + $errmsg | Write-DscTrace
            exit 1
        }

        $className = $r.type.Split("/")[-1]
        $namespace = $r.type.Split("/")[0].Replace(".", "/")
        $class = Get-CimClass -ClassName $className -Namespace $namespace

        $classMethods = $class.CimClassMethods.Name
        if ($classMethods -notcontains $methodName)
        {
            $errmsg = 'Method ' + ('"{0}"' -f $r.properties.MethodName) + ' was not found on ' + $r.type + "; Please ensure you call the correct method" 
            # $debugmsg = 'Available method(s) ' + ('{0}' -f ($class.CimClassMethods.Name | ConvertTo-Json -Compress))
            'ERROR: ' + $errmsg | Write-DscTrace
            #'DEBUG: ' + $debugmsg | Write-DscTrace
            exit 1 
        }

        $parameters = $class.CimClassMethods.parameters.Name
        $props = $r.properties | Get-Member | Where-Object { $_.MemberType -eq 'NoteProperty' } | Select-Object -ExpandProperty Name

        # TODO: can also validate if empty values are provided and which might be mandatory
        $arguments = @{}
        if (-not ($null -eq $props))
        {
            $props | ForEach-Object {
                $propertyName = $_
                if ($propertyName -notin $parameters)
                {
                    $msg = 'Parameter ' + $propertyName + " not found on $className." 
                    'WARNING: ' + $msg | Write-DscTrace
                }
                else 
                {
                    $arguments += @{$propertyName = $r.Properties.$propertyName }
                }
            }
        }
        
        # return hash table of parameters for InvokeCimMethod
        $inputObject += @{
            CimInstance = $r.CimInstance
            MethodName  = $methodName
            Arguments   = $arguments
        }
    }

    return $inputObject
}

function Invoke-DscWmi 
{
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

    $osVersion = [System.Environment]::OSVersion.VersionString
    'OS version: ' + $osVersion | Write-DscTrace

    $psVersion = $PSVersionTable.PSVersion.ToString()
    'PowerShell version: ' + $psVersion | Write-DscTrace

    switch ($Operation)
    {
        'Get'
        {
            $addToActualState = GetCimSpace -Operation $Operation -DesiredState $DesiredState
        }
        'Set'
        {
            # TODO: validate output
            $setState = GetCimSpace -Operation $Operation -DesiredState $DesiredState

            $wmiResources = ValidateCimMethodAndArguments -DesiredState $setState

            foreach ($resource in $wmiResources)
            {
                $addToActualState = InvokeCimMethod @resource
            }
        }
        'Test'
        {

        }
        'Export'
        {
            $addToActualState = GetCimSpace -Operation $Operation -DesiredState $DesiredState
        }
    }

    return $addToActualState
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

    if ($PSBoundParameters.ContainsKey('Arguments'))
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
        $errmsg = "Could not execute 'Invoke-CimMethod' with error message: " + $_.Exception.Message
        'ERROR: ' + $errmsg | Write-DscTrace
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
        'ERROR: ' + $errmsg | Write-DscTrace
        exit 1
    }

    return $invokeCimMethodResult
}

class dscResourceObject
{
    [string] $name
    [string] $type
    [PSCustomObject] $properties
    [Microsoft.Management.Infrastructure.CimInstance] $CimInstance
}