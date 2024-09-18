function Get-ProcessOutput 
{
    [CmdletBinding()]
    Param 
    (
        [Parameter(Mandatory = $true)]
        [System.Diagnostics.Process]
        $Process,

        [Parameter(Mandatory = $false)]
        [ValidateSet('StandardOutput', 'StandardError')]
        [System.String]
        $ReadLine = 'StandardOutput'
    )

    # TODO: Can determine if classes can be created to have more strong-typed
    $output = [System.Collections.Generic.List[object]]::new()

    do
    {
        $line = $Process.$ReadLine.ReadLine()

        if ($line)
        {
            try 
            {
                $jsonOutput = $line | ConvertFrom-Json    

                # add to output
                $output.Add($jsonOutput)
            }
            catch 
            {
                $msg = "Could not convert '$line' to JSON."
                Write-Debug -Message $msg
            }
        }
        else 
        {
            break
        }
    } while ($true)

    return $output
}