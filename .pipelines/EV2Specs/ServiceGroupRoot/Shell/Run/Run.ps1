<#
.SYNOPSIS
    Publishes DSC Linux packages (RPM and DEB) to packages.microsoft.com (PMC).

.DESCRIPTION
    This script is executed by the EV2 shell extension during deployment. It:
    1. Downloads and extracts the mapping file, packages archive, and metadata
    2. Installs the pmc-cli tool
    3. Maps packages to their target PMC repositories
    4. Uploads and publishes each package to the appropriate repository

.NOTES
    Environment variables are set by EV2 from UploadLinux.Rollout.json:
    - MAPPING_FILE: URL to the mapping.json file
    - DSC_PACKAGES_TARGZIP: URL to the packages.tar.gz archive
    - PMC_METADATA: URL to the pmcMetadata.json file
#>

function Get-MappedRepositoryIds {
    param(
        [Parameter(Mandatory)]
        [hashtable]$Mapping,

        [Parameter(Mandatory)]
        $RepoList,

        [Parameter(Mandatory)]
        [ValidateSet('stable', 'preview')]
        [string]$Channel
    )

    $mappedRepos = @()
    foreach ($package in $Mapping.Packages) {
        $packageChannel = $package.channel
        if (-not $packageChannel) {
            $packageChannel = 'all'
        }

        if ($packageChannel -eq 'all' -or $packageChannel -eq $Channel) {
            $repoIds = [System.Collections.Generic.List[string]]::new()
            $packageFormat = $package.PackageFormat
            $extension = [System.IO.Path]::GetExtension($packageFormat)
            $packageType = $extension -replace '^\.'

            if ($package.distribution.count -gt 1) {
                throw "Package $($package | Out-String) has more than one Distribution."
            }

            foreach ($distribution in $package.distribution) {
                $urlGlob = $package.url
                switch ($packageType) {
                    'deb' { $urlGlob = $urlGlob + '-apt' }
                    'rpm' { $urlGlob = $urlGlob + '-yum' }
                    default { throw "Unknown package type: $packageType" }
                }

                Write-Verbose "Finding repo id for: $urlGlob" -Verbose
                $repos = $RepoList | Where-Object { $_.name -eq $urlGlob }

                if ($repos.id) {
                    $repoIds.AddRange(([string[]]$repos.id))
                } else {
                    throw "Could not find repo for $urlGlob"
                }

                if ($repoIds.Count -gt 0) {
                    $mappedRepos += ($package + @{ "RepoId" = $repoIds.ToArray() })
                }
            }
        }
    }

    Write-Verbose "Mapped repos count: $($mappedRepos.Length)" -Verbose
    return $mappedRepos
}

function Get-PackageObjects {
    param(
        [Parameter(Mandatory)]
        [psobject[]]$RepoObjects,

        [Parameter(Mandatory)]
        [string]$ReleaseVersion,

        [Parameter(Mandatory)]
        [string]$PackageName
    )

    $packages = @()

    foreach ($pkg in $RepoObjects) {
        if ($pkg.RepoId.count -gt 1) {
            throw "Package $($pkg.name) has more than one repo id."
        }

        if ($pkg.Distribution.count -gt 1) {
            throw "Package $($pkg.name) has more than one Distribution."
        }

        $pkgRepo = $pkg.RepoId | Select-Object -First 1
        $pkgDistribution = $pkg.Distribution | Select-Object -First 1

        $pkgName = $pkg.PackageFormat.Replace('PACKAGE_NAME', $PackageName).Replace('RELEASE_VERSION', $ReleaseVersion)

        if ($pkgName.EndsWith('.rpm')) {
            $pkgName = $pkgName.Replace($ReleaseVersion, $ReleaseVersion.Replace('-', '_'))
        }

        $packagePath = "$script:dscPackagesFolder/$pkgName"
        if (-not (Test-Path -Path $packagePath)) {
            throw "Package path $packagePath does not exist"
        }

        Write-Verbose "Package object: Name=$pkgName RepoId=$pkgRepo Distribution=$pkgDistribution" -Verbose
        $packages += @{
            PackagePath  = $packagePath
            PackageName  = $pkgName
            RepoId       = $pkgRepo
            Distribution = $pkgDistribution
        }
    }

    Write-Verbose "Total package objects: $($packages.Length)" -Verbose
    return $packages
}

