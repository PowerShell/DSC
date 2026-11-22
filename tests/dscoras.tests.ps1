# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Publish-Artifact' {
    BeforeEach {
        $fixtureRoot = Join-Path $TestDrive ([guid]::NewGuid().ToString())
        $modulePath = Join-Path $fixtureRoot 'packaging' 'oci' 'DSCOras.psm1'
        $null = New-Item -ItemType Directory -Path (Split-Path -Parent $modulePath) -Force
        Copy-Item -LiteralPath (Join-Path $PSScriptRoot '..' 'packaging' 'oci' 'DSCOras.psm1') -Destination $modulePath
        Copy-Item -LiteralPath (Join-Path $PSScriptRoot '..' 'helpers.build.psm1') -Destination $fixtureRoot
        Import-Module $modulePath -Force
        & (Get-Module DSCOras) {
            function script:oras {
                [CmdletBinding(PositionalBinding = $false)]
                param(
                    [Parameter(ValueFromRemainingArguments)][string[]]$Arguments,
                    [Parameter(ValueFromPipeline)][string]$InputToken
                )
                throw 'ORAS must be mocked in publishing tests.'
            }

            function script:az {
                param([Parameter(ValueFromRemainingArguments)][string[]]$Arguments)
                throw 'Azure CLI must be mocked in publishing tests.'
            }

            function script:ConvertFrom-Yaml {
                param([Parameter(ValueFromPipeline)][string]$Yaml)
                throw 'YAML conversion must be mocked in publishing tests.'
            }
        }
        $data = @{
            PackageFiles = @{ Windows = @(); Linux = @(); macOS = @(); Executable = @() }
            Projects = @()
            ResourcePackageFiles = @{
                All = @{
                    'Microsoft.Test.Bundle' = @{
                        zipContents = @('shared.exe')
                        manifests = @('one.dsc.resource.json', 'two.dsc.resource.json')
                    }
                }
                Windows = @{
                    'Microsoft.Test.Bundle' = @{
                        zipContents = @('shared.exe', 'nested\support.ps1')
                        manifests = @('one.dsc.resource.json', 'two.dsc.resource.json', 'windows.dsc.resource.json')
                    }
                }
                Linux = @{}
                macOS = @{}
            }
        }
        $data | ConvertTo-Json -Depth 20 |
            Set-Content -LiteralPath (Join-Path $fixtureRoot 'data.build.json')
        $bundle = @{
            resources = @(
                @{ type = 'Microsoft.Test/One'; version = '1.0.0'; author = 'Microsoft Corporation' }
                @{ type = 'Microsoft.Test/Two'; version = '2.0.0'; author = 'Other Author'; kind = 'adapter' }
            )
            actions = @(@{ type = 'Microsoft.Test/NotAResource'; version = '1.0.0' })
        }
        $bins = foreach ($target in @('current', 'x86_64-pc-windows-msvc', 'aarch64-pc-windows-msvc')) {
            $bin = if ($target -eq 'current') {
                Join-Path $fixtureRoot 'bin' 'debug'
            } else {
                Join-Path $fixtureRoot 'bin' $target 'debug'
            }
            $null = New-Item -ItemType Directory -Path (Join-Path $bin 'nested') -Force
            $null = New-Item -ItemType Directory -Path (Join-Path $bin 'resource-packages')
            Set-Content -LiteralPath (Join-Path $bin 'shared.exe') -Value 'binary'
            Set-Content -LiteralPath (Join-Path $bin 'nested\support.ps1') -Value 'support'
            Set-Content -LiteralPath (Join-Path $bin 'resource-packages' 'untouched.zip') -Value 'keep'
            $bundle | ConvertTo-Json -Depth 10 |
                Set-Content -LiteralPath (Join-Path $bin 'bundle.dsc.manifests.json')
            $bundle.resources[0] | ConvertTo-Json -Depth 10 |
                Set-Content -LiteralPath (Join-Path $bin 'one.dsc.resource.json')
            $bundle.resources[1] | ConvertTo-Json -Depth 10 |
                Set-Content -LiteralPath (Join-Path $bin 'two.dsc.resource.json')
            Copy-Item -LiteralPath (Join-Path $bin 'shared.exe') -Destination (Join-Path $bin 'shared')
            @{ type = 'Microsoft.Test/Three'; version = '3.0.0'; author = 'Microsoft Corporation' } |
                ConvertTo-Json | Set-Content -LiteralPath (Join-Path $bin 'windows.dsc.resource.json')
            $bin
        }
        Import-Module (Join-Path $fixtureRoot 'helpers.build.psm1') -Force
        $pushes = [System.Collections.Generic.List[hashtable]]::new()
        $indexes = [System.Collections.Generic.List[hashtable]]::new()
        $failedDirectories = [System.Collections.Generic.List[string]]::new()
        $logins = [System.Collections.Generic.List[hashtable]]::new()
        Mock az -ModuleName DSCOras {
            $global:LASTEXITCODE = 0
            'test-access-token'
        }
        Mock oras -ModuleName DSCOras {
            $global:LASTEXITCODE = 0
            if ($Arguments[0] -eq 'login') {
                $logins.Add(@{ Arguments = $Arguments; Token = $InputToken })
            } elseif ($Arguments[0] -eq 'push') {
                $archive = [System.IO.Compression.ZipFile]::OpenRead(
                    (Join-Path (Get-Location).Path ($Arguments[-1].Split(':')[0]))
                )
                try {
                    $entries = @($archive.Entries.FullName -replace '\\', '/')
                } finally {
                    $archive.Dispose()
                }
                $pushes.Add(@{
                    Reference = $Arguments[1]
                    Resource = Get-Content -LiteralPath ($Arguments[-2].Split(':')[0]) -Raw | ConvertFrom-Json
                    Entries = $entries
                    Annotations = Get-Content -LiteralPath 'annotations.json' -Raw | ConvertFrom-Json -AsHashtable
                    Directory = (Get-Location).Path
                })
            } elseif ($Arguments[1] -eq 'fetch') {
                '{"mediaType":"application/vnd.oci.image.manifest.v1+json","digest":"sha256:abc","size":123}'
            } elseif ($Arguments[1] -eq 'push') {
                $indexes.Add(@{
                    Reference = $Arguments[2]
                    Index = Get-Content -LiteralPath $Arguments[3] -Raw | ConvertFrom-Json
                })
            }
        }
    }

    AfterEach {
        Remove-Module DSCOras -Force
        Remove-Module helpers.build -Force
    }

    It 'Authenticates ORAS once using an ACR access token on stdin' {
        Publish-Artifact -PackageName Microsoft.Test.Bundle -OS Windows -Platforms windows-amd64, windows-arm64 6>$null

        Should -Invoke az -ModuleName DSCOras -Exactly 1 -ParameterFilter {
            ($Arguments -join ' ') -eq 'acr login --name orasdscartifacts --expose-token --query accessToken --output tsv --only-show-errors'
        }
        $logins | Should -HaveCount 1
        $logins[0].Arguments | Should -Be @(
            'login', 'orasdscartifacts.azurecr.io',
            '--username', '00000000-0000-0000-0000-000000000000',
            '--password-stdin'
        )
        $logins[0].Token | Should -Be 'test-access-token'
        $logins[0].Arguments | Should -Not -Contain 'test-access-token'
    }

    It 'Stops before publishing when Azure CLI cannot obtain a token' {
        Mock az -ModuleName DSCOras { $global:LASTEXITCODE = 7 }

        { Publish-Artifact -PackageName Microsoft.Test.Bundle -OS Windows -Platforms windows-arm64 6>$null } |
            Should -Throw '*az acr login failed with exit code 7*'

        Should -Invoke oras -ModuleName DSCOras -Exactly 0
    }

    It 'Rejects an empty ACR access token' {
        Mock az -ModuleName DSCOras {
            $global:LASTEXITCODE = 0
            ' '
        }

        { Publish-Artifact -PackageName Microsoft.Test.Bundle -OS Windows -Platforms windows-arm64 6>$null } |
            Should -Throw '*empty access token*'

        Should -Invoke oras -ModuleName DSCOras -Exactly 0
    }

    It 'Stops before publishing when ORAS authentication fails' {
        Mock oras -ModuleName DSCOras { $global:LASTEXITCODE = 8 } -ParameterFilter { $Arguments[0] -eq 'login' }

        { Publish-Artifact -PackageName Microsoft.Test.Bundle -OS Windows -Platforms windows-arm64 6>$null } |
            Should -Throw '*oras login failed with exit code 8*'

        $pushes | Should -HaveCount 0
        $indexes | Should -HaveCount 0
    }

    It 'Publishes individual resources and multi-platform indexes with a shared implementation ZIP' {
        Publish-Artifact -PackageName Microsoft.Test.Bundle -OS Windows -Platforms windows-amd64, windows-arm64 6>$null

        $pushes | Should -HaveCount 6
        $indexes | Should -HaveCount 3
        foreach ($push in $pushes) {
            $push.Entries | Should -HaveCount 2
            $push.Entries | Should -Contain 'shared.exe'
            $push.Entries | Should -Contain 'nested/support.ps1'
            $push.Resource.type | Should -BeIn @('Microsoft.Test/One', 'Microsoft.Test/Two', 'Microsoft.Test/Three')
            $push.Annotations.'$manifest'.'com.microsoft.dsc.resource.id' | Should -Be $push.Resource.type
            $push.Directory | Should -Not -Exist
        }
        $adapter = $pushes | Where-Object { $_.Resource.type -eq 'Microsoft.Test/Two' } | Select-Object -First 1
        $adapter.Annotations.'$manifest'.'com.microsoft.dsc.kind' | Should -Be 'adapter'
        $adapter.Annotations.'$manifest'.'com.microsoft.dsc.publisher' | Should -Be 'Other Author'
        foreach ($publication in $indexes) {
            $publication.Index.manifests | Should -HaveCount 2
            $publication.Index.manifests.platform.os | Should -Be @('windows', 'windows')
            $publication.Index.manifests.platform.architecture | Should -Be @('amd64', 'arm64')
        }
        foreach ($bin in $bins) {
            Get-Content -LiteralPath (Join-Path $bin 'resource-packages' 'untouched.zip') | Should -Be 'keep'
        }
    }

    It 'Finds All-only packages without requiring an OS-specific definition' {
        $data.ResourcePackageFiles.Windows = @{}
        $data | ConvertTo-Json -Depth 20 |
            Set-Content -LiteralPath (Join-Path $fixtureRoot 'data.build.json')

        Publish-Artifact -PackageName Microsoft.Test.Bundle -OS Windows -Platforms windows-arm64 6>$null

        $pushes | Should -HaveCount 2
        $indexes | Should -HaveCount 2
        foreach ($push in $pushes) {
            $push.Entries | Should -Be @('shared.exe')
        }
    }

    It 'Labels current artifacts with the actual host platform' {
        Publish-Artifact -PackageName Microsoft.Test.Bundle 6>$null

        $hostOS = $IsWindows ? 'windows' : ($IsMacOS ? 'darwin' : 'linux')
        $hostArchitecture = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString()
        $architecture = $hostArchitecture -eq 'Arm64' ? 'arm64' : 'amd64'
        $indexes.Count | Should -Be ($IsWindows ? 3 : 2)
        foreach ($publication in $indexes) {
            $publication.Index.manifests | Should -HaveCount 1
            $publication.Index.manifests[0].platform.os | Should -Be $hostOS
            $publication.Index.manifests[0].platform.architecture | Should -Be $architecture
        }
    }

    It 'Rejects a platform that does not match the selected OS before publishing' {
        { Publish-Artifact -PackageName Microsoft.Test.Bundle -OS Windows -Platforms linux-amd64 6>$null } |
            Should -Throw "*does not match OS 'Windows'*"

        Should -Invoke oras -ModuleName DSCOras -Exactly 0
    }

    It 'Rejects bundle inputs before authenticating or uploading' {
        $data.ResourcePackageFiles.All['Microsoft.Test.Bundle'].manifests = @('bundle.dsc.manifests.json')
        $data | ConvertTo-Json -Depth 20 |
            Set-Content -LiteralPath (Join-Path $fixtureRoot 'data.build.json')

        { Publish-Artifact -PackageName Microsoft.Test.Bundle -OS Windows -Platforms windows-arm64 6>$null } |
            Should -Throw '*individual resource manifest*'

        Should -Invoke az -ModuleName DSCOras -Exactly 0
        Should -Invoke oras -ModuleName DSCOras -Exactly 0
    }

    It 'Rejects an invalid semantic version during preflight' -ForEach @(
        @{ InvalidVersion = 'banana' },
        @{ InvalidVersion = '1.0' }
    ) {
        $bundle.resources[0].version = $InvalidVersion
        $bundle.resources[0] | ConvertTo-Json -Depth 10 |
            Set-Content -LiteralPath (Join-Path $bins[2] 'one.dsc.resource.json')

        { Publish-Artifact -PackageName Microsoft.Test.Bundle -OS Windows -Platforms windows-amd64, windows-arm64 6>$null } |
            Should -Throw '*Invalid semantic version*'

        Should -Invoke az -ModuleName DSCOras -Exactly 0
        Should -Invoke oras -ModuleName DSCOras -Exactly 0
    }

    It 'Publishes native Unix binaries from .exe package definitions' -ForEach @(
        @{ UnixOS = 'Linux'; UnixPlatform = 'linux-amd64'; UnixTarget = 'x86_64-unknown-linux-gnu' },
        @{ UnixOS = 'macOS'; UnixPlatform = 'darwin-arm64'; UnixTarget = 'aarch64-apple-darwin' }
    ) {
        $bin = Join-Path $fixtureRoot 'bin' $UnixTarget 'debug'
        $null = New-Item -ItemType Directory -Path $bin -Force
        Copy-Item -LiteralPath (Join-Path $bins[0] 'shared') -Destination $bin
        Copy-Item -LiteralPath (Join-Path $bins[0] 'one.dsc.resource.json'),(Join-Path $bins[0] 'two.dsc.resource.json') -Destination $bin

        Publish-Artifact -PackageName Microsoft.Test.Bundle -OS $UnixOS -Platforms $UnixPlatform 6>$null

        $pushes | Should -HaveCount 2
        foreach ($push in $pushes) {
            $push.Entries | Should -Be @('shared')
        }
    }

    It 'Converts an adapted YAML manifest into a separate JSON layer' {
        $data.ResourcePackageFiles.Windows['Microsoft.Test.Bundle'].manifests += 'adapted.dsc.adaptedResource.yaml'
        $data | ConvertTo-Json -Depth 20 |
            Set-Content -LiteralPath (Join-Path $fixtureRoot 'data.build.json')
        Set-Content -LiteralPath (Join-Path $bins[2] 'adapted.dsc.adaptedResource.yaml') -Value 'type: Microsoft.Test/Adapted'
        Mock ConvertFrom-Yaml -ModuleName DSCOras {
            @{ type = 'Microsoft.Test/Adapted'; version = '1.0.0'; requireAdapter = 'Microsoft.Test/Two' }
        }

        Publish-Artifact -PackageName Microsoft.Test.Bundle -OS Windows -Platforms windows-arm64 6>$null

        $pushes | Should -HaveCount 4
        $adapted = $pushes | Where-Object { $_.Resource.type -eq 'Microsoft.Test/Adapted' }
        $adapted.Resource.requireAdapter | Should -Be 'Microsoft.Test/Two'
        $adapted.Annotations.Keys | Should -Contain 'adapted.dsc.adaptedResource.json'
        $adapted.Annotations.Keys | Should -Not -Contain 'adapted.dsc.resource.json'
        Should -Invoke ConvertFrom-Yaml -ModuleName DSCOras -Exactly 1
    }

    It 'Fails before publishing when another platform has missing artifacts' {
        Remove-Item -LiteralPath (Join-Path $bins[2] 'shared.exe')

        { Publish-Artifact -PackageName Microsoft.Test.Bundle -OS Windows -Platforms windows-amd64, windows-arm64 6>$null } |
            Should -Throw '*shared.exe*'

        Should -Invoke oras -ModuleName DSCOras -Exactly 0
    }

    It 'Stops on ORAS failure, restores the working directory, and removes staging files' {
        $originalDirectory = (Get-Location).Path
        Mock oras -ModuleName DSCOras {
            $failedDirectories.Add((Get-Location).Path)
            $global:LASTEXITCODE = 9
        } -ParameterFilter { $Arguments[0] -eq 'push' }

        { Publish-Artifact -PackageName Microsoft.Test.Bundle -OS Windows -Platforms windows-arm64 6>$null } |
            Should -Throw '*exit code 9*'

        (Get-Location).Path | Should -Be $originalDirectory
        $failedDirectories[0] | Should -Not -Exist
        $indexes | Should -HaveCount 0
    }
}

