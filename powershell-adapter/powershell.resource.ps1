[CmdletBinding()]
param(
    [Parameter(Mandatory = $true, Position = 0, HelpMessage = 'Operation to perform. Choose from List, Get, Set, Test, Export, Validate.')]
    [ValidateSet('List', 'Get', 'Set', 'Test', 'Export', 'Validate')]
    [string]$Operation = 'Default',
    [Parameter(Mandatory = $false, Position = 1, ValueFromPipeline = $true, HelpMessage = 'Configuration or resource input in JSON format.')]
    [string]$stdinput = '@{}',
    [Parameter(Mandatory = $false, Position = 2, HelpMessage = 'Use Windows PowerShell 5.1 instead of PowerShell 7.')]
    [switch]$WinPS = $false
)

# If the OS is Windows, import the latest installed PSDesiredStateConfiguration module. For Linux/MacOS, only class based resources are supported and are called directly.
if ($IsWindows) {
    $DscModule = Get-Module -Name PSDesiredStateConfiguration -ListAvailable | Sort-Object -Property Version -Descending | Select-Object -First 1
    Import-Module $DscModule -DisableNameChecking -ErrorAction Ignore
        
    if ($null -eq $DscModule) {
        # Missing module is okay for listing resources
        if ($Operation -eq 'List') {
            Write-Warning 'The PowerShell adapter was called but the module PSDesiredStateConfiguration could not be found in PSModulePath. To install the module, run Install-PSResource -Name PSDesiredStateConfiguration'
            exit 0
        }
        else {
            Write-Error 'The PowerShell adapter was called but the module PSDesiredStateConfiguration could not be found in PSModulePath. To install the module, run Install-PSResource -Name PSDesiredStateConfiguration'
            exit 1
        }
    }
}

# Cache the results of Get-DscResource to optimize performance
function Invoke-CacheRefresh {
    # cache the results of Get-DscResource
    [resourceCache[]]$resourceCache = @()
    $DscResources = Get-DscResource
    foreach ($dsc in $DscResources) {
        if ($dsc.ModuleName) { $moduleName = $dsc.ModuleName }
        elseif ($dsc.ParentPath) { $moduleName = Split-Path $dsc.ParentPath | Split-Path | Split-Path -Leaf }

        $resourceCache += [resourceCache]@{
            Type            = "$moduleName/$($dsc.Name)"
            DscResourceInfo = $dsc
        }
    }
    return $resourceCache
}
$resourceCache = Invoke-CacheRefresh

# Convert the INPUT to a configFormat object so configuration and resource are standardized as moch as possible
function Get-ConfigObject {
    param(
        $stdinput
    )
    # normalize the INPUT object to an array of configFormat objects
    $inputObj = $stdInput | ConvertFrom-Json -Depth 10
    $desiredState = @()

    # catch potential for improperly formatted configuration input
    if ($inputObj.resources -and -not $inputObj.metadata.'Microsoft.DSC'.context -eq 'configuration') {
        Write-Warning 'The input has a top level property named "resources" but is not a configuration. If the input should be a configuration, include the property: "metadata": {"Microsoft.DSC": {"context": "Configuration"}}'
    }

    if ($inputObj.metadata.'Microsoft.DSC'.context -eq 'configuration') {
        # change the type from pscustomobject to configFormat
        $inputObj.resources | ForEach-Object -Process {
            $desiredState += [configFormat]@{
                name       = $_.name
                type       = $_.type
                properties = $_.properties
            }
        }
    }
    else {
        # mimic a config object with a single resource
        $type = $inputObj.type
        $inputObj.psobject.properties.Remove('type')
        $desiredState += [configFormat]@{
            name       = 'Microsoft.Dsc/PowerShell'
            type       = $type
            properties = $inputObj
        }
    }
    return $desiredState
}

