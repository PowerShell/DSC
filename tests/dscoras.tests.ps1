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
                        manifests = @('bundle.dsc.manifests.json')
                    }
                }
                Windows = @{
                    'Microsoft.Test.Bundle' = @{
                        zipContents = @('shared.exe', 'nested\support.ps1')
                        manifests = @('bundle.dsc.manifests.json', 'windows.dsc.resource.json')
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
        @(Get-Command -Module DSCOras).Name | Should -Be @('Find-Artifact', 'Publish-Artifact')
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
        Should -Invoke Connect-DscRegistry -ModuleName DSCOras -Exactly 1
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
