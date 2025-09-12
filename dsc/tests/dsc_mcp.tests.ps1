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
        $response.result.tools.Count | Should -Be 2
        $response.result.tools[0].name | Should -BeIn @('list_adapted_resources', 'list_dsc_resources')
        $response.result.tools[1].name | Should -BeIn @('list_adapted_resources', 'list_dsc_resources')
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
        $resources = dsc resource list | ConvertFrom-Json -Depth 20 | Select-Object type, kind, description -Unique
        $response.result.structuredContent.resources.Count | Should -Be $resources.Count
        for ($i = 0; $i -lt $resources.Count; $i++) {
            ($response.result.structuredContent.resources[$i].psobject.properties | Measure-Object).Count | Should -Be 3
            $response.result.structuredContent.resources[$i].type | Should -BeExactly $resources[$i].type -Because ($response.result.structuredContent | ConvertTo-Json -Depth 20 | Out-String)
            $response.result.structuredContent.resources[$i].kind | Should -BeExactly $resources[$i].kind -Because ($response.result.structuredContent | ConvertTo-Json -Depth 20 | Out-String)
            $response.result.structuredContent.resources[$i].description | Should -BeExactly $resources[$i].description -Because ($response.result.structuredContent | ConvertTo-Json -Depth 20 | Out-String)
        }
    }

    It 'Calling list_adapted_resources works' {
        $mcpRequest = @{
            jsonrpc = "2.0"
            id = 4
            method = "tools/call"
            params = @{
                name = "list_adapted_resources"
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

    It 'Calling list_adapted_resources with no matches works' {
        $mcpRequest = @{
            jsonrpc = "2.0"
            id = 5
            method = "tools/call"
            params = @{
                name = "list_adapted_resources"
                arguments = @{
                    adapter = "Non.Existent/Adapter"
                }
            }
        }

        $response = Send-McpRequest -request $mcpRequest
        $response.id | Should -Be 5
        $response.result.structuredContent.resources.Count | Should -Be 0
    }
}
