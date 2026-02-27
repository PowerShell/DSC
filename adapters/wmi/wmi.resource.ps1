# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true, Position = 0, HelpMessage = 'Operation to perform. Choose from List, Get, Set, Test, Validate.')]
    [ValidateSet('List', 'Get', 'Set', 'Test', 'Validate')]
    [string]$Operation,
    [Parameter(Mandatory = $false, Position = 1, ValueFromPipeline = $true, HelpMessage = 'Configuration or resource input in JSON format.')]
    [string]$jsonInput = '@{}'
)

# Read JSON input from stdin using $input automatic variable for operations that need it
if ($Operation -ne 'List') {
    $stdinData = $input | Out-String
    if (-not [string]::IsNullOrWhiteSpace($stdinData)) {
        $jsonInput = $stdinData
    }
}

# Import private functions
$wmiAdapter = Import-Module "$PSScriptRoot\wmiAdapter.psm1" -Force -PassThru

if ('Validate' -ne $Operation) {
    # initialize OUTPUT as array
    $result = [System.Collections.Generic.List[Object]]::new()

    Write-DscTrace -Operation Debug -Message "jsonInput=$jsonInput"
}

# Adding some debug info to STDERR
'PSVersion=' + $PSVersionTable.PSVersion.ToString() | Write-DscTrace
'PSPath=' + $PSHome | Write-DscTrace
'PSModulePath=' + $env:PSModulePath | Write-DscTrace

switch ($Operation) {
    'List' {
        $clases = Get-CimClass

        foreach ($r in $clases) {
            $version_string = ""
            $author_string = ""
            $description = ""

            $propertyList = @()
            foreach ($p in $r.CimClassProperties) {
                if ($p.Name) {
                    $propertyList += $p.Name
                }
            }

            $namespace = $r.CimSystemProperties.Namespace.ToLower().Replace('/', '.')
            $classname = $r.CimSystemProperties.ClassName
            $fullResourceTypeName = "$namespace/$classname"
            $requiresString = "Microsoft.Windows/WMI"

            # OUTPUT dsc is expecting the following properties
            [resourceOutput]@{
                type           = $fullResourceTypeName
                kind           = 'resource'
                version        = $version_string
                capabilities   = @('get', 'set', 'test')
                path           = ""
                directory      = ""
                implementedAs  = ""
                author         = $author_string
                properties     = $propertyList
                requireAdapter = $requiresString
                description    = $description
            } | ConvertTo-Json -Compress
        }
    }
    { @('Get', 'Set', 'Test') -contains $_ } {
        $desiredState = $wmiAdapter.invoke(   { param($jsonInput) Get-DscResourceObject -jsonInput $jsonInput }, $jsonInput )
        if ($null -eq $desiredState) {
            "Failed to create configuration object from provided input JSON." | Write-DscTrace -Operation Error
            exit 1
        }

        foreach ($ds in $desiredState) {
            # process the INPUT (desiredState) for each resource as dscresourceInfo and return the OUTPUT as actualState
            $actualstate = $wmiAdapter.Invoke( { param($op, $ds) Invoke-DscWmi -Operation $op -DesiredState $ds }, $Operation, $ds)
            if ($null -eq $actualState) {
                "Incomplete GET for resource $($ds.Type)" | Write-DscTrace -Operation Error
                exit 1
            }

            $result += $actualstate
        }

        # OUTPUT json to stderr for debug, and to stdout
        "jsonOutput=$($result | ConvertTo-Json -Depth 10 -Compress)" | Write-DscTrace -Operation Debug
        return (@{ result = $result } | ConvertTo-Json -Depth 10 -Compress)
    }
    'Validate' {
        # TODO: VALIDATE not implemented
        
        # OUTPUT
        @{ valid = $true } | ConvertTo-Json
    }
    Default {
        Write-DscTrace -Operation Error -Message 'Unsupported operation. Please use one of the following: List, Get, Set, Test, Export, Validate'
    }
}

# output format for resource list
class resourceOutput {
    [string] $type
    [string] $kind
    [string] $version
    [string[]] $capabilities
    [string] $path
    [string] $directory
    [string] $implementedAs
    [string] $author
    [string[]] $properties
    [string] $requireAdapter
    [string] $description
}
