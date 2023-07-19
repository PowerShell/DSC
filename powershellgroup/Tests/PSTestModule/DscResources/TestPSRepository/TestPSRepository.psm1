function Get-TargetResource {
    [CmdletBinding()]
    [OutputType([System.Collections.Hashtable])]
    param
    (
        [Parameter(Mandatory = $true)]
        [System.String]
        $Name
    )

    $returnValue = @{
        Ensure                    = 'Absent'
        Name                      = $Name
        SourceLocation            = $null
        ScriptSourceLocation      = $null
        PublishLocation           = $null
        ScriptPublishLocation     = $null
        InstallationPolicy        = $null
        PackageManagementProvider = $null
        Trusted                   = $false
        Registered                = $false
    }

    if ($Name -eq "TestPSRepository1") {
        $returnValue.Ensure = 'Present'
        $returnValue.SourceLocation = 'https://www.powershellgallery.com/api/v2'
        $returnValue.ScriptSourceLocation = 'https://www.powershellgallery.com/api/v2/items/psscript'
        $returnValue.PublishLocation = 'https://www.powershellgallery.com/api/v2/package/'
        $returnValue.ScriptPublishLocation = 'https://www.powershellgallery.com/api/v2/package/'
        $returnValue.InstallationPolicy = 'Untrusted'
        $returnValue.PackageManagementProvider = 'NuGet'
        $returnValue.Trusted = $False
        $returnValue.Registered = $True
    }
    else {
        Write-Verbose -Message ($localizedData.RepositoryNotFound -f $Name)
    }

    return $returnValue
}

function Test-TargetResource {
    [CmdletBinding()]
    [OutputType([System.Boolean])]
    param
    (
        [Parameter(Mandatory = $true)]
        [System.String]
        $Name,

        [Parameter()]
        [System.String]
        $PackageManagementProvider
    )
    
    if (($Name -eq "TestPSRepository1") -and ($PackageManagementProvider -eq 'NuGet'))
    {
        return $true
    }
    else
    {
        return $false
    }
}

function Set-TargetResource {
    [CmdletBinding()]
    param
    (
        [Parameter(Mandatory = $true)]
        [System.String]
        $Name,

        [Parameter()]
        [System.String]
        $PackageManagementProvider
    )
}
