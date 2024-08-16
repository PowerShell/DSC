# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

[CmdletBinding()]
param(
    [Parameter(Mandatory = $true, Position = 0, HelpMessage = 'Operation to perform. Choose from List, Get, Set, Test, Export, Validate.')]
    [ValidateSet('List', 'Get', 'Set', 'Test', 'Export', 'Validate')]
    [string]$Operation,
    [Parameter(Mandatory = $false, Position = 1, ValueFromPipeline = $true, HelpMessage = 'Configuration or resource input in JSON format.')]
    [string]$jsonInput = '@{}'
)

function Write-DscTrace
{
    param
    (
        [Parameter(Mandatory = $false)]
        [ValidateSet('Error', 'Warn', 'Info', 'Debug', 'Trace')]
        [string]$Operation = 'Debug',

        [Parameter(Mandatory = $true, ValueFromPipeline = $true)]
        [string]$Message
    )

    $trace = @{$Operation = $Message } | ConvertTo-Json -Compress
    $host.ui.WriteErrorLine($trace)
}

# Adding some debug info to STDERR
'PSVersion=' + $PSVersionTable.PSVersion.ToString() | Write-DscTrace
'PSPath=' + $PSHome | Write-DscTrace
'PSModulePath=' + $env:PSModulePath | Write-DscTrace

if ('Validate' -ne $Operation)
{
    # write $jsonInput to STDERR for debugging
    $trace = @{'Debug' = 'jsonInput=' + $jsonInput } | ConvertTo-Json -Compress
    $host.ui.WriteErrorLine($trace)
    $wmiAdapter = Import-Module "$PSScriptRoot/wmiAdapter.psd1" -Force -PassThru
    
    # initialize OUTPUT as array
    $result = [System.Collections.Generic.List[Object]]::new()
}

switch ($Operation)
{
    'List'
    {
        $clases = Get-CimClass

        foreach ($r in $clases)
        {
            $version_string = "";
            $author_string = "";
            $moduleName = "";
    
            $propertyList = @()
            foreach ($p in $r.CimClassProperties)
            {
                if ($p.Name)
                {
                    $propertyList += $p.Name
                }
            }
    
            # TODO: create class
            $methodList = [System.Collections.Generic.List[PSObject]]@()
            foreach ($m in $r.CimClassMethods)
            {
                $inputObject = [PSCustomObject]@{
                    methodName = $m.Name
                    parameters = @()
                }
                
                if ($m.Parameters)
                {
                    $inputObject.parameters = $m.Parameters.Name
                }
                $methodList += $inputObject
            }
    
            $namespace = $r.CimSystemProperties.Namespace.ToLower().Replace('/', '.')
            $classname = $r.CimSystemProperties.ClassName
            $fullResourceTypeName = "$namespace/$classname"
            $requiresString = "Microsoft.Windows/WMI"
    
            $z = [pscustomobject]@{
                type           = $fullResourceTypeName;
                kind           = 'Resource';
                version        = $version_string;
                capabilities   = @('Get', 'Set', 'Test', 'Export');
                # capabilities   = $methodList
                path           = "";
                directory      = "";
                implementedAs  = "";
                author         = $author_string;
                properties     = $propertyList;
                # TODO: Could not use methodsDetails because expected one of `type`, `kind`, `version`, `capabilities`, `path`, `description`, `directory`, `implementedAs`, `author`, `properties`, `requireAdapter`, `manifest`
                # Where is this coming from?
                # methodsDetails = $methodList
                requireAdapter = $requiresString
            }
    
            $z | ConvertTo-Json -Compress -Depth 10
        }
    }
    { @('Get', 'Set', 'Test', 'Export') -contains $_ }
    {
        
        $desiredState = $wmiAdapter.invoke(   { param($jsonInput) Get-DscResourceObject -jsonInput $jsonInput }, $jsonInput )
        if ($null -eq $desiredState)
        {
            $trace = @{'Debug' = 'ERROR: Failed to create configuration object from provided input JSON.' } | ConvertTo-Json -Compress
            $host.ui.WriteErrorLine($trace)
            exit 1
        }

        foreach ($ds in $desiredState)
        {
            # process the INPUT (desiredState) for each resource as dscresourceInfo and return the OUTPUT as actualState
            $actualstate = $wmiAdapter.Invoke( { param($op, $ds) Invoke-DscWmi -Operation $op -DesiredState $ds }, $Operation, $ds)
            if ($null -eq $actualState)
            {
                $trace = @{'Debug' = 'ERROR: Incomplete GET for resource ' + $ds.type } | ConvertTo-Json -Compress
                $host.ui.WriteErrorLine($trace)
                exit 1
            }

            $result += $actualstate
        }

        # OUTPUT json to stderr for debug, and to stdout
        $result = @{ result = $result } | ConvertTo-Json -Depth 10 -Compress
        $trace = @{'Debug' = 'jsonOutput=' + $result } | ConvertTo-Json -Compress
        $host.ui.WriteErrorLine($trace)
        return $result
    }
    'Validate'
    {
        # VALIDATE not implemented
        
        # OUTPUT
        @{ valid = $true } | ConvertTo-Json
    }
    Default
    {
        Write-Error 'Unsupported operation. Please use one of the following: List, Get, Set, Test, Export, Validate'
    }
}