function Publish-PackageToPMC {
    param(
        [Parameter(Mandatory)]
        [pscustomobject[]]$PackageObject,

        [Parameter(Mandatory)]
        [string]$ConfigPath,

        [Parameter(Mandatory)]
        [bool]$SkipPublish
    )

    $errorMessages = [System.Collections.Generic.List[string]]::new()

    foreach ($finalPackage in $PackageObject) {
        Write-Verbose "Staging package: $($finalPackage.PackageName)" -Verbose
        $packagePath = $finalPackage.PackagePath
        $pkgRepo = $finalPackage.RepoId

        $extension = [System.IO.Path]::GetExtension($packagePath)
        $packageType = $extension -replace '^\.'

        $packageListJson = pmc --config $ConfigPath package $packageType list --file $packagePath
        $list = $packageListJson | ConvertFrom-Json

        $packageId = @()
        if ($list.count -ne 0) {
            Write-Verbose "Package '$packagePath' already exists, skipping upload" -Verbose
            $packageId = $list.results.id | Select-Object -First 1
        } else {
            Write-Verbose "Uploading package: '$packagePath'" -Verbose
            $uploadResult = $null
            try {
                $uploadResult = pmc --config $ConfigPath package upload $packagePath --type $packageType
            } catch {
                $errorMessages.Add("Uploading package $($finalPackage.PackageName) to $pkgRepo failed. See errors above for details.")
                continue
            }

            $packageId = ($uploadResult | ConvertFrom-Json).id
        }

        Write-Verbose "Package ID: '$packageId'" -Verbose
        $distribution = $finalPackage.Distribution | Select-Object -First 1

        if (-not $SkipPublish) {
            Write-Verbose "Publishing package: $($finalPackage.PackageName) to $pkgRepo" -Verbose

            $rawUpdateResponse = $null
            try {
                if ($packageType -eq 'rpm') {
                    $rawUpdateResponse = pmc --config $ConfigPath repo package update $pkgRepo --add-packages $packageId
                } elseif ($packageType -eq 'deb') {
                    $rawUpdateResponse = pmc --config $ConfigPath repo package update $pkgRepo $distribution --add-packages $packageId
                } else {
                    throw "Unsupported package type: $packageType"
                }
            } catch {
                $errorMessages.Add("Update for package $($finalPackage.PackageName) to $pkgRepo failed. See errors above for details.")
                continue
            }

            $state = ($rawUpdateResponse | ConvertFrom-Json).state
            Write-Verbose "Update response state: $state" -Verbose
            if ($state -ne 'completed') {
                $errorMessages.Add("Publishing package $($finalPackage.PackageName) to $pkgRepo failed: $rawUpdateResponse")
                continue
            }

            $rawPublishResponse = $null
            try {
                $rawPublishResponse = pmc --config $ConfigPath repo publish $pkgRepo
            } catch {
                $errorMessages.Add("Final publish for package $($finalPackage.PackageName) to $pkgRepo failed. See errors above for details.")
                continue
            }

            $publishState = ($rawPublishResponse | ConvertFrom-Json).state
            Write-Verbose "Publish response state: $publishState" -Verbose
            if ($publishState -ne 'completed') {
                $errorMessages.Add("Final publishing of package $($finalPackage.PackageName) to $pkgRepo failed: $rawPublishResponse")
                continue
            }
        } else {
            Write-Verbose "Skipping publish for package '$($finalPackage.PackageName)' to '$pkgRepo'" -Verbose
        }
    }

    if ($errorMessages) {
        throw ($errorMessages -join [Environment]::NewLine)
    }
}

