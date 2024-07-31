function Get-PlatformInformation
{
    if ($IsWindows -or $PSVersionTable.PSVersion.Major -lt 6)
    {
        $os = 'Windows'
    }
    elseif ($IsLinux)
    {
        $os = 'Linux'
    }
    elseif ($IsMacOS)
    {
        $os = 'MacOS'
    }

    else
    {
        throw 'Could not identify operating system'
    }
        
    switch ($os)
    {
        'Linux'
        {
            $pacMan = Get-AvailablePackageManager
        
            switch ($pacMan)
            {
                # 'apt'
                # {
                #     $platform = 'linux-deb-x64'
                #     $ext = 'deb'
                #     break
                # }
        
                # { 'dnf', 'yum', 'zypper' -contains $_ }
                # {
                #     $platform = 'linux-rpm-x64'
                #     $ext = 'rpm'
                #     break
                # }
        
                default
                {
                    $platform = 'x86_64-unknown-linux-gnu'
                    break
                }
            }
        
            $exePath = '/usr/bin/dsc'
            break
        }
        
        'MacOS'
        {
            $platform = 'x86_64-apple-darwin'
        
            $exePath = '/usr/local/bin/dsc'
            break
        }
        
        'Windows'
        {
            $platform = 'x86_64-pc-windows-msvc'

            $exePath = if (Test-Administrator) { "$env:ProgramFiles\DSC" } else { "$env:LOCALAPPDATA\DSC" }
        }
    }

    # TODO: If latest is present, just change it to always point to latest release
    $releases = Invoke-GitHubApi -Uri "repos/PowerShell/DSC/releases"

    [uri]$latestAsset = ($releases  | Sort-Object created_at -Descending | Select-Object -First 1).assets_url

    # invoke the rest api call
    $resp = Invoke-GitHubApi -Uri $latestAsset.LocalPath
            
    $downloadUrl = $resp.browser_download_url | Where-Object { $_ -like "*$platform*" }


        
    $info = @{
        FileName  = ($downloadUrl.Split("/")[-1])
        ExePath   = $exePath
        Platform  = $platform
        FileUri   = $downloadUrl
        Extension = [System.IO.Path]::GetExtension($downloadUrl)
    }

    if ($IsWindows)
    {
        $info['RunAsAdmin'] = Test-Administrator
    }
        
    if ($pacMan)
    {
        $info['PackageManager'] = $pacMan
    }
        
    return $info
}