$VerbosePreference = 'SilentlyContinue'
$InformationPreference = 'SilentlyContinue'
$ProgressPreference = 'Continue'
$ErrorActionPreference = 'SilentlyContinue'

function Get-TargetResource {
    [System.Diagnostics.CodeAnalysis.SuppressMessageAttribute('PSAvoidGlobalVars', '')]
    [OutputType([Hashtable])]
    param (
        [Parameter(Mandatory)]
        [string] $Name,

        [Parameter(Mandatory = $true)]
        [System.Management.Automation.PSCredential]
        $Credential
    )
    Write-Verbose "[GET] Get Function running"
    return @{
            Name = $Name
            Credential = $Credential
    }
  
}

function Test-TargetResource {
    [System.Diagnostics.CodeAnalysis.SuppressMessageAttribute('PSAvoidGlobalVars', '')]
    [OutputType([System.Boolean])]
    param (
        [Parameter(Mandatory)]
        [string] $Name,

        [Parameter(Mandatory = $true)]
        [System.Management.Automation.PSCredential]
        $Credential

    )
    Write-Verbose "[TEST]Checking credentials"
    Write-Verbose "[TEST]Checking credentials UserName:  $($Credential.UserName)"
    Write-Verbose "[TEST]Checking credentials Password:  <redacted>"

   if ($null -eq $Credential) {
          $inDesiredState = $false
          return $false
        }

    if ($Credential.UserName -ne 'MyUser') {
            $inDesiredState = $false
    } else {
            $inDesiredState = $true
    }


    return $inDesiredState
    
}

function Set-TargetResource {
    [System.Diagnostics.CodeAnalysis.SuppressMessageAttribute('PSAvoidGlobalVars', '')]
    [CmdletBinding()]
    param (
        [Parameter(Mandatory)]
        [string] $Name,

        [Parameter(Mandatory = $true)]
        [System.Management.Automation.PSCredential]
        $Credential

    )

       if ($null -eq $Credential) {
          $inDesiredState = $false
          return $false
        }

        if ($Credential.UserName -ne 'MyUser') {
                $inDesiredState = $false
        } else {
                $inDesiredState = $true
        }

    Write-Verbose "[SET]Credential cannot be remediated by DSC."
        return $inDesiredState
}