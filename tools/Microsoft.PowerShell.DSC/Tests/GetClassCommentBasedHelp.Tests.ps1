# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'GetClassCommentBasedHelp integration' {

    BeforeAll {
        $modulePath = Join-Path (Join-Path $PSScriptRoot '..') 'Microsoft.PowerShell.DSC.psd1'
        Import-Module $modulePath -Force

        $fixturesPath = Join-Path $PSScriptRoot 'Fixtures'
    }

    Context 'Single class with full comment-based help' {

        BeforeAll {
            $psd1 = Join-Path $fixturesPath 'HelpResource' 'HelpResource.psd1'
            $result = New-DscAdaptedResourceManifest -Path $psd1 -WarningVariable warnings -WarningAction SilentlyContinue
        }

        It 'Returns exactly one manifest object' {
            $result | Should -HaveCount 1
        }

        It 'Uses Synopsis as the manifest description' {
            $result.Description | Should -BeExactly 'Manages a help-documented resource.'
        }

        It 'Uses Synopsis as the schema description' {
            $result.ManifestSchema.Embedded['description'] | Should -BeExactly 'Manages a help-documented resource.'
        }

        It 'Sets the Name property description from .PARAMETER help' {
            $result.ManifestSchema.Embedded['properties']['Name']['description'] |
                Should -BeExactly 'The unique name identifying this resource instance.'
        }

        It 'Sets the Value property description from .PARAMETER help' {
            $result.ManifestSchema.Embedded['properties']['Value']['description'] |
                Should -BeExactly 'The value to assign to this resource.'
        }

        It 'Sets the Enabled property description from .PARAMETER help' {
            $result.ManifestSchema.Embedded['properties']['Enabled']['description'] |
                Should -BeExactly 'Whether this resource is active.'
        }

        It 'Does not emit any warnings about missing parameter documentation' {
            $warnings | Should -BeNullOrEmpty
        }
    }

    Context 'Single class with partial comment-based help (missing some .PARAMETER entries)' {

        BeforeAll {
            $psd1 = Join-Path $fixturesPath 'PartialHelpResource' 'PartialHelpResource.psd1'
            $result = New-DscAdaptedResourceManifest -Path $psd1 -WarningVariable warnings -WarningAction SilentlyContinue
        }

        It 'Returns exactly one manifest object' {
            $result | Should -HaveCount 1
        }

        It 'Uses Synopsis as the manifest description' {
            $result.Description | Should -BeExactly 'Manages a partially documented resource.'
        }

        It 'Sets the Name property description from .PARAMETER help' {
            $result.ManifestSchema.Embedded['properties']['Name']['description'] |
                Should -BeExactly 'The unique name for the resource.'
        }

        It 'Falls back to default description for undocumented Value property' {
            $result.ManifestSchema.Embedded['properties']['Value']['description'] |
                Should -BeExactly 'The Value property.'
        }

        It 'Falls back to default description for undocumented Count property' {
            $result.ManifestSchema.Embedded['properties']['Count']['description'] |
                Should -BeExactly 'The Count property.'
        }

        It 'Emits a warning about missing parameter documentation' {
            $warnings | Should -Not -BeNullOrEmpty
            $warnings[0] | Should -BeLike "*missing .PARAMETER documentation for: Value, Count"
        }
    }

    Context 'File with no comment-based help on class' {

        BeforeAll {
            $psd1 = Join-Path $fixturesPath 'SimpleResource' 'SimpleResource.psd1'
            $result = New-DscAdaptedResourceManifest -Path $psd1 -WarningVariable warnings -WarningAction SilentlyContinue
        }

        It 'Returns a manifest object' {
            $result | Should -HaveCount 1
        }

        It 'Falls back to module description' {
            $result.Description | Should -BeExactly 'A simple DSC resource for testing.'
        }

        It 'Uses default property descriptions' {
            $result.ManifestSchema.Embedded['properties']['Name']['description'] |
                Should -BeExactly 'The Name property.'
        }

        It 'Emits a warning about no comment-based help found' {
            $warnings | Should -Not -BeNullOrEmpty
            $warnings[0] | Should -BeLike "*No comment-based help found above class*"
        }
    }

    Context 'Two classes in one file - one with help, one without' {

        BeforeAll {
            $psd1 = Join-Path $fixturesPath 'MixedHelpResource' 'MixedHelpResource.psd1'
            $results = @(New-DscAdaptedResourceManifest -Path $psd1 -WarningVariable warnings -WarningAction SilentlyContinue)
        }

        It 'Returns two manifest objects' {
            $results | Should -HaveCount 2
        }

        Context 'DocumentedResource - has comment-based help' {

            BeforeAll {
                $documented = $results | Where-Object { $_.Type -eq 'MixedHelpResource/DocumentedResource' }
            }

            It 'Uses Synopsis as the manifest description' {
                $documented.Description | Should -BeExactly 'A fully documented DSC resource.'
            }

            It 'Sets the Name property description from .PARAMETER help' {
                $documented.ManifestSchema.Embedded['properties']['Name']['description'] |
                    Should -BeExactly 'The unique identifier for the resource.'
            }

            It 'Sets the Setting property description from .PARAMETER help' {
                $documented.ManifestSchema.Embedded['properties']['Setting']['description'] |
                    Should -BeExactly 'The configuration setting to apply.'
            }
        }

        Context 'UndocumentedResource - no comment-based help' {

            BeforeAll {
                $undocumented = $results | Where-Object { $_.Type -eq 'MixedHelpResource/UndocumentedResource' }
            }

            It 'Falls back to module description' {
                $undocumented.Description | Should -BeExactly 'Module with two classes, one with help and one without.'
            }

            It 'Uses default property description for Id' {
                $undocumented.ManifestSchema.Embedded['properties']['Id']['description'] |
                    Should -BeExactly 'The Id property.'
            }

            It 'Uses default property description for Data' {
                $undocumented.ManifestSchema.Embedded['properties']['Data']['description'] |
                    Should -BeExactly 'The Data property.'
            }
        }

        It 'Emits a warning for UndocumentedResource but not DocumentedResource' {
            $noHelpWarning = $warnings | Where-Object { $_ -like "*No comment-based help found above class 'UndocumentedResource'*" }
            $noHelpWarning | Should -Not -BeNullOrEmpty

            $documentedWarning = $warnings | Where-Object { $_ -like "*No comment-based help found above class 'DocumentedResource'*" }
            $documentedWarning | Should -BeNullOrEmpty
        }
    }

    Context 'Two classes in one file - both with comment-based help' {

        BeforeAll {
            $psd1 = Join-Path $fixturesPath 'BothHelpResource' 'BothHelpResource.psd1'
            $results = @(New-DscAdaptedResourceManifest -Path $psd1 -WarningVariable warnings -WarningAction SilentlyContinue)
        }

        It 'Returns two manifest objects' {
            $results | Should -HaveCount 2
        }

        Context 'FirstResource' {

            BeforeAll {
                $first = $results | Where-Object { $_.Type -eq 'BothHelpResource/FirstResource' }
            }

            It 'Uses Synopsis as the manifest description' {
                $first.Description | Should -BeExactly 'Manages the first resource.'
            }

            It 'Sets Name property description from help' {
                $first.ManifestSchema.Embedded['properties']['Name']['description'] |
                    Should -BeExactly 'The unique name of the first resource.'
            }

            It 'Sets Mode property description from help' {
                $first.ManifestSchema.Embedded['properties']['Mode']['description'] |
                    Should -BeExactly 'The operating mode for the first resource.'
            }
        }

        Context 'SecondResource' {

            BeforeAll {
                $second = $results | Where-Object { $_.Type -eq 'BothHelpResource/SecondResource' }
            }

            It 'Uses Synopsis as the manifest description' {
                $second.Description | Should -BeExactly 'Manages the second resource.'
            }

            It 'Sets Id property description from help' {
                $second.ManifestSchema.Embedded['properties']['Id']['description'] |
                    Should -BeExactly 'The identifier for the second resource.'
            }

            It 'Sets Label property description from help' {
                $second.ManifestSchema.Embedded['properties']['Label']['description'] |
                    Should -BeExactly 'A label for the second resource.'
            }
        }

        It 'Does not emit warnings about missing comment-based help' {
            $noHelpWarnings = $warnings | Where-Object { $_ -like "*No comment-based help found*" }
            $noHelpWarnings | Should -BeNullOrEmpty
        }

        It 'Does not emit warnings about missing parameter documentation' {
            $paramWarnings = $warnings | Where-Object { $_ -like "*missing .PARAMETER*" }
            $paramWarnings | Should -BeNullOrEmpty
        }
    }

    Context 'Existing tests still pass - SimpleResource without help retains correct schema' {

        BeforeAll {
            $psd1 = Join-Path $fixturesPath 'SimpleResource' 'SimpleResource.psd1'
            $result = New-DscAdaptedResourceManifest -Path $psd1 -WarningAction SilentlyContinue
        }

        It 'Schema still has correct $schema URI' {
            $result.ManifestSchema.Embedded['$schema'] | Should -BeExactly 'https://json-schema.org/draft/2020-12/schema'
        }

        It 'Schema still marks Key property Name as required' {
            $result.ManifestSchema.Embedded['required'] | Should -Contain 'Name'
        }

        It 'Schema still maps string properties correctly' {
            $result.ManifestSchema.Embedded['properties']['Name']['type'] | Should -BeExactly 'string'
        }

        It 'Schema still maps bool properties correctly' {
            $result.ManifestSchema.Embedded['properties']['Enabled']['type'] | Should -BeExactly 'boolean'
        }
    }
}
