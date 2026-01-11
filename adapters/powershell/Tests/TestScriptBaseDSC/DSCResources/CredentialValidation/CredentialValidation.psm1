function Get-TargetResource {
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
    Write-Verbose "[TEST]Checking credentials Password:  $($Credential.Password)"

   if ($null -eq $Credential) {
          throw 'Credential property is required'
          return $false
        }

    if ($Credential.UserName -ne 'MyUser') {
            throw 'Invalid user name'
            return $false
    } else {
            return $true
    }
    
}

function Set-TargetResource {
    [CmdletBinding()]
    param (
        [Parameter(Mandatory)]
        [string] $Name,

        [Parameter(Mandatory = $true)]
        [System.Management.Automation.PSCredential]
        $Credential

    )
    Write-Verbose "[SET]Credential cannot be remediated by DSC."
}