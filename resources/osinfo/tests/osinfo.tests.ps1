# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'osinfo resource tests' {
    It 'should get osinfo' {
        $out = dsc resource get -r Microsoft/osInfo | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        if ($IsWindows) {
            $out.actualState.family | Should -BeExactly 'Windows'
        }
        elseif ($IsLinux) {
            $out.actualState.family | Should -BeExactly 'Linux'
        }
        elseif ($IsMacOS) {
            $out.actualState.family | Should -BeExactly 'macOS'
        }

        $out.actualState.version | Should -Not -BeNullOrEmpty
        if ([Environment]::Is64BitProcess) {
            $out.actualState.bitness | Should -BeExactly '64'
        }
        else {
            $out.actualState.bitness | Should -BeExactly '32'
        }

        $out._name | Should -BeNullOrEmpty
    }

    It 'should perform synthetic test' {
        if ($IsWindows) {
            $invalid = 'Linux'
        }
        else {
            $invalid = 'Windows'
        }
        $out = "{`"family`": `"$invalid`"}" | dsc resource test -r 'Microsoft/OSInfo' -f - | ConvertFrom-Json
        $actual = dsc resource get -r Microsoft/OSInfo | ConvertFrom-Json
        $out.actualState.family | Should -BeExactly $actual.actualState.family
        $out.actualState.version | Should -BeExactly $actual.actualState.version
        $out.actualState.bitness | Should -BeExactly $actual.actualState.bitness
        $out.actualState.edition | Should -BeExactly $actual.actualState.edition
        $out.differingproperties | Should -Be @('family')
    }

    It 'should support export' {
        $out = dsc resource export -r Microsoft/OSInfo | ConvertFrom-Json
        $out.'$schema' | Should -BeExactly 'https://aka.ms/dsc/schemas/v3/bundled/config/document.json'
        if ($IsWindows) {
            $out.resources[0].properties.family | Should -BeExactly 'Windows'
        }
        elseif ($IsLinux) {
            $out.resources[0].properties.family | Should -BeExactly 'Linux'
        }
        elseif ($IsMacOS) {
            $out.resources[0].properties.family | Should -BeExactly 'macOS'
        }
        $out.resources[0].name | Should -BeExactly "$($out.resources[0].properties.family) $($out.resources[0].properties.version) $($out.resources[0].properties.architecture)"
    }
}

Describe 'osinfo test subcommand version operator tests' {
    BeforeDiscovery {
        $osGetResult = dsc resource get -r Microsoft/OSInfo | ConvertFrom-Json
        $currentVersion = $osGetResult.actualState.version

        $versionTestCases = @(
            @{ constraint = $currentVersion;          expectedState = $true;  description = 'exact version without operator' }
            @{ constraint = "= $currentVersion";      expectedState = $true;  description = 'exact version with = operator' }
            @{ constraint = ">= $currentVersion";     expectedState = $true;  description = '>= current version' }
            @{ constraint = "<= $currentVersion";     expectedState = $true;  description = '<= current version' }
            @{ constraint = "> $currentVersion";      expectedState = $false; description = '> current version' }
            @{ constraint = "< $currentVersion";      expectedState = $false; description = '< current version' }
        )

        $invalidSyntaxCases = @(
            @{ constraint = '?? 1.0';  description = 'unknown ?? operator treated as exact match' }
            @{ constraint = '~= 1.0';  description = 'unsupported ~= operator treated as exact match' }
            @{ constraint = '>> 1.0';  description = 'unsupported >> operator treated as exact match' }
        )
    }

    Context 'valid version constraints' {
        It 'version constraint "<constraint>" (<description>) should report inDesiredState = <expectedState>' -ForEach $versionTestCases {
            $json = "{`"version`": `"$constraint`"}"
            $out = $json | dsc resource test -r Microsoft/OSInfo -f - | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0
            $out.inDesiredState | Should -Be $expectedState
        }
    }

    Context 'unsupported version syntax' {
        It 'version "<constraint>" (<description>) should not be in desired state' -ForEach $invalidSyntaxCases {
            $json = "{`"version`": `"$constraint`"}"
            $out = $json | dsc resource test -r Microsoft/OSInfo -f - | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0
            $out.inDesiredState | Should -Be $false
        }
    }
}