Describe 'Find-Artifact' {
    BeforeEach {
        Import-Module (Join-Path $PSScriptRoot '..' 'packaging' 'oci' 'DSCOras.psm1') -Force
        Mock Connect-DscRegistry -ModuleName DSCOras
        $registry = 'orasdscartifacts.azurecr.io'
        $repository = "$registry/dsc/resource/microsoft.windows.registry"
        $otherRepository = "$registry/dsc/resource/unrelated"
        $documents = @{}
        foreach ($version in @('1.0.0', '2.0.0-beta.1')) {
            $platforms = if ($version -eq '1.0.0') { @('windows-amd64', 'windows-arm64') } else { @('linux-arm64') }
            $descriptors = foreach ($platform in $platforms) {
                $os, $architecture = $platform.Split('-')
                @{
                    mediaType = 'application/vnd.oci.image.manifest.v1+json'
                    digest = "sha256:$version-$platform"
                    size = 123
                    platform = @{ os = $os; architecture = $architecture }
                }
            }
            $documents["$repository`:registrylist-$version"] = @{
                schemaVersion = 2
                mediaType = 'application/vnd.oci.image.index.v1+json'
                artifactType = 'application/vnd.microsoft.dsc.resource.v1+zip'
                annotations = @{
                    'com.microsoft.dsc.resource.id' = 'Microsoft.Windows/RegistryList'
                    'com.microsoft.dsc.version' = $version
                    'com.microsoft.dsc.publisher' = 'Microsoft Corporation'
                }
                manifests = @($descriptors)
            }
        }
        $documents["$otherRepository`:registrylist-1.0.0"] = @{
            mediaType = 'application/vnd.oci.image.index.v1+json'
            artifactType = 'application/vnd.microsoft.dsc.resource.v1+zip'
            annotations = @{
                'com.microsoft.dsc.resource.id' = 'Microsoft.Other/RegistryList'
                'com.microsoft.dsc.version' = '1.0.0'
            }
            manifests = @()
        }
        Mock Invoke-OrasCommand -ModuleName DSCOras {
            if ($Arguments[0] -eq 'repo' -and $Arguments[1] -eq 'ls') {
                @{
                    repositories = @(
                        'dsc/resource/microsoft.windows.registry',
                        'dsc/resource/unrelated',
                        'dsc/resource/microsoft.windows.registry',
                        'not-a-resource'
                    )
                } | ConvertTo-Json
            } elseif ($Arguments[0] -eq 'repo' -and $Arguments[1] -eq 'tags') {
                if ($Arguments[2] -eq $repository) {
                    @{
                        tags = @(
                            'registrylist-1.0.0',
                            'registrylist-1.0.0-windows-arm64',
                            'registrylist-2.0.0-beta.1',
                            'registrylist-1.0.0',
                            'registry-1.0.0'
                        )
                    } | ConvertTo-Json
                } else {
                    @{ tags = @('registrylist-1.0.0') } | ConvertTo-Json
                }
            } elseif ($Arguments[0] -eq 'manifest' -and $Arguments[1] -eq 'fetch') {
                if ($documents.ContainsKey($Arguments[2])) {
                    $documents[$Arguments[2]] | ConvertTo-Json -Depth 20
                } else {
                    @{ mediaType = 'application/vnd.oci.image.manifest.v1+json' } | ConvertTo-Json
                }
            } else {
                throw "Unexpected ORAS invocation: $Arguments"
            }
        }
    }

    AfterEach {
        Remove-Module DSCOras -Force
    }

    It 'Exports only the public publishing and discovery commands' {
        @(Get-Command -Module DSCOras).Name | Should -Be @('Find-Artifact', 'Get-Artifact', 'Publish-Artifact')
        Should -Invoke Connect-DscRegistry -ModuleName DSCOras -Exactly 0
    }

    It 'Finds all versions and platforms in a shared package, excluding other resource types and child tags' {
        $results = @(Find-Artifact -ResourceType Microsoft.Windows/RegistryList)

        $results | Should -HaveCount 3
        $results.Version | Should -Be @('1.0.0', '1.0.0', '2.0.0-beta.1')
        $results.Platform | Should -Be @('windows-amd64', 'windows-arm64', 'linux-arm64')
        foreach ($result in $results) {
            $result.ResourceType | Should -Be 'Microsoft.Windows/RegistryList'
            $result.Reference | Should -Be "$repository@$($result.Digest)"
            $result.MediaType | Should -Be 'application/vnd.oci.image.manifest.v1+json'
            $result.Size | Should -Be 123
            $result.PSObject.TypeNames[0] | Should -Be 'DSCOras.Artifact'
        }
        Should -Invoke Connect-DscRegistry -ModuleName DSCOras -Exactly 0
        Should -Invoke Invoke-OrasCommand -ModuleName DSCOras -Exactly 0 -ParameterFilter {
            $Arguments[0] -in @('pull', 'push', 'blob')
        }
    }

    It 'Filters an exact version and platform without selecting similarly named resource types' {
        $results = @(Find-Artifact -ResourceType Microsoft.Windows/RegistryList -Version 1.0.0 -Platform windows-arm64)

        $results | Should -HaveCount 1
        $results[0].Version | Should -Be '1.0.0'
        $results[0].Platform | Should -Be 'windows-arm64'
        $results[0].IndexReference | Should -Be "$repository`:registrylist-1.0.0"
    }

    It 'Filters a platform across all versions' {
        $results = @(Find-Artifact -ResourceType Microsoft.Windows/RegistryList -Platform linux-arm64)

        $results | Should -HaveCount 1
        $results[0].Version | Should -Be '2.0.0-beta.1'
    }

    It 'Returns all platforms when only an exact version is specified' {
        $results = @(Find-Artifact -ResourceType Microsoft.Windows/RegistryList -Version 1.0.0)

        $results | Should -HaveCount 2
        $results.Platform | Should -Be @('windows-amd64', 'windows-arm64')
    }

    It 'Returns no objects when the requested version is unavailable' {
        @(Find-Artifact -ResourceType Microsoft.Windows/RegistryList -Version 9.0.0) | Should -HaveCount 0
    }

    It 'Rejects invalid resource types before accessing the registry' {
        { Find-Artifact -ResourceType RegistryList } | Should -Throw
        Should -Invoke Connect-DscRegistry -ModuleName DSCOras -Exactly 0
    }

    It 'Surfaces registry failures instead of reporting no matches' {
        Mock Invoke-OrasCommand -ModuleName DSCOras { throw 'Registry unavailable' }

        { Find-Artifact -ResourceType Microsoft.Windows/RegistryList } | Should -Throw '*Registry unavailable*'
    }

    It 'Rejects invalid descriptors in a matching resource index' {
        $documents["$repository`:registrylist-1.0.0"].manifests[0].digest = $null

        { Find-Artifact -ResourceType Microsoft.Windows/RegistryList -Version 1.0.0 } |
            Should -Throw '*Invalid platform descriptor*'
    }
}

