# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Tests for MCP server' {
    BeforeAll {
        $processStartInfo = [System.Diagnostics.ProcessStartInfo]::new()
        $processStartInfo.FileName = "dsc"
        $processStartInfo.Arguments = "--trace-format plaintext mcp"
        $processStartInfo.UseShellExecute = $false
        $processStartInfo.RedirectStandardError = $true
        $processStartInfo.RedirectStandardOutput = $true
        $processStartInfo.RedirectStandardInput = $true
        $mcp = [System.Diagnostics.Process]::Start($processStartInfo)

        function Send-McpRequest($request, [switch]$notify) {
            $request = $request | ConvertTo-Json -Compress -Depth 10
            $mcp.StandardInput.WriteLine($request)
            $mcp.StandardInput.Flush()
            if (!$notify) {
                while ($mcp.StandardOutput.Peek() -eq -1) {
                    Start-Sleep -Milliseconds 100
                }
                $stdout = $mcp.StandardOutput.ReadLine()
                return ($stdout | ConvertFrom-Json -Depth 30)
            }
        }
    }

    AfterAll {
        $mcp.StandardInput.Close()
        $mcp.WaitForExit()
    }

    It 'Initialization works' {
        $mcpRequest = @{
            jsonrpc = "2.0"
            id      = 1
            method  = "initialize"
            params  = @{
                protocolVersion = "2024-11-05"
                capabilities    = @{
                    tools = @{}
                }
                clientInfo      = @{
                    name    = "Test Client"
                    version = "1.0.0"
                }
            }
        }

        $response = Send-McpRequest -request $mcpRequest

        $response.id | Should -Be 1
        $response.result.capabilities.tools | Should -Not -Be $null
        $response.result.instructions | Should -Not -BeNullOrEmpty

        $notifyInitialized = @{
            jsonrpc = "2.0"
            method  = "notifications/initialized"
        }

        Send-McpRequest -request $notifyInitialized -notify
    }

    It 'Tools/List works' {
        $mcpRequest = @{
            jsonrpc = "2.0"
            id      = 2
            method  = "tools/list"
            params  = @{}
        }

        $tools = @{
            'invoke_dsc_config'   = $false
            'invoke_dsc_resource' = $false
            'list_dsc_functions'  = $false
            'list_dsc_resources'  = $false
            'show_dsc_resource'   = $false
        }

        $response = Send-McpRequest -request $mcpRequest
        $response.id | Should -Be 2
        $response.result.tools.Count | Should -Be $tools.Count
        foreach ($tool in $response.result.tools) {
            $tools.ContainsKey($tool.name) | Should -Be $true
            $tools[$tool.name] = $true
            $tool.description | Should -Not -BeNullOrEmpty
        }
        foreach ($tool in $tools.GetEnumerator()) {
            $tool.Value | Should -Be $true -Because "Tool '$($tool.Key)' was not found in the list of tools"
        }
    }

    It 'Calling list_dsc_resources works' {
        $mcpRequest = @{
            jsonrpc = "2.0"
            id      = 3
            method  = "tools/call"
            params  = @{
                name      = "list_dsc_resources"
                arguments = @{}
            }
        }

        $response = Send-McpRequest -request $mcpRequest
        $response.id | Should -BeGreaterOrEqual 3
        $resources = dsc resource list | ConvertFrom-Json -Depth 20 | Select-Object type, kind, description -Unique
        $response.result.structuredContent.resources.Count | Should -Be $resources.Count
        for ($i = 0; $i -lt $resources.Count; $i++) {
            ($response.result.structuredContent.resources[$i].psobject.properties | Measure-Object).Count | Should -BeGreaterOrEqual 3
            $response.result.structuredContent.resources[$i].type | Should -BeExactly $resources[$i].type -Because ($response.result.structuredContent | ConvertTo-Json -Depth 20 | Out-String)
            $response.result.structuredContent.resources[$i].kind | Should -BeExactly $resources[$i].kind -Because ($response.result.structuredContent | ConvertTo-Json -Depth 20 | Out-String)
            $response.result.structuredContent.resources[$i].description | Should -BeExactly $resources[$i].description -Because ($response.result.structuredContent | ConvertTo-Json -Depth 20 | Out-String)
        }
    }

    It 'Calling list_dsc_resources with adapter works' {
        $mcpRequest = @{
            jsonrpc = "2.0"
            id      = 4
            method  = "tools/call"
            params  = @{
                name      = "list_dsc_resources"
                arguments = @{
                    adapter = "Microsoft.DSC/PowerShell"
                }
            }
        }

        $response = Send-McpRequest -request $mcpRequest
        $response.id | Should -Be 4
        $resources = dsc resource list --adapter Microsoft.DSC/PowerShell | ConvertFrom-Json -Depth 20
        $response.result.structuredContent.resources.Count | Should -Be $resources.Count
        for ($i = 0; $i -lt $resources.Count; $i++) {
            ($response.result.structuredContent.resources[$i].psobject.properties | Measure-Object).Count | Should -Be 4
            $response.result.structuredContent.resources[$i].type | Should -BeExactly $resources[$i].type -Because ($response.result.structuredContent | ConvertTo-Json -Depth 20 | Out-String)
            $response.result.structuredContent.resources[$i].require_adapter | Should -BeExactly $resources[$i].require_adapter -Because ($response.result.structuredContent | ConvertTo-Json -Depth 20 | Out-String)
            $response.result.structuredContent.resources[$i].description | Should -BeExactly $resources[$i].description -Because ($response.result.structuredContent | ConvertTo-Json -Depth 20 | Out-String)
        }
    }

    It 'Calling list_dsc_resources with <adapter> returns error' -TestCases @(
        @{"adapter" = "Non.Existent/Adapter" },
        @{"adapter" = "Microsoft.DSC.Debug/Echo" }
    ) {
        param($adapter)

        $mcpRequest = @{
            jsonrpc = "2.0"
            id      = 5
            method  = "tools/call"
            params  = @{
                name      = "list_dsc_resources"
                arguments = @{
                    adapter = $adapter
                }
            }
        }

        $response = Send-McpRequest -request $mcpRequest
        $response.id | Should -Be 5
        $response.error.code | Should -Be -32602
        $response.error.message | Should -Not -BeNullOrEmpty
    }

    It 'Calling show_dsc_resource works' {
        $resource = (dsc resource list 'Microsoft/OSInfo' | ConvertFrom-Json -Depth 20)

        $mcpRequest = @{
            jsonrpc = "2.0"
            id      = 6
            method  = "tools/call"
            params  = @{
                name      = "show_dsc_resource"
                arguments = @{
                    type = $resource.type
                }
            }
        }

        $response = Send-McpRequest -request $mcpRequest
        $response.id | Should -Be 6
        ($response.result.structuredContent.psobject.properties | Measure-Object).Count | Should -BeGreaterOrEqual 4
        $because = ($response.result.structuredContent | ConvertTo-Json -Depth 20 | Out-String)
        $response.result.structuredContent.type | Should -BeExactly $resource.type -Because $because
        $response.result.structuredContent.kind | Should -BeExactly $resource.kind -Because $because
        $response.result.structuredContent.version | Should -Be $resource.version -Because $because
        $response.result.structuredContent.capabilities | Should -Be $resource.capabilities -Because $because
        $response.result.structuredContent.description | Should -Be $resource.description -Because $because
        $schema = (dsc resource schema --resource $resource.type | ConvertFrom-Json -Depth 20)
        $response.result.structuredContent.schema.'$id' | Should -Be $schema.'$id' -Because $because
        $response.result.structuredContent.schema.type | Should -Be $schema.type -Because $because
        $response.result.structuredContent.schema.properties.keys | Should -Be $schema.properties.keys -Because $because
    }

    It 'Calling show_dsc_resource with non-existent resource returns error' {
        $mcpRequest = @{
            jsonrpc = "2.0"
            id      = 7
            method  = "tools/call"
            params  = @{
                name      = "show_dsc_resource"
                arguments = @{
                    type = "Non.Existent/Resource"
                }
            }
        }

        $response = Send-McpRequest -request $mcpRequest
        $response.id | Should -Be 7
        $response.error.code | Should -Be -32602
        $response.error.message | Should -Not -BeNullOrEmpty
    }

    It 'Calling list_dsc_functions works' {
        $mcpRequest = @{
            jsonrpc = "2.0"
            id      = 8
            method  = "tools/call"
            params  = @{
                name      = "list_dsc_functions"
                arguments = @{}
            }
        }

        $response = Send-McpRequest -request $mcpRequest
        $response.id | Should -Be 8
        $functions = dsc function list --output-format json | ConvertFrom-Json
        $response.result.structuredContent.functions.Count | Should -Be $functions.Count

        $mcpFunctions = $response.result.structuredContent.functions | Sort-Object name
        $dscFunctions = $functions | Sort-Object name

        for ($i = 0; $i -lt $dscFunctions.Count; $i++) {
            ($mcpFunctions[$i].psobject.properties | Measure-Object).Count | Should -BeGreaterOrEqual 8
            $mcpFunctions[$i].name | Should -BeExactly $dscFunctions[$i].name -Because ($response.result.structuredContent | ConvertTo-Json -Depth 10 | Out-String)
            $mcpFunctions[$i].category | Should -BeExactly $dscFunctions[$i].category -Because ($response.result.structuredContent | ConvertTo-Json -Depth 10 | Out-String)
            $mcpFunctions[$i].description | Should -BeExactly $dscFunctions[$i].description -Because ($response.result.structuredContent | ConvertTo-Json -Depth 10 | Out-String)
        }
    }

    It 'Calling list_dsc_functions with function_filter filter works' {
        $mcpRequest = @{
            jsonrpc = "2.0"
            id      = 9
            method  = "tools/call"
            params  = @{
                name      = "list_dsc_functions"
                arguments = @{
                    function_filter = "array"
                }
            }
        }

        $response = Send-McpRequest -request $mcpRequest
        $response.id | Should -Be 9
        $response.result.structuredContent.functions.Count | Should -Be 1
        $response.result.structuredContent.functions[0].name | Should -BeExactly "array"
        $response.result.structuredContent.functions[0].category | Should -BeExactly "Array"
    }

    It 'Calling list_dsc_functions with wildcard pattern works' {
        $mcpRequest = @{
            jsonrpc = "2.0"
            id      = 10
            method  = "tools/call"
            params  = @{
                name      = "list_dsc_functions"
                arguments = @{
                    function_filter = "*Array*"
                }
            }
        }

        $response = Send-McpRequest -request $mcpRequest
        $response.id | Should -Be 10
        $arrayFunctions = dsc function list --output-format json | ConvertFrom-Json -Depth 20 | Where-Object { $_.name -like "*Array*" }
        $response.result.structuredContent.functions.Count | Should -Be $arrayFunctions.Count
        foreach ($func in $response.result.structuredContent.functions) {
            $func.name | Should -Match "Array" -Because "Function name should contain 'Array'"
        }
    }

    # dont check for error as dsc function list returns empty list for invalid patterns
    It 'Calling list_dsc_functions with invalid pattern returns empty result' {
        $mcpRequest = @{
            jsonrpc = "2.0"
            id      = 11
            method  = "tools/call"
            params  = @{
                name      = "list_dsc_functions"
                arguments = @{
                    function_filter = "[invalid]"
                }
            }
        }

        $response = Send-McpRequest -request $mcpRequest
        $response.id | Should -Be 11
        $response.result.structuredContent.functions.Count | Should -Be 0
        $response.result.structuredContent.functions | Should -BeNullOrEmpty
    }

    It 'Calling invoke_dsc_resource for operation: <operation>' -TestCases @(
        @{ operation = 'get'; property = 'actualState' }
        @{ operation = 'set'; property = 'beforeState' }
        @{ operation = 'test'; property = 'desiredState' }
        @{ operation = 'export'; property = 'actualState' }
    ) {
        param($operation, $property)

        $mcpRequest = @{
            jsonrpc = "2.0"
            id      = 12
            method  = "tools/call"
            params  = @{
                name      = "invoke_dsc_resource"
                arguments = @{
                    type            = 'Test/Operation'
                    operation       = $operation
                    resource_type   = 'Test/Operation'
                    properties_json = (@{
                            hello  = "World"
                            action = $operation
                        } | ConvertTo-Json -Depth 20)
                }
            }
        }

        $response = Send-McpRequest -request $mcpRequest
        $response.id | Should -Be 12
        $because = ($response | ConvertTo-Json -Depth 20 | Out-String)
        ($response.result.structuredContent.psobject.properties | Measure-Object).Count | Should -Be 1 -Because $because
        $response.result.structuredContent.result.$property.action | Should -BeExactly $operation -Because $because
        $response.result.structuredContent.result.$property.hello | Should -BeExactly "World" -Because $because
    }

    It 'Calling invoke_dsc_resource for delete operation' {
        $mcpRequest = @{
            jsonrpc = "2.0"
            id      = 12
            method  = "tools/call"
            params  = @{
                name      = "invoke_dsc_resource"
                arguments = @{
                    type            = 'Test/Operation'
                    operation       = 'delete'
                    resource_type   = 'Test/Operation'
                    properties_json = (@{
                            hello  = "World"
                            action = 'delete'
                        } | ConvertTo-Json -Depth 20)
                }
            }
        }

        $response = Send-McpRequest -request $mcpRequest
        $response.id | Should -Be 12
        $because = ($response | ConvertTo-Json -Depth 20 | Out-String)
        ($response.result.structuredContent.psobject.properties | Measure-Object).Count | Should -Be 1 -Because $because
        $response.result.structuredContent.result.success | Should -Be $true -Because $because
    }

    It 'Calling invoke_dsc_config for operation: <operation>' -TestCases @(
        @{ operation = 'get' }
        @{ operation = 'set' }
        @{ operation = 'test' }
    ) {
        param($operation)

        $config = @{
            '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/config/document.json'
            resources = @(
                @{
                    name       = 'TestOperation'
                    type       = 'Test/Operation'
                    properties = @{
                        hello  = 'Hello from config'
                        action = $operation
                    }
                }
            )
        }

        $mcpRequest = @{
            jsonrpc = "2.0"
            id      = 13
            method  = "tools/call"
            params  = @{
                name      = "invoke_dsc_config"
                arguments = @{
                    operation     = $operation
                    configuration = ($config | ConvertTo-Json -Depth 20)
                }
            }
        }

        $response = Send-McpRequest -request $mcpRequest
        $response.id | Should -Be 13
        $because = ($response | ConvertTo-Json -Depth 20 | Out-String)
        $response.result.structuredContent.result.results | Should -Not -BeNullOrEmpty -Because $because
        $response.result.structuredContent.result.results.Count | Should -Be 1 -Because $because
        $response.result.structuredContent.result.results[0].name | Should -Be 'TestOperation' -Because $because
    }

    It 'Calling invoke_dsc_config for export operation' {
        $config = @{
            '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/config/document.json'
            resources = @(
                @{
                    name       = 'TestExport'
                    type       = 'Test/Export'
                    properties = @{
                        count = 2
                    }
                }
            )
        }

        $mcpRequest = @{
            jsonrpc = "2.0"
            id      = 13
            method  = "tools/call"
            params  = @{
                name      = "invoke_dsc_config"
                arguments = @{
                    operation     = 'export'
                    configuration = ($config | ConvertTo-Json -Depth 20)
                }
            }
        }

        $response = Send-McpRequest -request $mcpRequest
        $response.id | Should -Be 13
        $because = ($response | ConvertTo-Json -Depth 20 | Out-String)
        $response.result.structuredContent.result.result | Should -Not -BeNullOrEmpty -Because $because
        $response.result.structuredContent.result.result.resources.Count | Should -Be 2 -Because $because
        $response.result.structuredContent.result.result.resources[0].name | Should -Be 'TestName' -Because $because
    }

    It 'Calling invoke_dsc_config with parameters works' {
        $config = @{
            '$schema'  = 'https://aka.ms/dsc/schemas/v3/bundled/config/document.json'
            parameters = @{
                message = @{
                    type         = 'string'
                    defaultValue = 'default message'
                }
            }
            resources  = @(
                @{
                    name       = 'TestResource'
                    type       = 'Test/Operation'
                    properties = @{
                        hello  = "[parameters('message')]"
                        action = 'get'
                    }
                }
            )
        }

        $parameters = @{
            message = 'custom message'
        }

        $mcpRequest = @{
            jsonrpc = "2.0"
            id      = 14
            method  = "tools/call"
            params  = @{
                name      = "invoke_dsc_config"
                arguments = @{
                    operation     = 'get'
                    configuration = ($config | ConvertTo-Json -Depth 20)
                    parameters    = ($parameters | ConvertTo-Json -Depth 20)
                }
            }
        }

        $response = Send-McpRequest -request $mcpRequest
        $response.id | Should -Be 14
        $because = ($response | ConvertTo-Json -Depth 20 | Out-String)
        $response.result.structuredContent.result.results[0].result.actualState.hello | Should -Be 'custom message' -Because $because
    }

    It 'Calling invoke_dsc_config with YAML configuration works' {
        $configYaml = @'
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
  - name: TestResource
    type: Test/Operation
    properties:
      hello: Hello from YAML
      action: get
'@

        $mcpRequest = @{
            jsonrpc = "2.0"
            id      = 15
            method  = "tools/call"
            params  = @{
                name      = "invoke_dsc_config"
                arguments = @{
                    operation     = 'get'
                    configuration = $configYaml
                }
            }
        }

        $response = Send-McpRequest -request $mcpRequest
        $response.id | Should -Be 15
        $because = ($response | ConvertTo-Json -Depth 20 | Out-String)
        $response.result.structuredContent.result.results[0].result.actualState.hello | Should -Be 'Hello from YAML' -Because $because
    }

    It 'Calling invoke_dsc_config with YAML parameters works' {
        $config = @{
            '$schema'  = 'https://aka.ms/dsc/schemas/v3/bundled/config/document.json'
            parameters = @{
                greeting = @{
                    type = 'string'
                }
            }
            resources  = @(
                @{
                    name       = 'TestResource'
                    type       = 'Test/Operation'
                    properties = @{
                        hello  = "[parameters('greeting')]"
                        action = 'get'
                    }
                }
            )
        }

        $parametersYaml = @'
greeting: Hello from YAML parameters
'@

        $mcpRequest = @{
            jsonrpc = "2.0"
            id      = 16
            method  = "tools/call"
            params  = @{
                name      = "invoke_dsc_config"
                arguments = @{
                    operation     = 'get'
                    configuration = ($config | ConvertTo-Json -Depth 20)
                    parameters    = $parametersYaml
                }
            }
        }

        $response = Send-McpRequest -request $mcpRequest
        $response.id | Should -Be 16
        $because = ($response | ConvertTo-Json -Depth 20 | Out-String)
        $response.result.structuredContent.result.results[0].result.actualState.hello | Should -Be 'Hello from YAML parameters' -Because $because
    }

    It 'Calling invoke_dsc_config with invalid configuration returns error' {
        $mcpRequest = @{
            jsonrpc = "2.0"
            id      = 17
            method  = "tools/call"
            params  = @{
                name      = "invoke_dsc_config"
                arguments = @{
                    operation     = 'get'
                    configuration = '{ invalid json }'
                }
            }
        }

        $response = Send-McpRequest -request $mcpRequest
        $response.id | Should -Be 17
        $response.error.code | Should -Be -32600
        $response.error.message | Should -Match 'Invalid configuration'
    }

    It 'Calling invoke_dsc_config with invalid parameters returns error' {
        $config = @{
            '$schema' = 'https://aka.ms/dsc/schemas/v3/bundled/config/document.json'
            resources = @(
                @{
                    name       = 'TestResource'
                    type       = 'Test/Operation'
                    properties = @{
                        hello  = 'test'
                        action = 'get'
                    }
                }
            )
        }

        $mcpRequest = @{
            jsonrpc = "2.0"
            id      = 18
            method  = "tools/call"
            params  = @{
                name      = "invoke_dsc_config"
                arguments = @{
                    operation     = 'get'
                    configuration = ($config | ConvertTo-Json -Depth 20)
                    parameters    = '{[invalid'
                }
            }
        }

        $response = Send-McpRequest -request $mcpRequest
        $response.id | Should -Be 18
        $response.error.code | Should -Be -32600
        $response.error.message | Should -Match 'Invalid parameters'
    }
}
