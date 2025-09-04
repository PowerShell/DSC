# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Tests for MCP server' {
    BeforeAll {
        $processStartInfo = [System.Diagnostics.ProcessStartInfo]::new()
        $processStartInfo.FileName = "dsc"
        $processStartInfo.Arguments = "--trace-format plaintext mcp"
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
            id = 1
            method = "initialize"
            params = @{
                protocolVersion = "2024-11-05"
                capabilities = @{
                    tools = @{}
                }
                clientInfo = @{
                    name = "Test Client"
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
            method = "notifications/initialized"
        }

        Send-McpRequest -request $notifyInitialized -notify
    }

    It 'Tools/List works' {
        $mcpRequest = @{
            jsonrpc = "2.0"
            id = 2
            method = "tools/list"
            params = @{}
        }

        $response = Send-McpRequest -request $mcpRequest

        $response.id | Should -Be 2
        $response.result.tools.Count | Should -Be 1
        $response.result.tools[0].name | Should -BeExactly 'list_dsc_resources'
    }

    It 'Calling list_dsc_resources works' {
        $mcpRequest = @{
            jsonrpc = "2.0"
            id = 3
            method = "tools/call"
            params = @{
                name = "list_dsc_resources"
                arguments = @{}
            }
        }

        $response = Send-McpRequest -request $mcpRequest
        $response.id | Should -Be 3
        $resources = dsc resource list | ConvertFrom-Json -Depth 20
        $response.result.structuredContent.resources.Count | Should -Be $resources.Count
        for ($i = 0; $i -lt $resources.Count; $i++) {
            $response.result.structuredContent.resources[$i].Resource.type | Should -BeExactly $resources[$i].type
            $response.result.structuredContent.resources[$i].Resource.version | Should -BeExactly $resources[$i].version
            $response.result.structuredContent.resources[$i].Resource.path | Should -BeExactly $resources[$i].path
        }
    }
}
