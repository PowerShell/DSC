# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

[CmdletBinding()]
param(
    [Parameter()]
    [switch]$RelativePath,
    [Parameter()]
    [string]$Extensions
)

if ($Extensions) {
    $count = 1
    foreach ($extension in $Extensions.Split(',')) {
        $resource = [pscustomobject]@{
            manifestContent = @{
                '$schema' = "https://aka.ms/dsc/schemas/v3/bundled/adaptedresource/manifest.json"
                type = "TestDiscover/$count"
                kind = "resource"
                version = "1.0.0"
                capabilities = @("get")
                description = "Test discover $count"
                requireAdapter = "Test/Adapter"
                content = @{
                    extension = $extension
                }
                schema = @{
                    embedded = @{
                        '$schema' = "http://json-schema.org/draft-07/schema#"
                        type = "object"
                        properties = @{
                            extension = @{
                                type = "string"
                            }
                        }
                    }
                }
            }
        }
        $resource | ConvertTo-Json -Compress -Depth 10
        $count++
    }
} else {
    Get-ChildItem -Path $PSScriptRoot/resources/*.json | ForEach-Object {
        $resource = [pscustomobject]@{
            manifestPath = if ($RelativePath) {
                Resolve-Path -Path $_.FullName -Relative
            } else {
                $_.FullName
            }
        }
        $resource | ConvertTo-Json -Compress
    }
}
