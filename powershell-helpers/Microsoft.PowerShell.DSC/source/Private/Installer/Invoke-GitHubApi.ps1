function Invoke-GitHubApi
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
                $_.ErrorDetails.Message | ConvertFrom-Json | ConvertTo-GitHubErrorRecord | Write-Error
            }
        }