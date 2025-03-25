# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

[CmdletBinding()]
param(
    [ValidateSet('List','Get','Set','Test','Validate')]
    $Operation = 'List',
    [Parameter(ValueFromPipeline)]
    $stdinput
)

# catch any un-caught exception and write it to the error stream
trap {
    Write-Trace -Level Error -message $_.Exception.Message
    exit 1
}

$ProgressPreference = 'Ignore'
$WarningPreference = 'Ignore'
$VerbosePreference = 'Ignore'

function Write-Trace {
    param(
        [string]$message,
        [string]$level = 'Error'
    )

    $trace = [pscustomobject]@{
        $level.ToLower() = $message
    } | ConvertTo-Json -Compress

    $host.ui.WriteErrorLine($trace)
}

if ($Operation -eq 'List')
{
    $clases = Get-CimClass

    foreach ($r in $clases)
    {
        $version_string = "";
        $author_string = "";

        $propertyList = @()
        foreach ($p in $r.CimClassProperties)
        {
            if ($p.Name)
            {
                $propertyList += $p.Name
            }
        }

        $namespace = $r.CimSystemProperties.Namespace.ToLower().Replace('/','.')
        $classname = $r.CimSystemProperties.ClassName
        $fullResourceTypeName = "$namespace/$classname"
        $requiresString = "Microsoft.Windows/WMI"

        $z = [pscustomobject]@{
            type = $fullResourceTypeName;
            kind = 'resource';
            version = $version_string;
            capabilities = @('get');
            path = "";
            directory = "";
            implementedAs = "";
            author = $author_string;
            properties = $propertyList;
            requireAdapter = $requiresString
        }

        $z | ConvertTo-Json -Compress
    }
}
elseif ($Operation -eq 'Get')
{
    $inputobj_pscustomobj = $null
    if ($stdinput)
    {
        $inputobj_pscustomobj = $stdinput | ConvertFrom-Json
    }

    $result = @()

    foreach($r in $inputobj_pscustomobj.resources)
    {
        $type_fields = $r.type -split "/"
        $wmi_namespace = $type_fields[0].Replace('.','\')
        $wmi_classname = $type_fields[1]

        # TODO: identify key properties and add WHERE clause to the query
        if ($r.properties)
        {
            $query = "SELECT $($r.properties.psobject.properties.name -join ',') FROM $wmi_classname"
            $where = " WHERE "
            $useWhere = $false
            $first = $true
            foreach ($property in $r.properties.psobject.properties)
            {
                # TODO: validate property against the CIM class to give better error message
                if ($null -ne $property.value)
                {
                    $useWhere = $true
                    if ($first)
                    {
                        $first = $false
                    }
                    else
                    {
                        $where += " AND "
                    }

                    if ($property.TypeNameOfValue -eq "System.String")
                    {
                        $where += "$($property.Name) = '$($property.Value)'"
                    }
                    else
                    {
                        $where += "$($property.Name) = $($property.Value)"
                    }
                }
            }
            if ($useWhere)
            {
                $query += $where
            }
            Write-Trace -Level Trace -message "Query: $query"
            $wmi_instances = Get-CimInstance -Namespace $wmi_namespace -Query $query -ErrorAction Stop
        }
        else
        {
            $wmi_instances = Get-CimInstance -Namespace $wmi_namespace -ClassName $wmi_classname -ErrorAction Stop
        }

        if ($wmi_instances)
        {
            $instance_result = @{}
            # TODO: for a `Get`, they key property must be provided so a specific instance is returned rather than just the first
            $wmi_instance = $wmi_instances[0] # for 'Get' we return just first matching instance; for 'export' we return all instances
            $wmi_instance.psobject.properties | %{
                if (($_.Name -ne "type") -and (-not $_.Name.StartsWith("Cim")))
                {
                    if ($r.properties)
                    {
                        if ($r.properties.psobject.properties.name -contains $_.Name)
                        {
                            $instance_result[$_.Name] = $_.Value
                        }
                    }
                    else
                    {
                        $instance_result[$_.Name] = $_.Value
                    }
                }
            }

            $result += @($instance_result)
        }
    }

    $result | ConvertTo-Json -Compress
}
elseif ($Operation -eq 'Validate')
{
    # TODO: this is placeholder
    @{ valid = $true } | ConvertTo-Json
}
else
{
    Write-Trace "ERROR: Unsupported operation requested from wmigroup.resource.ps1"
}
