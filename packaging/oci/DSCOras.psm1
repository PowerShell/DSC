# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $true
$script:RegistryName = 'orasdscartifacts'
$script:RegistryServer = "$script:RegistryName.azurecr.io"
$script:ArtifactType = 'application/vnd.microsoft.dsc.resource.v1+zip'

function Invoke-OrasCommand {
    param(
        [string[]]$Arguments,
        [string]$InputText
    )

    if ($PSBoundParameters.ContainsKey('InputText')) {
        $InputText | & oras @Arguments
    } else {
        & oras @Arguments
    }
    if ($LASTEXITCODE -ne 0) {
        throw "oras $($Arguments[0]) failed with exit code $LASTEXITCODE."
    }
}

function Connect-DscRegistry {
    $null = Get-Command oras -ErrorAction Stop
    $null = Get-Command az -ErrorAction Stop
    $accessToken = $null
    try {
        $accessToken = (& az acr login --name $script:RegistryName --expose-token `
            --query accessToken --output tsv --only-show-errors | Out-String).Trim()
        if ($LASTEXITCODE -ne 0) {
            throw "az acr login failed with exit code $LASTEXITCODE. Sign in with az login before using DSCOras."
        }
        if ([string]::IsNullOrWhiteSpace($accessToken)) {
            throw 'az acr login returned an empty access token.'
        }
        Invoke-OrasCommand -Arguments @(
            'login', $script:RegistryServer,
            '--username', '00000000-0000-0000-0000-000000000000',
            '--password-stdin'
        ) -InputText $accessToken | Out-Host
    } finally {
        $accessToken = $null
    }
}

function Publish-Artifact {
    <#
    .SYNOPSIS
    Publishes resource manifests and shared implementation ZIPs as OCI artifacts.

    .DESCRIPTION
    Combines All and OS-specific ResourcePackageFiles definitions. Each resource gets
    resourceName-version tags, platform-specific child artifacts, and an OCI index.
    ZIPs contain only zipContents; bundled manifests are split into individual resource
    manifests. YAML manifests require the ConvertFrom-Yaml command.
    ORAS and Azure CLI must be installed. Sign in with az login before publishing;
    the module obtains an ACR access token and authenticates ORAS through stdin.

    .PARAMETER PackageName
    Resource package name from data.build.json.

    .PARAMETER OS
    Target operating system. Defaults to the host operating system.

    .PARAMETER Platforms
    OCI platforms to publish, using artifacts built for the corresponding Rust targets.
    When omitted, uses the host platform and bin/debug or bin/release artifacts.

    .PARAMETER Release
    Publishes release rather than debug artifacts.

    .EXAMPLE
    Publish-Artifact -PackageName Microsoft.Windows.Registry

    .EXAMPLE
    Publish-Artifact -PackageName Microsoft.OpenSSH.SSHD `
        -OS Windows -Platforms windows-amd64, windows-arm64 -Release

    .OUTPUTS
    None.
    #>
    [CmdletBinding()]
    param (
        [Parameter(Mandatory)]
        [ValidateNotNullOrEmpty()]
        [string]$PackageName,
        [ValidateSet('Windows', 'Linux', 'macOS')]
        [string]$OS = ($IsWindows ? 'Windows' : ($IsMacOS ? 'macOS' : 'Linux')),
        [ValidateSet(
            'linux-amd64', 'linux-arm64', 'darwin-amd64', 'darwin-arm64',
            'windows-amd64', 'windows-arm64'
        )]
        [string[]]$Platforms,
        [switch]$Release
    )

    Import-Module (Join-Path $PSScriptRoot '..' '..' 'helpers.build.psm1')
    $buildData = Import-DscBuildData
    $definitions = @(
        $buildData.ResourcePackageFiles.All[$PackageName]
        $buildData.ResourcePackageFiles.$OS[$PackageName]
    ) | Where-Object { $null -ne $_ }
    if (@($definitions).Count -eq 0) {
        throw "Package not found: $PackageName for OS $OS"
    }
    $manifests = @($definitions.manifests | Sort-Object -Unique)
    if ($manifests.Count -eq 0) {
        throw "No manifests defined for package '$PackageName'."
    }
    foreach ($section in @('All', 'Linux', 'macOS', 'Windows')) {
        $definition = $buildData.ResourcePackageFiles.$section[$PackageName]
        $buildData.ResourcePackageFiles.$section = @{}
        if ($null -ne $definition) {
            $buildData.ResourcePackageFiles.$section[$PackageName] = $definition
        }
    }

    $targets = @{
        'linux-amd64' = 'x86_64-unknown-linux-gnu'
        'linux-arm64' = 'aarch64-unknown-linux-gnu'
        'darwin-amd64' = 'x86_64-apple-darwin'
        'darwin-arm64' = 'aarch64-apple-darwin'
        'windows-amd64' = 'x86_64-pc-windows-msvc'
        'windows-arm64' = 'aarch64-pc-windows-msvc'
    }
    $platformOS = @{ linux = 'Linux'; darwin = 'macOS'; windows = 'Windows' }
    $useCurrent = -not $PSBoundParameters.ContainsKey('Platforms')
    if ($useCurrent) {
        $hostOS = $IsWindows ? 'windows' : ($IsMacOS ? 'darwin' : 'linux')
        $hostArchitecture = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString()
        $architecture = switch ($hostArchitecture) {
            'X64' { 'amd64' }
            'Arm64' { 'arm64' }
            default { throw "Unsupported host architecture: $hostArchitecture" }
        }
        $Platforms = @("$hostOS-$architecture")
    }
    if ($Platforms.Count -eq 0) {
        throw 'At least one platform must be specified.'
    }
    foreach ($platform in $Platforms) {
        $targetOS = $platformOS[$platform.Split('-')[0]]
        if ($targetOS -ne $OS) {
            throw "Platform '$platform' does not match OS '$OS'."
        }
    }

    $repository = "$script:RegistryServer/dsc/resource/" + $PackageName.ToLowerInvariant()
    $artifactType = $script:ArtifactType
    $created = [DateTime]::UtcNow.ToString('o')
    $prepared = [System.Collections.Generic.List[hashtable]]::new()
    $publications = @{}
    $indexDirectory = $null

    # Prepare every platform before publishing so missing artifacts cannot produce a partial index.
    foreach ($platform in ($Platforms | Select-Object -Unique)) {
        $target = $useCurrent ? 'current' : $targets[$platform]
        $artifactDirectory = Get-ArtifactDirectoryPath -Architecture $target -Release:$Release
        $platformResources = @{}
        foreach ($manifest in $manifests) {
            $source = Get-Item -LiteralPath (Join-Path $artifactDirectory.Bin $manifest)
            $content = Get-Content -LiteralPath $source.FullName -Raw
            if ($source.Extension -in @('.yaml', '.yml')) {
                $null = Get-Command ConvertFrom-Yaml -ErrorAction Stop
                $document = $content | ConvertFrom-Yaml
            } else {
                $document = $content | ConvertFrom-Json -AsHashtable -Depth 100
            }
            $resources = if ($document.type) {
                @($document)
            } else {
                @($document.resources) + @($document.adaptedResources)
            }
            if (@($resources | Where-Object { $null -ne $_ }).Count -eq 0) {
                throw "No resources found in manifest '$($source.FullName)'."
            }
            foreach ($resource in ($resources | Where-Object { $null -ne $_ })) {
                if ($resource.type -notmatch '^[^/]+/[^/]+$' -or [string]::IsNullOrWhiteSpace($resource.version)) {
                    throw "Missing or invalid resource type/version in '$($source.FullName)'."
                }
                $slug = $resource.type.Split('/')[1].ToLowerInvariant()
                $tag = "$slug-$($resource.version)"
                if ($tag -notmatch '^[a-z0-9_][a-zA-Z0-9_.-]{0,127}$') {
                    throw "Resource '$($resource.type)' produces an invalid OCI tag: $tag"
                }
                if ("$tag-$platform".Length -gt 128) {
                    throw "Platform-specific OCI tag exceeds 128 characters: $tag-$platform"
                }
                if ($platformResources.ContainsKey($tag)) {
                    throw "Duplicate resource tag '$tag' for platform '$platform'."
                }
                if ($publications.ContainsKey($tag) -and $publications[$tag].ResourceType -ne $resource.type) {
                    throw "Different resource types produce the same OCI tag '$tag'."
                }
                $annotations = @{
                    'org.opencontainers.image.created' = $created
                    'com.microsoft.dsc.kind' = ($resource.kind ? $resource.kind.ToLowerInvariant() : 'resource')
                    'com.microsoft.dsc.resource.id' = $resource.type
                    'com.microsoft.dsc.version' = $resource.version
                }
                if ($resource.author) {
                    $annotations['com.microsoft.dsc.publisher'] = $resource.author
                }
                if (-not $publications.ContainsKey($tag)) {
                    $publications[$tag] = @{
                        ResourceType = $resource.type
                        Annotations = $annotations
                        Children = [System.Collections.Generic.List[hashtable]]::new()
                    }
                }
                $platformResources[$tag] = @{
                    Tag = $tag
                    Platform = $platform
                    Resource = $resource
                    ManifestName = "$slug.dsc.resource.json"
                    Annotations = $annotations
                    ArtifactDirectory = $artifactDirectory
                }
            }
        }
        Build-DscResourcePackage -BuildData $buildData -ArtifactDirectory $artifactDirectory -Architecture $target
        $packagesRoot = Join-Path $artifactDirectory.Bin 'resource-packages'
        $indexDirectory ??= $packagesRoot
        foreach ($entry in $platformResources.Values) {
            $entry.ZipPath = Join-Path $packagesRoot "$PackageName.zip"
            $prepared.Add($entry)
        }
    }

    Connect-DscRegistry

    $stagingDirectory = Join-Path ([System.IO.Path]::GetTempPath()) ([guid]::NewGuid().ToString())
    $null = New-Item -ItemType Directory -Path $stagingDirectory
    try {
        foreach ($entry in $prepared) {
            $childTag = "$($entry.Tag)-$($entry.Platform)"
            $directory = Join-Path $stagingDirectory $childTag
            $null = New-Item -ItemType Directory -Path $directory
            $entry.Resource | ConvertTo-Json -Depth 100 |
                Set-Content -LiteralPath (Join-Path $directory $entry.ManifestName) -Encoding utf8NoBOM
            Copy-Item -LiteralPath $entry.ZipPath -Destination $directory
            $fileAnnotations = @{
                'com.microsoft.dsc.resource.id' = $entry.Resource.type
                'com.microsoft.dsc.resource.version' = $entry.Resource.version
            }
            $annotations = @{ '$manifest' = $entry.Annotations }
            $annotations[$entry.ManifestName] = $fileAnnotations
            $annotations["$PackageName.zip"] = $fileAnnotations
            $annotations | ConvertTo-Json -Depth 10 |
                Set-Content -LiteralPath (Join-Path $directory 'annotations.json') -Encoding utf8NoBOM

            Push-Location $directory
            try {
                Invoke-OrasCommand -Arguments @(
                    'push', "${repository}:$childTag",
                    '--artifact-type', $artifactType,
                    '--annotation-file', 'annotations.json',
                    '--export-manifest', (Join-Path $entry.ArtifactDirectory.Bin 'resource-packages' "$childTag.oci.json"),
                    "$($entry.ManifestName):application/json",
                    "$PackageName.zip`:application/zip"
                ) | Out-Host
                $descriptor = Invoke-OrasCommand -Arguments @(
                    'manifest', 'fetch', '--descriptor', "${repository}:$childTag"
                ) | ConvertFrom-Json
                if (-not $descriptor.mediaType -or -not $descriptor.digest -or $descriptor.size -le 0) {
                    throw "Invalid OCI descriptor returned for '$childTag'."
                }
            } finally {
                Pop-Location
            }
            $targetOS, $architecture = $entry.Platform.Split('-')
            $publications[$entry.Tag].Children.Add(@{
                mediaType = $descriptor.mediaType
                digest = $descriptor.digest
                size = $descriptor.size
                platform = @{ os = $targetOS; architecture = $architecture }
            })
        }
        foreach ($tag in ($publications.Keys | Sort-Object)) {
            $publication = $publications[$tag]
            $index = @{
                schemaVersion = 2
                mediaType = 'application/vnd.oci.image.index.v1+json'
                artifactType = $artifactType
                manifests = @($publication.Children.ToArray())
                annotations = $publication.Annotations
            }
            $indexPath = Join-Path $indexDirectory "$PackageName`_$tag.index.json"
            $index | ConvertTo-Json -Depth 20 | Set-Content -LiteralPath $indexPath -Encoding utf8NoBOM
            Invoke-OrasCommand -Arguments @('manifest', 'push', "${repository}:$tag", $indexPath) | Out-Host
        }
    } finally {
        Remove-Item -LiteralPath $stagingDirectory -Recurse -Force
    }
}

function Find-Artifact {
    <#
    .SYNOPSIS
    Finds available OCI artifacts for a DSC resource type.

    .DESCRIPTION
    Searches resource-package repositories and matches resource IDs in OCI indexes,
    including resources that share a package. Returns one metadata object per available
    version and platform without downloading manifests or implementation ZIPs.
    Uses Azure CLI authentication and ORAS; sign in with az login first.

    .PARAMETER ResourceType
    Full DSC resource type, such as Microsoft.Windows/RegistryList.

    .PARAMETER Version
    Optional exact resource version. Omit to return all available versions.

    .PARAMETER Platform
    Optional OCI platform, such as windows-arm64. Omit to return all available platforms.

    .EXAMPLE
    Find-Artifact -ResourceType Microsoft.Windows/RegistryList

    .EXAMPLE
    Find-Artifact -ResourceType Microsoft.Windows/Registry -Version 1.0.0 -Platform windows-arm64

    .OUTPUTS
    DSCOras.Artifact metadata objects containing resource type, version, platform,
    digest-pinned reference, index reference, descriptor details, and annotations.
    #>
    [CmdletBinding()]
    [OutputType('DSCOras.Artifact')]
    param(
        [Parameter(Mandatory)]
        [ValidatePattern('^[^/]+/[^/]+$')]
        [string]$ResourceType,
        [ValidateNotNullOrEmpty()]
        [string]$Version,
        [ValidateSet(
            'linux-amd64', 'linux-arm64', 'darwin-amd64', 'darwin-arm64',
            'windows-amd64', 'windows-arm64'
        )]
        [string]$Platform
    )

    Connect-DscRegistry
    $catalog = Invoke-OrasCommand -Arguments @(
        'repo', 'ls', $script:RegistryServer, '--format', 'json'
    ) | ConvertFrom-Json -AsHashtable -Depth 100
    if (-not $catalog.ContainsKey('repositories')) {
        throw 'ORAS returned an invalid repository catalog.'
    }
    $tagPrefix = $ResourceType.Split('/')[1].ToLowerInvariant() + '-'
    foreach ($name in ($catalog.repositories | Sort-Object -Unique)) {
        if (-not $name.StartsWith('dsc/resource/', [System.StringComparison]::Ordinal)) {
            continue
        }
        $repository = "$script:RegistryServer/$name"
        $listing = Invoke-OrasCommand -Arguments @(
            'repo', 'tags', $repository, '--format', 'json'
        ) | ConvertFrom-Json -AsHashtable -Depth 100
        if (-not $listing.ContainsKey('tags')) {
            throw "ORAS returned an invalid tag listing for '$repository'."
        }
        foreach ($tag in ($listing.tags | Sort-Object -Unique)) {
            if (-not $tag.StartsWith($tagPrefix, [System.StringComparison]::Ordinal) -or
                ($Version -and $tag -cne "$tagPrefix$Version")) {
                continue
            }
            $indexReference = "${repository}:$tag"
            $index = Invoke-OrasCommand -Arguments @(
                'manifest', 'fetch', $indexReference
            ) | ConvertFrom-Json -AsHashtable -Depth 100
            if ($index.mediaType -ne 'application/vnd.oci.image.index.v1+json' -or
                $index.artifactType -ne $script:ArtifactType -or
                $index.annotations.'com.microsoft.dsc.resource.id' -ne $ResourceType) {
                continue
            }
            $resourceVersion = $index.annotations.'com.microsoft.dsc.version'
            if ([string]::IsNullOrWhiteSpace($resourceVersion) -or -not $index.manifests) {
                throw "Missing resource version or platform descriptors in '$indexReference'."
            }
            if ($Version -and $resourceVersion -cne $Version) {
                continue
            }
            foreach ($descriptor in $index.manifests) {
                if (-not $descriptor.platform.os -or -not $descriptor.platform.architecture -or
                    -not $descriptor.digest -or -not $descriptor.mediaType -or $descriptor.size -le 0) {
                    throw "Invalid platform descriptor in '$indexReference'."
                }
                $availablePlatform = "$($descriptor.platform.os)-$($descriptor.platform.architecture)"
                if ($Platform -and $availablePlatform -ne $Platform) {
                    continue
                }
                [pscustomobject]@{
                    PSTypeName = 'DSCOras.Artifact'
                    ResourceType = $index.annotations.'com.microsoft.dsc.resource.id'
                    Version = $resourceVersion
                    Platform = $availablePlatform
                    Reference = "$repository@$($descriptor.digest)"
                    IndexReference = $indexReference
                    Digest = $descriptor.digest
                    MediaType = $descriptor.mediaType
                    Size = $descriptor.size
                    Annotations = $index.annotations
                }
            }
        }
    }
}

Export-ModuleMember -Function Publish-Artifact, Find-Artifact
