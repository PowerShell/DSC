function Install-DscCommand
{
    [CmdletBinding(SupportsShouldProcess)]
    param 
    ()

    begin 
    {
        $commandName = $MyInvocation.MyCommand.Name 
        Write-Verbose ("Starting: {0}" -f $commandName)
    }
    process 
    {
        try
        {
            $prevProgressPreference = $ProgressPreference
            $ProgressPreference = 'SilentlyContinue'

            # get details about the platform
            $platformInfo = Get-PlatformInformation

            # download the installer
            $tmpdir = [System.IO.Path]::GetTempPath()
            $installerPath = [System.IO.Path]::Combine($tmpDir, $platformInfo.FileName)
            
            if ($PSVersionTable.PSVersion.Major -le 5)
            {
                Save-WithBitsTransfer -FileUri $platformInfo.FileUri -Destination $installerPath -AppName $platformInfo.AppName
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
                    Expand-Archive -LiteralPath $installerPath -DestinationPath $platformInfo.ExePath -Force
                }

                # TODO: Add other platforms
            }

            if ($platformInfo.RunAsAdmin)
            {
                $platformInfo.ExePath | Add-ToPath -Persistent:$true
            }

            else
            {
                # TODO: Check when user is assigned to throw a warning when multiple environment variables are present e.g. DSC_RESOURCE_PATH
                $platformInfo.ExePath | Add-ToPath -User
            }

            Write-Information -MessageData "Successfully installed 'dsc.exe' in: $($platformInfo.ExePath)"
            dsc --version
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