# Validate environment variables
if ($null -eq $env:MAPPING_FILE) {
    Write-Error "MAPPING_FILE environment variable is not set"
    return 1
}

if ($null -eq $env:DSC_PACKAGES_TARGZIP) {
    Write-Error "DSC_PACKAGES_TARGZIP environment variable is not set"
    return 1
}

if ($null -eq $env:PMC_METADATA) {
    Write-Error "PMC_METADATA environment variable is not set"
    return 1
}

try {
    $baseDir = '/package/unarchive'
    $mappingFilePath = Join-Path $baseDir 'mapping.json'
    $packagesTarPath = Join-Path $baseDir 'packages.tar.gz'
    $metadataFilePath = Join-Path $baseDir 'pmcMetadata.json'

    Write-Verbose "Downloading files to $baseDir" -Verbose
    Invoke-WebRequest -Uri $env:MAPPING_FILE -OutFile $mappingFilePath -ErrorAction Stop
    Invoke-WebRequest -Uri $env:DSC_PACKAGES_TARGZIP -OutFile $packagesTarPath -ErrorAction Stop
    Invoke-WebRequest -Uri $env:PMC_METADATA -OutFile $metadataFilePath -ErrorAction Stop

    # Extract packages
    Write-Verbose "Extracting packages.tar.gz" -Verbose
    $script:dscPackagesFolder = Join-Path $baseDir 'packages'
    New-Item -Path $script:dscPackagesFolder -ItemType Directory -Force > $null
    tar -xzvf $packagesTarPath -C $script:dscPackagesFolder --force-local
    Get-ChildItem $script:dscPackagesFolder -Recurse

    # pmc-cli config (shipped in the Run.tar archive)
    $configPath = Join-Path "$baseDir/Run" 'settings.toml'
    if (-not (Test-Path $configPath)) {
        Write-Error "settings.toml not found at $configPath"
        return 1
    }

    $pythonDlFolder = Join-Path "$baseDir/Run" 'python_dl'
    if (-not (Test-Path $pythonDlFolder)) {
        Write-Error "python_dl not found at $pythonDlFolder"
        return 1
    }

    Write-Verbose "Installing pmc-cli" -Verbose
    pip install --upgrade pip
    pip --version --verbose
    pip install $pythonDlFolder/*.whl

    # Read metadata
    $metadataContent = Get-Content -Path $metadataFilePath | ConvertFrom-Json
    $releaseVersion = $metadataContent.ReleaseTag.TrimStart('v')
    $skipPublish = $metadataContent.SkipPublish

    $channel = if ($releaseVersion.Contains('-')) { 'preview' } else { 'stable' }
    $packageName = 'dsc'

    Write-Verbose "Release version: $releaseVersion, Channel: $channel" -Verbose

    # Get PMC repository list
    Write-Verbose "Getting PMC repository list" -Verbose
    $rawResponse = pmc --config $configPath repo list --limit 800
    $response = $rawResponse | ConvertFrom-Json
    Write-Verbose "PMC repo list: limit=$($response.limit), count=$($response.count)" -Verbose
    $repoList = $response.results

    # Map packages to repositories
    Write-Verbose "Mapping packages to repositories" -Verbose
    $mapping = Get-Content -Raw -LiteralPath $mappingFilePath | ConvertFrom-Json -AsHashtable
    $mappedRepos = Get-MappedRepositoryIds -Mapping $mapping -RepoList $repoList -Channel $channel
    $packageObjects = Get-PackageObjects -RepoObjects $mappedRepos -PackageName $packageName -ReleaseVersion $releaseVersion

    Write-Verbose "SkipPublish: $skipPublish" -Verbose
    Publish-PackageToPMC -PackageObject $packageObjects -ConfigPath $configPath -SkipPublish $skipPublish
} catch {
    Write-Error -ErrorAction Stop $_.Exception.Message
    return 1
}

return 0
