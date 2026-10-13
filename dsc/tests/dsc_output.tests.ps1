# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'output tests' {
    It 'config with output property works' {
        $configYaml = @'
        $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
        variables:
          arrayVar:
            - 1
            - 2
            - 3
        resources:
          - name: echo
            type: Microsoft.DSC.Debug/Echo
            properties:
              output: This is a test
        outputs:
          simpleText:
            type: string
            value: Hello World
          expression:
            type: string
            value: "[reference(resourceId('Microsoft.DSC.Debug/Echo', 'echo')).output]"
          conditionSucceed:
            type: int
            condition: "[equals(1, 1)]"
            value: "[variables('arrayVar')[1]]"
          conditionFail:
            type: int
            condition: "[equals(1, 2)]"
            value: "[variables('arrayVar')[1]]"
'@
        $out = dsc config get -i $configYaml | ConvertFrom-Json -Depth 10
        $LASTEXITCODE | Should -Be 0
        $out.outputs.simpleText | Should -Be 'Hello World'
        $out.outputs.expression | Should -Be 'This is a test'
        $out.outputs.conditionSucceed | Should -Be 2
        $out.outputs.conditionFail | Should -BeNullOrEmpty
    }
}
