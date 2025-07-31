[CmdletBinding()]
param (
    [Parameter(ValueFromPipeline = $true)]
    [string]$stringInput
)

return "{}"

# begin {
#     $lines = [System.Collections.Generic.List[string]]::new()

#     $scriptModule = Import-Module "$PSScriptRoot/convertDscResource.psd1" -Force -PassThru
# }

# process {
#     # Process each line of input
#     foreach ($line in $stringInput) {
#         $lines.Add($line)  
#     }
# }

# end {
#     if ($lines.Count -ne 0) {
#         $result = $scriptModule.invoke( { param($lines) Build-DscConfigDocument -Content $lines }, ($lines | Out-String) )

#         return ($result | ConvertTo-Json -Depth 10)
#     }
# }