Describe 'osinfo test subcommand property tests' {
    BeforeAll {
        $actual = (dsc resource get -r Microsoft/OSInfo | ConvertFrom-Json).actualState
    }

    Context 'edition property' -Skip:(-not $IsWindows) {
        It 'should be in desired state when edition matches actual' {
            $json = @{ edition = $actual.edition } | ConvertTo-Json -Compress
            $out = $json | dsc resource test -r Microsoft/OSInfo -f - | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0
            $out.inDesiredState | Should -BeTrue
        }

        It 'should not be in desired state when edition does not match' {
            $json = @{ edition = 'NonExistentEdition' } | ConvertTo-Json -Compress
            $out = $json | dsc resource test -r Microsoft/OSInfo -f - | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0
            $out.inDesiredState | Should -BeFalse
        }
    }

    Context 'codename property' -Skip:(-not $IsLinux) {
        It 'should be in desired state when codename matches actual' {
            $json = @{ codename = $actual.codename } | ConvertTo-Json -Compress
            $out = $json | dsc resource test -r Microsoft/OSInfo -f - | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0
            $out.inDesiredState | Should -BeTrue
        }

        It 'should not be in desired state when codename does not match' {
            $json = @{ codename = 'nonexistentcodename' } | ConvertTo-Json -Compress
            $out = $json | dsc resource test -r Microsoft/OSInfo -f - | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0
            $out.inDesiredState | Should -BeFalse
        }
    }

    Context 'bitness property' {
        It 'should be in desired state when bitness matches actual' {
            $json = @{ bitness = [int]$actual.bitness } | ConvertTo-Json -Compress
            $out = $json | dsc resource test -r Microsoft/OSInfo -f - | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0
            $out.inDesiredState | Should -BeTrue
        }

        It 'should not be in desired state when bitness does not match' {
            $wrongBitness = if ([int]$actual.bitness -eq 64) { 32 } else { 64 }
            $json = @{ bitness = $wrongBitness } | ConvertTo-Json -Compress
            $out = $json | dsc resource test -r Microsoft/OSInfo -f - | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0
            $out.inDesiredState | Should -BeFalse
        }
    }

    Context 'architecture property' {
        It 'should be in desired state when architecture matches actual' {
            $json = @{ architecture = $actual.architecture } | ConvertTo-Json -Compress
            $out = $json | dsc resource test -r Microsoft/OSInfo -f - | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0
            $out.inDesiredState | Should -BeTrue
        }

        It 'should not be in desired state when architecture does not match' {
            $json = @{ architecture = 'mips' } | ConvertTo-Json -Compress
            $out = $json | dsc resource test -r Microsoft/OSInfo -f - | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0
            $out.inDesiredState | Should -BeFalse
        }
    }

    Context 'multiple properties combined' {
        It 'should be in desired state when all specified properties match' {
            $desiredState = @{ family = $actual.family; bitness = [int]$actual.bitness; architecture = $actual.architecture }
            $json = $desiredState | ConvertTo-Json -Compress
            $out = $json | dsc resource test -r Microsoft/OSInfo -f - | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0
            $out.inDesiredState | Should -BeTrue
        }

        It 'should not be in desired state when one of multiple properties does not match' {
            $desiredState = @{ family = $actual.family; bitness = [int]$actual.bitness; architecture = 'mips' }
            $json = $desiredState | ConvertTo-Json -Compress
            $out = $json | dsc resource test -r Microsoft/OSInfo -f - | ConvertFrom-Json
            $LASTEXITCODE | Should -Be 0
            $out.inDesiredState | Should -BeFalse
        }
    }
}
