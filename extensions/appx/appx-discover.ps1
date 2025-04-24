Get-AppxPackage | ForEach-Object {
    Get-ChildItem -LiteralPath $_.InstallLocation -File -Include '*.dsc.resource.json','*.dsc.resource.yaml','*.dsc.resource.yml' | ForEach-Object {
        @{ resourceManifestPath = $_.FullName } | ConvertTo-Json -Compress
    }
}
