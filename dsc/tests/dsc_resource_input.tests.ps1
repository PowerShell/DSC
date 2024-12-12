# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'tests for resource input' {
    BeforeAll {
        $manifest = @'
    {
        "$schema": "https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2024/04/bundled/resource/manifest.json",
        "type": "Test/EnvVarInput",
        "version": "0.1.0",
        "get": {
            "executable": "pwsh",
            "input": "env",
            "args": [
                "-NoLogo",
                "-NonInteractive",
                "-NoProfile",
                "-Command",
                "\"{ `\"Hello`\": `\"$env:Hello`\", `\"World`\": `\"$env:World`\", `\"Boolean`\": `\"$env:Boolean`\", `\"StringArray`\": `\"$env:StringArray`\", `\"NumberArray`\": `\"$env:NumberArray`\" }\""
            ]
        },
        "set": {
            "executable": "pwsh",
            "input": "env",
            "args": [
                "-NoLogo",
                "-NonInteractive",
                "-NoProfile",
                "-Command",
                "\"{ `\"Hello`\": `\"$env:Hello`\", `\"World`\": `\"$env:World`\", `\"Boolean`\": `\"$env:Boolean`\", `\"StringArray`\": `\"$env:StringArray`\", `\"NumberArray`\": `\"$env:NumberArray`\" }\""
            ],
            "return": "state",
            "implementsPretest": true
        },
        "test": {
            "executable": "pwsh",
            "input": "env",
            "args": [
                "-NoLogo",
                "-NonInteractive",
                "-NoProfile",
                "-Command",
                "\"{ `\"Hello`\": `\"$env:Hello`\", `\"World`\": `\"$env:World`\", `\"Boolean`\": `\"$env:Boolean`\", `\"StringArray`\": `\"$env:StringArray`\", `\"NumberArray`\": `\"$env:NumberArray`\" }\""
            ]
        },
        "schema": {
            "embedded": {
                "$schema": "http://json-schema.org/draft-07/schema#",
                "$id": "https://test",
                "title": "test",
                "description": "test",
                "type": "object",
                "required": [],
                "additionalProperties": false,
                "properties": {
                    "Hello": {
                        "type": "string",
                        "description": "test"
                    },
                    "World": {
                        "type": "number",
                        "description": "test"
                    },
                    "Boolean": {
                        "type": "boolean",
                        "description": "test"
                    },
                    "StringArray": {
                        "type": "array",
                        "description": "test",
                        "items": {
                            "type": "string"
                        }
                    },
                    "NumberArray": {
                        "type": "array",
                        "description": "test",
                        "items": {
                            "type": "number"
                        }
                    }
                }
            }
        }
    }
'@
        $oldPath = $env:DSC_RESOURCE_PATH
        $env:DSC_RESOURCE_PATH = $TestDrive
        Set-Content $TestDrive/EnvVarInput.dsc.resource.json -Value $manifest
    }

    AfterAll {
        $env:DSC_RESOURCE_PATH = $oldPath
    }

    It 'Input can be sent to the resource for: <operation>' -TestCases @(
        @{ operation = 'get'; member = 'actualState' }
        @{ operation = 'set'; member = 'afterState' }
        @{ operation = 'test'; member = 'actualState' }
    ) {
        param($operation, $member)

        $json = @"
        {
            "Hello": "foo",
            "World": 2,
            "Boolean": true,
            "StringArray": ["foo", "bar"],
            "NumberArray": [1, 2, 3]
        }
"@

        $result = $json | dsc resource $operation -r Test/EnvVarInput -f - | ConvertFrom-Json
        $result.$member.Hello | Should -BeExactly 'foo'
        $result.$member.World | Should -Be 2
        $result.$member.Boolean | Should -Be 'true'
        $result.$member.StringArray | Should -BeExactly 'foo,bar'
        $result.$member.NumberArray | Should -BeExactly '1,2,3'
    }
}
