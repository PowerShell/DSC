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
        if ($ResourcePath.Extension -ne '.json' -and $ResourcePath.Extension -ne '.yaml')
        {
            Throw "No JSON or YAML file was provided. Please provide valid DSC Configuration Document."
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
                    $jsonOutput = $data.exampleSnippet | ConvertTo-Json -Compress
                }
            }

            $outFile = Join-Path -Path 'C:\temp\' -ChildPath 'dsc_configuration_document.json'

            Set-Content -Path $outFile -Value $jsonOutput -Force
                
            [void]$subCommand.Append(" --path $outFile")
        }
        catch 
        {
            # TODO: Capture
        }
    }
}