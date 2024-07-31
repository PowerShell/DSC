function Install-DscExe 
{
    [CmdletBinding(SupportsShouldProcess)]
    param 
    ()

    begin 
    {
        $commandName = $MyInvocation.MyCommand.Name 
        Write-Verbose ("Starting: {0}" -f $commandName)

        function InvokeGitHubApi
        {
            [CmdletBinding()]
            param (
                [Parameter(Mandatory, Position = 0)]
                [string] $Uri,
                [Uri] $BaseUri = [Uri]::new('https://api.github.com'),

                # HTTP headers
                [HashTable] $Headers = @{Accept = 'application/vnd.github.v3+json' },

                # HTTP request method
                [Microsoft.PowerShell.Commands.WebRequestMethod] $Method = [Microsoft.PowerShell.Commands.WebRequestMethod]::Get,

                # Request body or query parameters for GET requests
                $Body,

                # File path to use as body (instead of $Body).
                [string] $InFile,

                # Accept header to be added (for accessing preview APIs or different resource representations)
                [string[]] $Accept,

                [switch] $Anonymous,
                [Security.SecureString] $Token = $null
            )

            $Headers['User-Agent'] = 'PowerShell PSGitHub'

            if ($Accept)
            {
                $Headers.Accept = ($Accept -join ',')
            }

            # Resolve the Uri parameter with https://api.github.com as a base URI
            # This allows to call this function with just a path,
            # but also supply a full URI (e.g. for a GitHub enterprise instance)
            $Uri = [Uri]::new($BaseUri, $Uri)

            $apiRequest = @{
                Headers       = $Headers;
                Uri           = $Uri;
                Method        = $Method;
                # enable automatic pagination
                # use | Select-Object -First to limit the result
                FollowRelLink = $true;
            };

            # If the caller hasn't specified the -Anonymous switch parameter, then add the HTTP Authorization header
            # to authenticate the HTTP request.
            if (!$Anonymous -and $Token)
            {
                $apiRequest.Authentication = 'Bearer'
                $apiRequest.Token = $Token
            }
            else
            {
                Write-Verbose -Message 'Making request without API token'
            }

            ### Append the HTTP message body (payload), if the caller specified one.
            if ($Body)
            {
                $apiRequest.Body = $Body
                Write-Debug -Message ("Request body: " + ($Body | Out-String))
            }
            if ($InFile)
            {
                $apiRequest.InFile = $InFile
            }

            # We need to communicate using TLS 1.2 against GitHub.
            [Net.ServicePointManager]::SecurityProtocol = 'tls12'

            # Invoke the REST API
            try
            {
                Write-Verbose ($apiRequest | ConvertTo-Json | Out-String)
                Invoke-RestMethod @apiRequest -ResponseHeadersVariable responseHeaders
                if ($responseHeaders.ContainsKey('X-RateLimit-Limit'))
                {
                    Write-Verbose "Rate limit total: $($responseHeaders['X-RateLimit-Limit'])"
                    Write-Verbose "Rate limit remaining: $($responseHeaders['X-RateLimit-Remaining'])"
                    $resetUnixSeconds = [int]($responseHeaders['X-RateLimit-Reset'][0])
                    $resetDateTime = ([System.DateTimeOffset]::FromUnixTimeSeconds($resetUnixSeconds)).DateTime
                    Write-Verbose "Rate limit resets: $resetDateTime"
                }
            }
            catch
            {
                if (
                    $_.Exception.PSObject.TypeNames -notcontains 'Microsoft.PowerShell.Commands.HttpResponseException' -and # PowerShell Core
                    $_.Exception -isnot [System.Net.WebException] # Windows PowerShell
                )
                {
                    # Throw any error that is not a HTTP response error (e.g. server not reachable)
                    throw $_
                }
                # This is the only way to get access to the response body for errors in old PowerShell versions.
                # PowerShell >=7.0 could use -SkipHttpErrorCheck with -StatusCodeVariable
                $_.ErrorDetails.Message | ConvertFrom-Json | ConvertToGitHubErrorRecord | Write-Error
            }
        }

        function ConvertToGitHubErrorRecord
        {
            [CmdletBinding()]
            [OutputType([ErrorRecord])]
            param (
                [Parameter(Mandatory, ValueFromPipeline)]
                [ValidateNotNull()]
                [PSObject] $Err
            )
            process
            {
                $message = ""
                $errorId = $null
                $docUrl = $null
                if ($null -ne $Err.PSObject.Properties['code'])
                {
                    $errorId = $Err.code
                    $message += "$($Err.code): "
                }
                if ($null -ne $Err.PSObject.Properties['field'])
                {
                    $message += "Field `"$($Err.field)`": "
                }
                if ($null -ne $Err.PSObject.Properties['message'])
                {
                    $message += $Err.message
                }
                if ($null -ne $Err.PSObject.Properties['documentation_url'])
                {
                    $docUrl = $Err.documentation_url
                }
                # Validation errors have nested errors
                $exception = if ($null -ne $Err.PSObject.Properties['errors'])
                {
                    [AggregateException]::new($message, @($Err.errors | ConvertTo-GitHubErrorRecord | ForEach-Object Exception -Confirm:$false))
                }
                else
                {
                    [Exception]::new($message)
                }
                $exception.HelpLink = $docUrl
                [ErrorRecord]::new($exception, $errorId, [ErrorCategory]::NotSpecified, $null)
            }
        }

        function TestAdministrator  
        {  
            $user = [Security.Principal.WindowsIdentity]::GetCurrent();
            (New-Object Security.Principal.WindowsPrincipal $user).IsInRole([Security.Principal.WindowsBuiltinRole]::Administrator)  
        }

        function TestIsOsArchX64
        {
            if ($PSVersionTable.PSVersion.Major -lt 6)
            {
                return (Get-CimInstance -ClassName Win32_OperatingSystem).OSArchitecture -match '64'
            }
        
            return [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture -eq [System.Runtime.InteropServices.Architecture]::X64
        }

        function GetAvailablePackageManager
        {
            if (Get-Command 'apt' -ErrorAction SilentlyContinue)
            {
                return 'apt'
            }

            if (Get-Command 'dnf' -ErrorAction SilentlyContinue)
            {
                return 'dnf'
            }

            if (Get-Command 'yum' -ErrorAction SilentlyContinue)
            {
                return 'yum'
            }

            if (Get-Command 'zypper' -ErrorAction SilentlyContinue)
            {
                return 'zypper'
            }
        }

        function GetPlatformInformation
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
                    $pacMan = GetAvailablePackageManager
        
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

                    $exePath = if (TestAdministrator) { "$env:ProgramFiles\DSC" } else { "$env:LOCALAPPDATA\DSC" }
                }
            }

            # TODO: If latest is present, just change it to always point to latest release
            $releases = InvokeGitHubApi -Uri "repos/PowerShell/DSC/releases"

            [uri]$latestAsset = ($releases  | Sort-Object created_at -Descending | Select-Object -First 1).assets_url

            # invoke the rest api call
            $resp = InvokeGitHubApi -Uri $latestAsset.LocalPath
            
            $downloadUrl = $resp.browser_download_url | Where-Object { $_ -like "*$platform*" }


        
            $info = @{
                FileName           = ($downloadUrl.Split("/")[-1])
                ExePath            = $exePath
                Platform           = $platform
                FileUri            = $downloadUrl
                Extension          = [System.IO.Path]::GetExtension($downloadUrl)
            }

            if ($IsWindows)
            {
                $info['RunAsAdmin'] = TestAdministrator
            }
        
            if ($pacMan)
            {
                $info['PackageManager'] = $pacMan
            }
        
            return $info
        }

        function SaveWithBitsTransfer {
            param(
                [Parameter(Mandatory=$true)]
                [string]
                $FileUri,
        
                [Parameter(Mandatory=$true)]
                [string]
                $Destination,
        
                [Parameter(Mandatory=$true)]
                [string]
                $AppName
            )
        
            Write-Host "`nDownloading latest $AppName..." -ForegroundColor Yellow
        
            Remove-Item -Force $Destination -ErrorAction SilentlyContinue
        
            $bitsDl = Start-BitsTransfer $FileUri -Destination $Destination -Asynchronous
        
            while (($bitsDL.JobState -eq 'Transferring') -or ($bitsDL.JobState -eq 'Connecting')) {
                Write-Progress -Activity "Downloading: $AppName" -Status "$([math]::round($bitsDl.BytesTransferred / 1mb))mb / $([math]::round($bitsDl.BytesTotal / 1mb))mb" -PercentComplete ($($bitsDl.BytesTransferred) / $($bitsDl.BytesTotal) * 100 )
            }
        
            switch ($bitsDl.JobState) {
        
                'Transferred' {
                    Complete-BitsTransfer -BitsJob $bitsDl
                    break
                }
        
                'Error' {
                    throw 'Error downloading installation media.'
                }
            }
        }

        function SetDscResourcePath
        {
            [CmdletBinding()]
            param 
            (
                [Parameter(Mandatory = $false)]
                [System.EnvironmentVariableTarget]
                $VariableTarget = [System.EnvironmentVariableTarget]::User,

                [Parameter(Mandatory = $true)]
                [System.String]
                $Path       
            )

            if (-not ([System.Environment]::GetEnvironmentVariable("DSC_RESOURCE_PATH", $VariableTarget)))
            {
                Write-Verbose -Message "Adding '$Path' to 'DSC_RESOURCE_PATH' variable on '$VariableTarget'"
                [System.Environment]::SetEnvironmentVariable("DSC_RESOURCE_PATH", $Path, $VariableTarget)

                $env:DSC_RESOURCE_PATH = [System.Environment]::GetEnvironmentVariable("DSC_RESOURCE_PATH", $VariableTarget)

                return
            }
        }
    }
    process 
    {
        try
        {
            $prevProgressPreference = $ProgressPreference
            $ProgressPreference = 'SilentlyContinue'

            $platformInfo = GetPlatformInformation
            # Download the installer
            $tmpdir = [System.IO.Path]::GetTempPath()

            $installerPath = [System.IO.Path]::Combine($tmpDir, $platformInfo.FileName)
            
            if ($PSVersionTable.PSVersion.Major -le 5)
            {
                SaveWithBitsTransfer -FileUri $platformInfo.FileUri -Destination $installerPath -AppName $platformInfo.AppName
            }
            elseif ($PSCmdlet.ShouldProcess($platformInfo.FileUri, "Invoke-WebRequest -OutFile $installerPath"))
            {
                Invoke-WebRequest -Uri $platformInfo.FileUri -OutFile $installerPath
            }

            # switch to install on different platforms based on extension

            switch ($platformInfo.Extension)
            {   
                # On windows and ...
                '.zip' 
                {
                    $zipDirPath = [System.IO.Path]::Combine($tmpdir, 'DSC')
                    Expand-Archive -LiteralPath $installerPath -DestinationPath $zipDirPath -Force
                    Move-Item "$zipDirPath/*" -Destination $platformInfo.ExePath -Force -ErrorAction SilentlyContinue
                    break
                }

                # TODO: Add other platforms
            }

            if ($platformInfo.RunAsAdministrator)
            {
                SetDscResourcePath -Path $platformInfo.ExePath -VariableTarget Machine
            }
            else
            {
                SetDscResourcePath -Path $platformInfo.ExePath
            }
        }
        finally {
            $ProgressPreference = $prevProgressPreference
        }
    }

    end
    {
        Write-Verbose ("Ended: {0}" -f $MyInvocation.MyCommand.Name)
    }
}