$psPaths = $env:PSModulePath -split [System.IO.Path]::PathSeparator | Where-Object { $_ -notmatch 'WindowsPowerShell' }

$m = [System.Collections.Concurrent.ConcurrentBag[hashtable]]::new()

$psPaths | ForEach-Object -Parallel {
    $queue = $using:m
    $searchPatterns = @('*.dsc.resource.json', '*.dsc.resource.yaml', '*.dsc.resource.yml')
    foreach ($pattern in $searchPatterns) {
        try {
            [System.IO.Directory]::EnumerateFiles($_, $pattern, 'AllDirectories') | ForEach-Object {
                $queue.Add(@{ manifestPath = $_ })
            }
        } catch { }
    }
} -ThrottleLimit 30

[array]$m | ConvertTo-Json -Compress