# Get-ActualState function to get the actual state of the resource
function Get-ActualState {
    param(
        [Parameter(Mandatory)]
        [configFormat]$DesiredState,
        [Parameter(Mandatory)]
        [resourceCache[]]$ResourceCache
    )
    # get details from cache about the DSC resource, if it exists
    $cachedResourceInfo = $ResourceCache | Where-Object Type -EQ $DesiredState.type | ForEach-Object DscResourceInfo

    # if the resource is found in the cache, get the actual state
    if ($cachedResourceInfo) {

        # formated OUTPUT of each resource
        $addToActualState = [configFormat]@{}

        # set top level properties of the OUTPUT object from INPUT object
        $DesiredState.psobject.properties | ForEach-Object -Process {
            if ($_.TypeNameOfValue -EQ 'System.String') { $addToActualState.$($_.Name) = $DesiredState.($_.Name) }
        }

        # workaround: script based resources do not validate Get parameter consistency, so we need to remove any parameters the author chose not to include in Get-TargetResource
        if ($cachedResourceInfo.ImplementationDetail -EQ 'ScriptBased') {

            # imports the .psm1 file for the DSC resource as a PowerShell module and stores the list of parameters
            Import-Module -Scope Local -Name $cachedResourceInfo.path -Force -ErrorAction stop
            $validParams = (Get-Command -Module $cachedResourceInfo.ResourceType -Name 'Get-TargetResource').Parameters.Keys
            # prune any properties that are not valid parameters of Get-TargetResource
            $DesiredState.properties.psobject.properties | ForEach-Object -Process {
                if ($validParams -notcontains $_.Name) {
                    $DesiredState.properties.psobject.properties.Remove($_.Name)
                }
            }

            # morph the INPUT object into a hashtable named "property" for the cmdlet Invoke-DscResource
            $DesiredState.properties.psobject.properties | ForEach-Object -Begin { $property = @{} } -Process { $property[$_.Name] = $_.Value }

            # using the cmdlet from PSDesiredStateConfiguration module, and handle errors
            try {
                $getResult = Invoke-DscResource -Method Get -ModuleName $cachedResourceInfo.ModuleName -Name $cachedResourceInfo.Name -Property $property

                # set the properties of the OUTPUT object from the result of Get-TargetResource
                $addToActualState.properties = $getResult
            }
            catch {
                Write-Error $_.Exception.Message
                exit 1
            }
        }
        else {
            # TODO: simplify and use direct calls for class based resources
            $addToActualState.properties = @{"NotImplemented" = "true"}
        }

        return $addToActualState
    }
    else {
        $errmsg = 'Can not find type "' + $ds.type + '". Please ensure that Get-DscResource returns this resource type.'
        Write-Error $errmsg
        exit 1
    }
}

# initialize OUTPUT as array
$result = @()

# process the operation requested to the script
switch ($Operation) {
    'List' {
        # cache was refreshed on script load
        foreach ($Type in $resourceCache.Type) {
        
            # https://learn.microsoft.com/dotnet/api/system.management.automation.dscresourceinfo
            $r = $resourceCache | Where-Object Type -EQ $Type | ForEach-Object DscResourceInfo

            if ($null -ne $r.moduleName) {
                if ($WinPS) {
                    $requiresString = 'Microsoft.DSC/WindowsPowerShell'
                }
                if (-not $WinPS) {
                    $requiresString = 'Microsoft.DSC/PowerShell' 
                    # Binary resources are not supported in PowerShell 7
                    if ($r.ImplementedAs -EQ 'Binary') { continue }
                }
            }

            # OUTPUT dsc is expecting the following properties
            [resourceOutput]@{
                type          = $Type
                kind          = 'Resource'
                version       = $r.version.ToString()
                path          = $r.Path
                directory     = $r.ParentPath
                implementedAs = $r.ImplementationDetail
                author        = $r.CompanyName
                properties    = $r.Properties.Name
                requires      = $requiresString
            } | ConvertTo-Json -Compress
        }
    }
    'Get' {
        $desiredState = Get-ConfigObject -stdinput $stdinput
        foreach ($ds in $desiredState) {
            # process the INPUT (desiredState) for each resource as dscresourceInfo and return the OUTPUT as actualState
            $actualState = Get-ActualState -DesiredState $ds -ResourceCache $resourceCache
            # add resource actual state to the OUTPUT object
            $result += $actualState
        }
    
        # OUTPUT
        @{ result = $result } | ConvertTo-Json -Depth 10 -Compress
    }
    'Set' {
        throw 'SET not implemented'
        
        # OUTPUT
        $result += @{}
        @{ result = $result } | ConvertTo-Json -Depth 10 -Compress
    }
    'Test' {
        throw 'TEST not implemented'
        
        # OUTPUT
        $result += @{}
        @{ result = $result } | ConvertTo-Json -Depth 10 -Compress
    }
    'Export' {
        throw 'EXPORT not implemented'
        
        # OUTPUT
        $result += @{}
        @{ result = $result } | ConvertTo-Json -Depth 10 -Compress
    }
    'Validate' {
        # VALIDATE not implemented
        
        # OUTPUT
        @{ valid = $true } | ConvertTo-Json
    }
    Default {
        Write-Error 'Unsupported operation. Please use one of the following: List, Get, Set, Test, Export, Validate'
    }
}

# cached resource
class resourceCache {
    [string] $Type
    [psobject] $DscResourceInfo
}

# format expected for configuration and resource output
class configFormat {
    [string] $name
    [string] $type
    [psobject] $properties
}

# output format for resource list
class resourceOutput {
    [string] $type
    [string] $version
    [string] $path
    [string] $directory
    [string] $implementedAs
    [string] $author
    [string[]] $properties
    [string] $requires
    [string] $kind
}