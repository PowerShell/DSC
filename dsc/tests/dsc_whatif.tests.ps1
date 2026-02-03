Describe 'whatif tests' {
    AfterEach {
        if ($IsWindows) {
            Remove-Item -Path 'HKCU:\1' -Recurse -ErrorAction Ignore
        }
    }

    It 'config set whatif when actual state matches desired state' {
        $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Hello
              type: Microsoft.DSC.Debug/Echo
              properties:
                output: hello
"@
        $what_if_result = $config_yaml | dsc config set -w -f - | ConvertFrom-Json
        $set_result = $config_yaml | dsc config set -f - | ConvertFrom-Json
        $what_if_result.metadata.'Microsoft.DSC'.executionType | Should -BeExactly 'whatIf'
        $what_if_result.results.result.beforeState.output | Should -Be $set_result.results.result.beforeState.output
        $what_if_result.results.result.afterState.output | Should -Be $set_result.results.result.afterState.output
        $what_if_result.results.result.changedProperties | Should -Be $set_result.results.result.changedProperties
        $what_if_result.hadErrors | Should -BeFalse
        $what_if_result.results.Count | Should -Be 1
        $LASTEXITCODE | Should -Be 0
    }

    It 'config set whatif when actual state does not match desired state' -Skip:(!$IsWindows) {
        # TODO: change/create cross-plat resource that implements set without just matching desired state
        $config_yaml = @"
            `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
            resources:
            - name: Registry
              type: Microsoft.Windows/Registry
              properties:
                keyPath: 'HKCU\1\2'
"@
        $what_if_result = dsc config set -w -i $config_yaml | ConvertFrom-Json
        $set_result = dsc config set -i $config_yaml | ConvertFrom-Json
        $what_if_result.metadata.'Microsoft.DSC'.executionType | Should -BeExactly 'whatIf'
        $what_if_result.results.result.beforeState._exist | Should -Be $set_result.results.result.beforeState._exist
        $what_if_result.results.result.beforeState.keyPath | Should -Be $set_result.results.result.beforeState.keyPath
        $what_if_result.results.result.afterState.KeyPath | Should -Be $set_result.results.result.afterState.keyPath
        # can be changed back to the following once _metadata is pulled out of resource return
        # $what_if_result.results.result.changedProperties | Should -Be $set_result.results.result.changedProperties
        $what_if_result.results.result.changedProperties | Should -Be @('_metadata', '_exist')
        $what_if_result.hadErrors | Should -BeFalse
        $what_if_result.results.Count | Should -Be 1
        $LASTEXITCODE | Should -Be 0

    }

    It 'config set whatif for group resource' {
        $result = dsc config set -f $PSScriptRoot/../examples/groups.dsc.yaml -w 2>&1
        $result | Should -Match 'ERROR.*?Not implemented.*?what-if'
        $LASTEXITCODE | Should -Be 2
    }

    It 'actual execution of WhatIf resource' {
        $config_yaml = @"
        `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
        resources:
        - name: WhatIf
          type: Test/WhatIf
          properties:
            executionType: Actual
"@
        $result = $config_yaml | dsc config set -f - | ConvertFrom-Json
        $result.metadata.'Microsoft.DSC'.executionType | Should -BeExactly 'actual'
        $result.results.result.afterState.executionType | Should -BeExactly 'Actual'
        $result.results.result.changedProperties | Should -Be $null
        $result.hadErrors | Should -BeFalse
        $result.results.Count | Should -Be 1
        $LASTEXITCODE | Should -Be 0
    }

    It 'what-if execution of WhatIf resource via <alias>' -TestCases @(
        @{ alias = '-w' }
        @{ alias = '--what-if' }
        @{ alias = '--dry-run' }
        @{ alias = '--noop' }
    ) {
        param($alias)
        $config_yaml = @"
        `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
        resources:
        - name: WhatIf
          type: Test/WhatIf
          properties:
            executionType: Actual
"@
        $result = $config_yaml | dsc config set $alias -f - | ConvertFrom-Json
        $result.metadata.'Microsoft.DSC'.executionType | Should -BeExactly 'whatIf'
        $result.results.result.afterState.executionType | Should -BeExactly 'WhatIf'
        $result.results.result.changedProperties | Should -BeExactly 'executionType'
        $result.hadErrors | Should -BeFalse
        $result.results.Count | Should -Be 1
        $LASTEXITCODE | Should -Be 0
    }

    It 'Test/WhatIfNative resource with set operation and WhatIfArgKind works' {
        $config_yaml = @"
        `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
        resources:
        - name: WhatIfArgKind
          type: Test/WhatIfArgKind
          properties:
            executionType: Actual
"@
        $what_if_result = $config_yaml | dsc config set -w -f - | ConvertFrom-Json
        $set_result = $config_yaml | dsc config set -f - | ConvertFrom-Json
        $what_if_result.metadata.'Microsoft.DSC'.executionType | Should -BeExactly 'whatIf'
        $set_result.metadata.'Microsoft.DSC'.executionType | Should -BeExactly 'actual'
        $what_if_result.results[0].result.afterState.executionType | Should -BeExactly 'WhatIf'
        $set_result.results[0].result.afterState.executionType | Should -BeExactly 'Actual'
        $what_if_result.hadErrors | Should -BeFalse
        $set_result.hadErrors | Should -BeFalse
    }

    It 'Echo resource with synthetic what-if works' {
        $config_yaml = @"
        `$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
        resources:
        - name: SyntheticWhatIf
          type: Microsoft.DSC.Debug/Echo
          properties:
            output: test
"@
        $what_if_result = $config_yaml | dsc config set -w -f - | ConvertFrom-Json
        $set_result = $config_yaml | dsc config set -f - | ConvertFrom-Json
        $what_if_result.metadata.'Microsoft.DSC'.executionType | Should -BeExactly 'whatIf'
        $set_result.metadata.'Microsoft.DSC'.executionType | Should -BeExactly 'actual'
        $what_if_result.hadErrors | Should -BeFalse
        $set_result.hadErrors | Should -BeFalse
        $LASTEXITCODE | Should -Be 0
    }
}
