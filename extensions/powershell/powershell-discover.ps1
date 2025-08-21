$psPaths = $env:PSModulePath -split [System.IO.Path]::PathSeparator | Where-Object { $_ -notmatch 'WindowsPowerShell' }
$manifests = [System.Collections.Generic.List[hashtable]]::new()

$psPaths | ForEach-Object -Parallel {
    $queue = $using:manifests
    $files = Get-ChildItem -Path $_ -Recurse -File -Include '*.dsc.resource.json', '*.dsc.resource.yaml', '*.dsc.resource.yml' -ErrorAction Ignore
    foreach ($file in $files) {
        $m = @{ manifestPath = $file.FullName }
        $queue.Add($m)
    }
} -ThrottleLimit 10

$manifests | ConvertTo-Json -Compress