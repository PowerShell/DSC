function Build-DscPathBuilder
{
    [OutputType([System.Text.StringBuilder])]
    Param 
    (
        [Parameter(Mandatory = $true, ValueFromPipeline = $true)]
        [hashtable]
        $Data,

        [Parameter(Mandatory = $true, ValueFromPipeline = $true)]
        [System.Text.StringBuilder]
        $SubCommand,
    
        [Parameter(Mandatory = $true)]
        [Alias('Name')]
        [System.String]
        $ResourceName,

        [Parameter(Mandatory = $false)]
        [Alias('Path')]
        [System.IO.FileInfo]
        $ResourcePath,

        [Parameter(Mandatory = $false)]
        [hashtable]
        $ResourceInput = @{}
    )

    if (Test-Path $ResourcePath -ErrorAction SilentlyContinue)
    {
        if ($ResourcePath.Extension -ne '.json' -and $ResourcePath.Extension -ne '.yaml' -and $ResourcePath.Extension -ne '.ps1')
        {
            Throw "No JSON, YAML or PowerShell script file was provided. Please provide valid DSC Configuration Document."
        }

        $command = " --path $($ResourcePath.FullName)"

        if ($ResourcePath.Extension -eq '.ps1')
        {
            # try converting to
        }

        [void]$subCommand.Append(" --path $($ResourcePath.FullName)")
    }
    else 
    {
        try 
        {
            $jsonOutput = $ResourceInput | ConvertTo-Json -Compress
            Write-Verbose -Message ("Starting input with:")
            Write-Verbose -Message ($jsonOutput | Out-String)
            if ($jsonOutput -eq '{}')
            {
                if ($data.exampleSnippet)
                {
                    Write-Verbose -Message "Using example snippet"
                    $jsonOutput = $data.exampleSnippet | ConvertTo-Json -Compress
                }
            }
            
            $filePath = if ($IsWindows)
            {
                Join-Path -Path $env:LOCALAPPDATA -ChildPath "dsc\dsc_tmp_configuration_doc.json"
            }
            else 
            {
                Join-Path -Path $env:HOME -ChildPath "dsc$([System.IO.Path]::DirectorySeparatorChar)dsc_tmp_configuration_doc.json"
            }

            if (-not (Test-Path $(Split-Path $filePath -Parent)))
            {
                $null = New-Item -Path $(Split-Path $filePath -Parent) -ItemType Directory -Force
            }

            Set-Content -Path $filePath -Value $jsonOutput -Force
            # TODO: The --input does not always work correctly even ProcMon states the characters are escaped correctly. Workaround for now.
            [void]$subCommand.Append(" --path $filePath")
        }
        catch 
        {
            # TODO: Capture
        }
    }
}