Describe 'Get-Artifact' {
    BeforeEach {
        Import-Module (Join-Path $PSScriptRoot '..' 'packaging' 'oci' 'DSCOras.psm1') -Force
        $fixtureRoot = Join-Path $TestDrive ([guid]::NewGuid().ToString())
        $sourceDirectory = Join-Path $fixtureRoot 'source'
        $null = New-Item -ItemType Directory -Path (Join-Path $sourceDirectory 'nested') -Force
        Set-Content -LiteralPath (Join-Path $sourceDirectory 'registry.exe') -Value 'implementation'
        Set-Content -LiteralPath (Join-Path $sourceDirectory 'nested' 'support.ps1') -Value 'support'
        $zipPath = Join-Path $fixtureRoot 'Microsoft.Windows.Registry.zip'
        [System.IO.Compression.ZipFile]::CreateFromDirectory($sourceDirectory, $zipPath)
        $zipDigest = 'sha256:' + (Get-FileHash -LiteralPath $zipPath).Hash.ToLowerInvariant()
        $cacheRoot = Join-Path $fixtureRoot 'resourceCache'
        $repository = 'orasdscartifacts.azurecr.io/dsc/resource/microsoft.windows.registry'
        $documents = @{}
        $blobs = @{ $zipDigest = $zipPath }
        $artifacts = @(
            foreach ($selection in @(
                @('Microsoft.Windows/RegistryList', '2.9.0', 'windows-amd64'),
                @('Microsoft.Windows/RegistryList', '2.10.0', 'windows-amd64'),
                @('Microsoft.Windows/RegistryList', '3.0.0-beta.1', 'windows-amd64'),
                @('Microsoft.Windows/RegistryList', '2.10.0', 'windows-arm64'),
                @('Microsoft.Windows/RegistryList', '2.10.0', 'linux-arm64'),
                @('Microsoft.Windows/Registry', '2.10.0', 'windows-amd64'),
                @('Microsoft.Test/Adapted', '2.10.0', 'windows-amd64')
            )) {
                $type, $resourceVersion, $resourcePlatform = $selection
                $slug = $type.Split('/')[1].ToLowerInvariant()
                $reference = "$repository@sha256:$slug-$resourceVersion-$resourcePlatform"
                $resourcePath = Join-Path $fixtureRoot "$slug-$resourceVersion-$resourcePlatform.json"
                $resourceDocument = @{ type = $type; version = $resourceVersion }
                $manifestName = "$slug.dsc.resource.json"
                if ($type -eq 'Microsoft.Test/Adapted') {
                    $manifestName = "$slug.dsc.adaptedResource.json"
                    $resourceDocument.'$schema' = 'https://aka.ms/dsc/schemas/v3/resource/adapted/manifest.json'
                    $resourceDocument.kind = 'resource'
                    $resourceDocument.capabilities = @('get')
                    $resourceDocument.requireAdapter = 'Microsoft.Test/Adapter'
                    $resourceDocument.content = @{ value = 'adapted' }
                    $resourceDocument.schema = @{ embedded = @{ type = 'object' } }
                }
                $resourceDocument | ConvertTo-Json -Depth 10 |
                    Set-Content -LiteralPath $resourcePath -Encoding utf8NoBOM
                $resourceDigest = 'sha256:' + (Get-FileHash -LiteralPath $resourcePath).Hash.ToLowerInvariant()
                $blobs[$resourceDigest] = $resourcePath
                $documents[$reference] = @{
                    mediaType = 'application/vnd.oci.image.manifest.v1+json'
                    artifactType = 'application/vnd.microsoft.dsc.resource.v1+zip'
                    annotations = @{
                        'com.microsoft.dsc.resource.id' = $type
                        'com.microsoft.dsc.version' = $resourceVersion
                    }
                    layers = @(
                        @{
                            mediaType = 'application/zip'
                            digest = $zipDigest
                            size = (Get-Item -LiteralPath $zipPath).Length
                            annotations = @{ 'org.opencontainers.image.title' = 'Microsoft.Windows.Registry.zip' }
                        },
                        @{
                            mediaType = 'application/json'
                            digest = $resourceDigest
                            size = (Get-Item -LiteralPath $resourcePath).Length
                            annotations = @{ 'org.opencontainers.image.title' = $manifestName }
                        }
                    )
                }
                [pscustomobject]@{
                    ResourceType = $type
                    Version = $resourceVersion
                    Platform = $resourcePlatform
                    Reference = $reference
                }
            }
        )
        Mock Get-DscHostPlatform -ModuleName DSCOras { 'windows-amd64' }
        Mock Get-DscResourceCachePath -ModuleName DSCOras { $cacheRoot }
        Mock Find-Artifact -ModuleName DSCOras {
            $artifacts | Where-Object {
                $_.ResourceType -eq $ResourceType -and $_.Platform -eq $Platform -and
                    (-not $Version -or $_.Version -ceq $Version)
            }
        }
        Mock Invoke-OrasCommand -ModuleName DSCOras {
            if ($Arguments[0] -eq 'manifest' -and $Arguments[1] -eq 'fetch') {
                $documents[$Arguments[2]] | ConvertTo-Json -Depth 20
            } elseif ($Arguments[0] -eq 'blob' -and $Arguments[1] -eq 'fetch' -and $Arguments[3] -eq '--output') {
                $digest = $Arguments[2].Split('@')[1]
                Copy-Item -LiteralPath $blobs[$digest] -Destination $Arguments[4]
            } else {
                throw "Unexpected ORAS invocation: $Arguments"
            }
        }
    }

    AfterEach {
        Remove-Module DSCOras -Force
    }

    It 'Downloads the latest stable version for the host and places the manifest beside the expanded ZIP contents' {
        $result = Get-Artifact -ResourceType Microsoft.Windows/RegistryList

        $result.Version | Should -Be '2.10.0'
        $result.Platform | Should -Be 'windows-amd64'
        $result.Path | Should -Be (Join-Path $cacheRoot 'Microsoft.Windows.Registry' '2.10.0')
        $result.ManifestPath | Should -Be (Join-Path $result.Path 'registrylist.dsc.resource.json')
        $result.PSObject.TypeNames[0] | Should -Be 'DSCOras.CachedArtifact'
        Join-Path $result.Path 'registry.exe' | Should -Exist
        Join-Path $result.Path 'nested' 'support.ps1' | Should -Exist
        $resource = Get-Content -LiteralPath $result.ManifestPath -Raw | ConvertFrom-Json
        $resource.type | Should -Be 'Microsoft.Windows/RegistryList'
        $resource.version | Should -Be '2.10.0'
        @(Get-ChildItem -LiteralPath $cacheRoot -Force) | Should -HaveCount 1
        Should -Invoke Invoke-OrasCommand -ModuleName DSCOras -Exactly 1 -ParameterFilter {
            $Arguments[0] -eq 'manifest' -and $Arguments[2] -eq "$repository@sha256:registrylist-2.10.0-windows-amd64"
        }
    }

    It 'Honors an explicit version and platform, including an explicitly selected prerelease' -ForEach @(
        @{ RequestedVersion = '2.10.0'; RequestedPlatform = 'linux-arm64' },
        @{ RequestedVersion = '3.0.0-beta.1'; RequestedPlatform = 'windows-amd64' }
    ) {
        $result = Get-Artifact -ResourceType Microsoft.Windows/RegistryList -Version $RequestedVersion -Platform $RequestedPlatform

        $result.Version | Should -Be $RequestedVersion
        $result.Platform | Should -Be $RequestedPlatform
        $result.ManifestPath | Should -Exist
    }

    It 'Installs adapted JSON manifests with the filename required by the DSC adapted-resource loader' {
        $result = Get-Artifact -ResourceType Microsoft.Test/Adapted

        $result.ManifestPath | Should -Be (Join-Path $result.Path 'adapted.dsc.adaptedResource.json')
        Join-Path $result.Path 'adapted.dsc.resource.json' | Should -Not -Exist
        $dsc = Join-Path $PSScriptRoot '..' 'bin' 'debug' ($IsWindows ? 'dsc.exe' : 'dsc')
        $dsc | Should -Exist -Because 'the DSC build is needed to exercise the actual manifest loader'
        $oldRestrictedPath = $env:DSC_RESTRICTED_PATH
        try {
            $env:DSC_RESTRICTED_PATH = $result.Path
            $resources = @(& $dsc resource list 'Microsoft.Test/Adapted' 2>(Join-Path $fixtureRoot 'discovery.log') |
                ConvertFrom-Json -Depth 20)
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -LiteralPath (Join-Path $fixtureRoot 'discovery.log') -Raw)
            $resources | Should -HaveCount 1
            $resources[0].requireAdapter | Should -Be 'Microsoft.Test/Adapter'
        } finally {
            $env:DSC_RESTRICTED_PATH = $oldRestrictedPath
        }
    }

    It 'Reuses a matching cache without downloading the ZIP or manifest again' {
        $first = Get-Artifact -ResourceType Microsoft.Windows/RegistryList
        $second = Get-Artifact -ResourceType Microsoft.Windows/RegistryList

        $second.Path | Should -Be $first.Path
        Should -Invoke Invoke-OrasCommand -ModuleName DSCOras -Exactly 2 -ParameterFilter {
            $Arguments[0] -eq 'blob'
        }
    }

    It 'Preserves manifests for resources sharing the same package and downloads the shared ZIP only once' {
        $registryList = Get-Artifact -ResourceType Microsoft.Windows/RegistryList
        $registry = Get-Artifact -ResourceType Microsoft.Windows/Registry

        $registry.Path | Should -Be $registryList.Path
        $registry.ManifestPath | Should -Exist
        $registryList.ManifestPath | Should -Exist
        Should -Invoke Invoke-OrasCommand -ModuleName DSCOras -Exactly 1 -ParameterFilter {
            $Arguments[0] -eq 'blob' -and $Arguments[2] -eq "$repository@$zipDigest"
        }
    }

    It 'Preserves the live cache if staging a reused-cache metadata update fails' {
        $existing = Get-Artifact -ResourceType Microsoft.Windows/RegistryList
        $metadataPath = Join-Path $existing.Path '.dsc-cache.json'
        $metadataBefore = Get-Content -LiteralPath $metadataPath -Raw
        $manifestBefore = Get-Content -LiteralPath $existing.ManifestPath -Raw
        Mock Set-Content -ModuleName DSCOras { throw 'Metadata write failed' } -ParameterFilter {
            $LiteralPath -like '*expanded*.dsc-cache.json'
        }

        { Get-Artifact -ResourceType Microsoft.Windows/Registry } | Should -Throw '*Metadata write failed*'

        Get-Content -LiteralPath $metadataPath -Raw | Should -BeExactly $metadataBefore
        Get-Content -LiteralPath $existing.ManifestPath -Raw | Should -BeExactly $manifestBefore
        Join-Path $existing.Path 'registry.dsc.resource.json' | Should -Not -Exist
        @(Get-ChildItem -LiteralPath $cacheRoot -Force) | Should -HaveCount 1
    }

    It 'Serializes cross-process acquisitions and preserves both shared-resource manifests' {
        $signals = Join-Path $fixtureRoot 'signals'
        $null = New-Item -ItemType Directory -Path $signals
        $modulePath = Join-Path $PSScriptRoot '..' 'packaging' 'oci' 'DSCOras.psm1'
        $worker = {
            param($ModulePath, $CacheRoot, $Artifact, $Documents, $Blobs, $Signals, $HoldDownload)
            Import-Module $ModulePath -Force
            & (Get-Module DSCOras) {
                param($Root, $Selection, $Documents, $Blobs, $Signals, $HoldDownload)
                $script:fixture = @{
                    Root = $Root; Artifact = $Selection; Documents = $Documents
                    Blobs = $Blobs; Signals = $Signals; HoldDownload = $HoldDownload
                }
                $script:originalEnter = (Get-Command Enter-DscResourceCacheLock).ScriptBlock
                function script:Find-Artifact { $script:fixture.Artifact }
                function script:Get-DscResourceCachePath { $script:fixture.Root }
                function script:Enter-DscResourceCacheLock {
                    param($Path)
                    $slug = $script:fixture.Artifact.ResourceType.Split('/')[1]
                    [System.IO.File]::WriteAllText((Join-Path $script:fixture.Signals "$slug-enter"), 'ready')
                    & $script:originalEnter -Path $Path
                }
                function script:Invoke-OrasCommand {
                    param($Arguments)
                    $state = $script:fixture
                    if ($Arguments[0] -eq 'manifest') {
                        return $state.Documents[$Arguments[2]] | ConvertTo-Json -Depth 20
                    }
                    if ($state.HoldDownload) {
                        [System.IO.File]::WriteAllText((Join-Path $state.Signals 'held'), 'ready')
                        $deadline = [DateTime]::UtcNow.AddSeconds(15)
                        while (-not [System.IO.File]::Exists((Join-Path $state.Signals 'release'))) {
                            if ([DateTime]::UtcNow -gt $deadline) { throw 'Timed out synchronizing cache writers.' }
                            Start-Sleep -Milliseconds 50
                        }
                        $state.HoldDownload = $false
                    }
                    $digest = $Arguments[2].Split('@')[1]
                    $slug = $state.Artifact.ResourceType.Split('/')[1]
                    [System.IO.File]::AppendAllText((Join-Path $state.Signals "$slug-downloads"), "$digest`n")
                    Copy-Item -LiteralPath $state.Blobs[$digest] -Destination $Arguments[4]
                }
                Get-Artifact -ResourceType $Selection.ResourceType -Version $Selection.Version -Platform $Selection.Platform
            } $CacheRoot $Artifact $Documents $Blobs $Signals $HoldDownload
        }
        $waitForSignal = {
            param($Path, $Job)
            $deadline = [DateTime]::UtcNow.AddSeconds(15)
            while (-not (Test-Path -LiteralPath $Path)) {
                if ([DateTime]::UtcNow -gt $deadline -or $Job.State -in @('Failed', 'Completed')) {
                    Receive-Job -Job $Job -ErrorAction Stop
                    throw "Worker did not signal '$Path'."
                }
                Start-Sleep -Milliseconds 50
            }
        }
        $jobs = @()
        try {
            foreach ($type in @('Microsoft.Windows/RegistryList', 'Microsoft.Windows/Registry')) {
                $selection = $artifacts | Where-Object {
                    $_.ResourceType -eq $type -and $_.Version -eq '2.10.0' -and $_.Platform -eq 'windows-amd64'
                } | Select-Object -First 1
                $jobs += Start-Job -ScriptBlock $worker -ArgumentList @(
                    $modulePath, $cacheRoot, $selection, $documents, $blobs, $signals, ($jobs.Count -eq 0)
                )
                if ($jobs.Count -eq 1) {
                    & $waitForSignal (Join-Path $signals 'held') $jobs[0]
                }
            }
            & $waitForSignal (Join-Path $signals 'Registry-enter') $jobs[1]
            Join-Path $signals 'Registry-downloads' | Should -Not -Exist
            Set-Content -LiteralPath (Join-Path $signals 'release') -Value 'continue'
            $null = $jobs | Wait-Job -Timeout 20
            foreach ($job in $jobs) {
                $job.State | Should -Be 'Completed'
                Receive-Job -Job $job -ErrorAction Stop | Out-Null
            }
            $destination = Join-Path $cacheRoot 'Microsoft.Windows.Registry' '2.10.0'
            Join-Path $destination 'registry.dsc.resource.json' | Should -Exist
            Join-Path $destination 'registrylist.dsc.resource.json' | Should -Exist
            $metadata = Get-Content -LiteralPath (Join-Path $destination '.dsc-cache.json') -Raw | ConvertFrom-Json -AsHashtable
            $metadata.ManifestDigests.Keys | Should -HaveCount 2
            @(Get-Content -LiteralPath (Join-Path $signals 'Registry-downloads'),(Join-Path $signals 'RegistryList-downloads') |
                Where-Object { $_ -eq $zipDigest }) | Should -HaveCount 1
        } finally {
            foreach ($job in $jobs) {
                if ($job.State -eq 'Running') { Stop-Job -Job $job }
                Remove-Job -Job $job -Force
            }
        }
    }

    It 'Repairs missing implementation files while retaining only hash-validated sibling manifests (corrupt: <CorruptSibling>)' -ForEach @(
        @{ CorruptSibling = $false }, @{ CorruptSibling = $true }
    ) {
        $cached = Get-Artifact -ResourceType Microsoft.Windows/RegistryList
        $sibling = Get-Artifact -ResourceType Microsoft.Windows/Registry
        $siblingDigest = 'sha256:' + (Get-FileHash -LiteralPath $sibling.ManifestPath).Hash.ToLowerInvariant()
        if ($CorruptSibling) {
            Set-Content -LiteralPath $sibling.ManifestPath -Value 'corrupt'
        }
        $supportPath = Join-Path $cached.Path 'nested' 'support.ps1'
        Remove-Item -LiteralPath $supportPath

        Get-Artifact -ResourceType Microsoft.Windows/RegistryList | Out-Null

        $supportPath | Should -Exist
        $cached.ManifestPath | Should -Exist
        $metadata = Get-Content -LiteralPath (Join-Path $cached.Path '.dsc-cache.json') -Raw | ConvertFrom-Json -AsHashtable
        if ($CorruptSibling) {
            $sibling.ManifestPath | Should -Not -Exist
            $metadata.ManifestDigests.Keys | Should -Not -Contain 'registry.dsc.resource.json'
        } else {
            $sibling.ManifestPath | Should -Exist
            $metadata.ManifestDigests['registry.dsc.resource.json'] | Should -Be $siblingDigest
            $reused = Get-Artifact -ResourceType Microsoft.Windows/Registry
            $reused.ManifestPath | Should -Be $sibling.ManifestPath
            Should -Invoke Invoke-OrasCommand -ModuleName DSCOras -Exactly 1 -ParameterFilter {
                $Arguments[0] -eq 'blob' -and $Arguments[2] -like '*@sha256:*' -and
                    $Arguments[2] -ne "$repository@$zipDigest" -and
                    $Arguments[4] -like '*registry.dsc.resource.json'
            }
        }
        Should -Invoke Invoke-OrasCommand -ModuleName DSCOras -Exactly 2 -ParameterFilter {
            $Arguments[0] -eq 'blob' -and $Arguments[2] -eq "$repository@$zipDigest"
        }
    }

    It 'Downloads a corrupted cached manifest again without downloading the ZIP' {
        $cached = Get-Artifact -ResourceType Microsoft.Windows/RegistryList
        Set-Content -LiteralPath $cached.ManifestPath -Value 'corrupt'

        $result = Get-Artifact -ResourceType Microsoft.Windows/RegistryList

        (Get-Content -LiteralPath $result.ManifestPath -Raw | ConvertFrom-Json).type |
            Should -Be 'Microsoft.Windows/RegistryList'
        Should -Invoke Invoke-OrasCommand -ModuleName DSCOras -Exactly 1 -ParameterFilter {
            $Arguments[0] -eq 'blob' -and $Arguments[2] -eq "$repository@$zipDigest"
        }
    }

    It 'Caches multiple versions independently and reuses either version without downloading again' {
        $latest = Get-Artifact -ResourceType Microsoft.Windows/RegistryList
        $older = Get-Artifact -ResourceType Microsoft.Windows/RegistryList -Version 2.9.0

        $older.Path | Should -Be (Join-Path $cacheRoot 'Microsoft.Windows.Registry' '2.9.0')
        $latest.ManifestPath | Should -Exist
        $older.ManifestPath | Should -Exist
        (Get-Content -LiteralPath $latest.ManifestPath -Raw | ConvertFrom-Json).version | Should -Be '2.10.0'
        (Get-Content -LiteralPath $older.ManifestPath -Raw | ConvertFrom-Json).version | Should -Be '2.9.0'
        @(Get-ChildItem -LiteralPath (Split-Path -Parent $older.Path) -Directory) | Should -HaveCount 2
        (Get-Artifact -ResourceType Microsoft.Windows/RegistryList).Path | Should -Be $latest.Path
        (Get-Artifact -ResourceType Microsoft.Windows/RegistryList -Version 2.9.0).Path | Should -Be $older.Path
        Should -Invoke Invoke-OrasCommand -ModuleName DSCOras -Exactly 4 -ParameterFilter {
            $Arguments[0] -eq 'blob'
        }
    }

    It 'Replaces only the selected version when its platform changes' {
        $older = Get-Artifact -ResourceType Microsoft.Windows/RegistryList -Version 2.9.0
        $old = Get-Artifact -ResourceType Microsoft.Windows/RegistryList
        $sibling = Get-Artifact -ResourceType Microsoft.Windows/Registry
        $old.Version | Should -Be '2.10.0'
        $stalePath = Join-Path $old.Path 'stale.exe'
        Set-Content -LiteralPath $stalePath -Value 'stale'

        $new = Get-Artifact -ResourceType Microsoft.Windows/RegistryList -Version 2.10.0 -Platform windows-arm64

        $stalePath | Should -Not -Exist
        $sibling.ManifestPath | Should -Not -Exist
        $new.ManifestPath | Should -Exist
        (Get-Content -LiteralPath $new.ManifestPath -Raw | ConvertFrom-Json).version | Should -Be '2.10.0'
        $older.ManifestPath | Should -Exist
        (Get-Content -LiteralPath $older.ManifestPath -Raw | ConvertFrom-Json).version | Should -Be '2.9.0'
        @(Get-ChildItem -LiteralPath $cacheRoot -Force) | Should -HaveCount 1
    }

    It 'Does not retain sibling manifests when the implementation ZIP changes' {
        $cached = Get-Artifact -ResourceType Microsoft.Windows/RegistryList
        $sibling = Get-Artifact -ResourceType Microsoft.Windows/Registry
        Set-Content -LiteralPath (Join-Path $sourceDirectory 'registry.exe') -Value 'updated implementation'
        $newZipPath = Join-Path $fixtureRoot 'updated.zip'
        [System.IO.Compression.ZipFile]::CreateFromDirectory($sourceDirectory, $newZipPath)
        $newZipDigest = 'sha256:' + (Get-FileHash -LiteralPath $newZipPath).Hash.ToLowerInvariant()
        $blobs[$newZipDigest] = $newZipPath
        $reference = "$repository@sha256:registrylist-2.10.0-windows-amd64"
        $documents[$reference].layers[0].digest = $newZipDigest
        $documents[$reference].layers[0].size = (Get-Item -LiteralPath $newZipPath).Length

        Get-Artifact -ResourceType Microsoft.Windows/RegistryList | Out-Null

        $cached.ManifestPath | Should -Exist
        $sibling.ManifestPath | Should -Not -Exist
        Get-Content -LiteralPath (Join-Path $cached.Path 'registry.exe') -Raw | Should -Match 'updated implementation'
        $metadata = Get-Content -LiteralPath (Join-Path $cached.Path '.dsc-cache.json') -Raw | ConvertFrom-Json -AsHashtable
        $metadata.ManifestDigests.Keys | Should -Not -Contain 'registry.dsc.resource.json'
        $metadata.ZipDigest | Should -Be $newZipDigest
    }

    It 'Leaves the existing cache intact and cleans up staging when a download fails' {
        $old = Get-Artifact -ResourceType Microsoft.Windows/RegistryList
        Mock Invoke-OrasCommand -ModuleName DSCOras { throw 'Download failed' } -ParameterFilter {
            $Arguments[0] -eq 'blob'
        }

        { Get-Artifact -ResourceType Microsoft.Windows/RegistryList -Version 2.9.0 } | Should -Throw '*Download failed*'

        (Get-Content -LiteralPath $old.ManifestPath -Raw | ConvertFrom-Json).version | Should -Be '2.10.0'
        @(Get-ChildItem -LiteralPath $cacheRoot -Force) | Should -HaveCount 1
    }

    It 'Fails explicitly when no stable version or requested platform is available' {
        { Get-Artifact -ResourceType Microsoft.Windows/RegistryList -Platform darwin-amd64 } |
            Should -Throw '*No matching resource artifact*'
        $artifacts = @($artifacts | Where-Object Version -eq '3.0.0-beta.1')
        { Get-Artifact -ResourceType Microsoft.Windows/RegistryList } |
            Should -Throw '*No matching resource artifact*'
        $cacheRoot | Should -Not -Exist
    }

    It 'Rejects ambiguous matches rather than downloading an arbitrary package' {
        $artifacts += $artifacts | Where-Object { $_.Version -eq '2.10.0' -and $_.Platform -eq 'windows-amd64' }

        { Get-Artifact -ResourceType Microsoft.Windows/RegistryList } | Should -Throw '*Multiple matching resource artifacts*'
        $cacheRoot | Should -Not -Exist
    }

    It 'Rejects filenames that could escape the cache' {
        $documents["$repository@sha256:registrylist-2.10.0-windows-amd64"].layers[0].annotations[
            'org.opencontainers.image.title'
        ] = '../escape.zip'

        { Get-Artifact -ResourceType Microsoft.Windows/RegistryList } | Should -Throw '*Invalid ZIP or resource manifest filename*'
        $cacheRoot | Should -Not -Exist
    }

    It 'Rejects a resource version that is unsafe as a cache directory name' {
        $entry = $artifacts | Where-Object { $_.Version -eq '2.10.0' -and $_.Platform -eq 'windows-amd64' } |
            Select-Object -First 1
        $entry.Version = '../../outside'

        { Get-Artifact -ResourceType Microsoft.Windows/RegistryList -Version '../../outside' } | Should -Throw
        $cacheRoot | Should -Not -Exist
    }

    It 'Rejects corrupted downloads before creating a package directory' {
        Mock Invoke-OrasCommand -ModuleName DSCOras {
            Set-Content -LiteralPath $Arguments[4] -Value 'corrupt'
        } -ParameterFilter { $Arguments[0] -eq 'blob' }

        { Get-Artifact -ResourceType Microsoft.Windows/RegistryList } | Should -Throw '*verification*'
        @(Get-ChildItem -LiteralPath $cacheRoot -Force) | Should -HaveCount 0
    }

    It 'Rejects ZIP entries that would escape the extraction directory' {
        $archive = [System.IO.Compression.ZipFile]::Open($zipPath, [System.IO.Compression.ZipArchiveMode]::Update)
        try {
            $null = $archive.CreateEntry('../escaped.exe')
        } finally {
            $archive.Dispose()
        }
        $newDigest = 'sha256:' + (Get-FileHash -LiteralPath $zipPath).Hash.ToLowerInvariant()
        $blobs[$newDigest] = $zipPath
        $layer = $documents["$repository@sha256:registrylist-2.10.0-windows-amd64"].layers[0]
        $layer.digest = $newDigest
        $layer.size = (Get-Item -LiteralPath $zipPath).Length

        { Get-Artifact -ResourceType Microsoft.Windows/RegistryList } | Should -Throw '*outside*'
        @(Get-ChildItem -LiteralPath $cacheRoot -Force) | Should -HaveCount 0
    }

    It 'Does not create or download cache files with WhatIf' {
        Get-Artifact -ResourceType Microsoft.Windows/RegistryList -WhatIf

        $cacheRoot | Should -Not -Exist
        Should -Invoke Invoke-OrasCommand -ModuleName DSCOras -Exactly 0 -ParameterFilter {
            $Arguments[0] -eq 'blob'
        }
    }
}

