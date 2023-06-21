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
        [Parameter()]
        [ValidateSet('Present', 'Absent')]
        [System.String]
        $Ensure = 'Present',

        [Parameter(Mandatory = $true)]
        [System.String]
        $Name,

        [Parameter()]
        [System.String]
        $SourceLocation,

        [Parameter()]
        [System.String]
        $ScriptSourceLocation,

        [Parameter()]
        [System.String]
        $PublishLocation,

        [Parameter()]
        [System.String]
        $ScriptPublishLocation,

        [Parameter()]
        [ValidateSet('Trusted', 'Untrusted')]
        [System.String]
        $InstallationPolicy = 'Untrusted',

        [Parameter()]
        [System.String]
        $PackageManagementProvider = 'NuGet'
    )
<#
    Write-Verbose -Message ($localizedData.TestTargetResourceMessage -f $Name)

    $returnValue = $false

    $getTargetResourceResult = Get-TargetResource -Name $Name

    if ($Ensure -eq $getTargetResourceResult.Ensure) {
        if ($getTargetResourceResult.Ensure -eq 'Present' ) {
            $returnValue = Test-DscParameterState `
                -CurrentValues $getTargetResourceResult `
                -DesiredValues $PSBoundParameters `
                -ValuesToCheck @(
                'SourceLocation'
                'ScriptSourceLocation'
                'PublishLocation'
                'ScriptPublishLocation'
                'InstallationPolicy'
                'PackageManagementProvider'
            )
        }
        else {
            $returnValue = $true
        }
    }

    if ($returnValue) {
        Write-Verbose -Message ($localizedData.InDesiredState -f $Name)
    }
    else {
        Write-Verbose -Message ($localizedData.NotInDesiredState -f $Name)
    }

    return $returnValue#>
}

function Set-TargetResource {
    [CmdletBinding()]
    param
    (
        [Parameter()]
        [ValidateSet('Present', 'Absent')]
        [System.String]
        $Ensure = 'Present',

        [Parameter(Mandatory = $true)]
        [System.String]
        $Name,

        [Parameter()]
        [System.String]
        $SourceLocation,

        [Parameter()]
        [System.String]
        $ScriptSourceLocation,

        [Parameter()]
        [System.String]
        $PublishLocation,

        [Parameter()]
        [System.String]
        $ScriptPublishLocation,

        [Parameter()]
        [ValidateSet('Trusted', 'Untrusted')]
        [System.String]
        $InstallationPolicy = 'Untrusted',

        [Parameter()]
        [System.String]
        $PackageManagementProvider = 'NuGet'
    )

    <#$getTargetResourceResult = Get-TargetResource -Name $Name

    # Determine if the repository should be present or absent.
    if ($Ensure -eq 'Present') {
        $repositoryParameters = New-SplatParameterHashTable `
            -FunctionBoundParameters $PSBoundParameters `
            -ArgumentNames @(
            'Name'
            'SourceLocation'
            'ScriptSourceLocation'
            'PublishLocation'
            'ScriptPublishLocation'
            'InstallationPolicy'
            'PackageManagementProvider'
        )

        # Determine if the repository is already present.
        if ($getTargetResourceResult.Ensure -eq 'Present') {
            Write-Verbose -Message ($localizedData.RepositoryExist -f $Name)

            # Repository exist, update the properties.
            Set-PSRepository @repositoryParameters -ErrorAction 'Stop'
        }
        else {
            Write-Verbose -Message ($localizedData.RepositoryDoesNotExist -f $Name)

            # Repository did not exist, create the repository.
            Register-PSRepository @repositoryParameters -ErrorAction 'Stop'
        }
    }
    else {
        if ($getTargetResourceResult.Ensure -eq 'Present') {
            Write-Verbose -Message ($localizedData.RemoveExistingRepository -f $Name)

            # Repository did exist, remove the repository.
            Unregister-PSRepository -Name $Name -ErrorAction 'Stop'
        }
    }#>
}
