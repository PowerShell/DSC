function Get-DscPsCacheRefreshPath
{
    $cacheFilePath = if ($IsWindows) {
        # PS 6+ on Windows
        Join-Path $env:LocalAppData "dsc\PSAdapterCache.json"
    } else {
        # PS 6+ on Linux/Mac
        Join-Path $env:HOME ".dsc" "PSAdapterCache.json"
    }

    return $cacheFilePath
}