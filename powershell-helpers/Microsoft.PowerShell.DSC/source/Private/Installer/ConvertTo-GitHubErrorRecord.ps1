function ConvertTo-GitHubErrorRecord
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