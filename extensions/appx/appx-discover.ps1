$packages = Get-AppxPackage
foreach ($package in $packages) {
    $manifests = Get-ChildItem -Path "$($package.InstallLocation)\*" -File -Include '*.dsc.resource.json','*.dsc.resource.yaml','*.dsc.resource.yml' -ErrorAction Ignore
    foreach ($manifest in $manifests) {
        @{ manifestPath = $manifest.FullName } | ConvertTo-Json -Compress
    }
}
