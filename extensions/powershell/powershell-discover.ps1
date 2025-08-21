$psPaths = $env:PSModulePath -split [System.IO.Path]::PathSeparator | Where-Object { $_ -notmatch 'WindowsPowerShell' }

$m = [System.Collections.Concurrent.ConcurrentBag[hashtable]]::new()

$psPaths | ForEach-Object -Parallel {
    $queue = $using:m
    $files = Get-ChildItem -Path $_ -Recurse -File -Filter '*.dsc.resource.*' -ErrorAction Ignore | 
             Where-Object -Property Extension -In @('.json', '.yaml', '.yml')
    
    foreach ($file in $files) {
        $queue.Add(@{ manifestPath = $file.FullName })
    }
} -ThrottleLimit 10

@($m) | ConvertTo-Json -Compress