Describe 'DSC resource cache location' {
    BeforeEach {
        Import-Module (Join-Path $PSScriptRoot '..' 'packaging' 'oci' 'DSCOras.psm1') -Force
        $savedLocalAppData = $env:LOCALAPPDATA
        $env:LOCALAPPDATA = Join-Path $TestDrive 'AppData' 'Local'
    }

    AfterEach {
        $env:LOCALAPPDATA = $savedLocalAppData
        Remove-Module DSCOras -Force
    }

    It 'Uses AppData Local dsc resourceCache on Windows' {
        $actual = & (Get-Module DSCOras) {
            Set-Variable -Name IsWindows -Value $true -Scope Script
            Get-DscResourceCachePath
        }

        $actual | Should -Be (Join-Path $env:LOCALAPPDATA 'dsc' 'resourceCache')
    }

    It 'Uses the home dot-dsc resourceCache folder on non-Windows systems' {
        $actual = & (Get-Module DSCOras) {
            param($HomeDirectory)
            Set-Variable -Name IsWindows -Value $false -Scope Script
            Set-Variable -Name HOME -Value $HomeDirectory -Scope Script
            Get-DscResourceCachePath
        } $TestDrive

        $actual | Should -Be (Join-Path $TestDrive '.dsc' 'resourceCache')
    }
}
