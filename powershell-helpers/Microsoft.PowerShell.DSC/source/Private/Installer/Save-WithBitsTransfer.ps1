function Save-WithBitsTransfer
{
    param(
        [Parameter(Mandatory = $true)]
        [string]
        $FileUri,

        [Parameter(Mandatory = $true)]
        [string]
        $Destination,

        [Parameter(Mandatory = $true)]
        [string]
        $AppName
    )

    Write-Information -MessageData "`nDownloading latest $AppName..."
    Remove-Item -Force $Destination -ErrorAction SilentlyContinue

    $bitsDl = Start-BitsTransfer $FileUri -Destination $Destination -Asynchronous

    while (($bitsDL.JobState -eq 'Transferring') -or ($bitsDL.JobState -eq 'Connecting'))
    {
        Write-Progress -Activity "Downloading: $AppName" -Status "$([math]::round($bitsDl.BytesTransferred / 1mb))mb / $([math]::round($bitsDl.BytesTotal / 1mb))mb" -PercentComplete ($($bitsDl.BytesTransferred) / $($bitsDl.BytesTotal) * 100 )
    }

    switch ($bitsDl.JobState)
    {

        'Transferred'
        {
            Complete-BitsTransfer -BitsJob $bitsDl
            break
        }

        'Error'
        {
            throw 'Error downloading installation media.'
        }
    }
}