Describe "DSC Configuration Migration Module tests" {
    BeforeAll {
        $modPath = (Resolve-Path -Path "$PSScriptRoot\..\dscCfgMigMod.psd1").Path
        $modLoad = Import-Module $modPath -Force -PassThru
    }

    Context "ConvertTo-DscYaml" {
        It "Should create an empty resource block" {
            $res = (ConvertTo-DscYaml -Path 'idonotexist' | ConvertFrom-Yaml)
            $res.resources | Should -BeNullOrEmpty
        }
    }

    Context "ConvertTo-DscJson" {
        It "Should create an empty resource block" {
            $res = (ConvertTo-DscJson -Path 'idonotexist' | ConvertFrom-Json)
            $res.resources | Should -BeNullOrEmpty
        }
    }

    AfterAll {
        Remove-Module -Name $modLoad.Name -Force
    }
}
