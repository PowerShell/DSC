BeforeAll {
    class ReproResult {
        [object]$Result
        [object]$Trace
    }

    function Invoke-DscRepro {
        [CmdletBinding()]
        param([hashtable]$instance)

        begin {
            $dscArgs = @(
                '--trace-format', 'json'
                'resource'
                'get'
                '--resource', 'Dsc.Repro/Trace'
                '--input', ($instance | ConvertTo-Json -Compress)
            )
        }

        process {
            $emitted = dsc @dscArgs 2>&1 | ConvertFrom-Json

            [ReproResult]@{
                Result = $emitted[1]
                Trace  = $emitted[0]
            }
        }
    }
}

Describe 'Resource trace message behavior' {
    describe "simple messaging" {
        BeforeAll {
            $repro = Invoke-DscRepro -Instance @{
                case = 'simpleMessage'
            }
            $actualCase  = $repro.Result.actualState.case
            $emittedData = $repro.Result.actualState.emittedData
            $traceTimestamp = $repro.Trace.timestamp -as [datetime]
        }
        context "actual state" {
            it "returns the expected case for actual state" {
                $actualCase | Should -BeExactly 'simpleMessage'
            }
            context "emitted data" {
                it "returns 'warn' with the expected message" {
                    $emittedData.warn | Should -BeExactly 'Simple message'
                }
            }
        }
        
        context "DSC emitted trace message" {
            it "defines the message field" {
                $repro.Trace.fields.message | Should -Not -BeNullOrEmpty
            }
            it "defines the expected message with PID prefix" {
                $repro.Trace.fields.message | Should -Match '^PID \d+\:\s+Simple message'
            }
            it "doesn't define the trace_message field" {
                $repro.Trace.fields.trace_message | Should -BeNullOrEmpty
            }
            it 'bubbles up the expected timestamp' {
                $traceTimestamp.Date | Should -be (Get-Date).Date
            }
            it 'bubbles up the expected level' {
                $repro.Trace.level | Should -BeExactly 'WARN'
            }
            it "doesn't define the target field" {
                $repro.Trace.target | Should -BeNullOrEmpty
            }
            it "doesn't define the line_number field" {
                $repro.Trace.line_number | Should -BeNullOrEmpty
            }
        }
    }
    describe "Minimal struct" {
        BeforeAll {
            $repro = Invoke-DscRepro -Instance @{
                case = 'minimalStruct'
            }
            $actualCase  = $repro.Result.actualState.case
            $emittedData = $repro.Result.actualState.emittedData
            $expectedDate = (Get-Date).AddYears(-1).Date
        }
        context "actual state" {
            it "returns the expected case for actual state" {
                $actualCase | Should -Be 'minimalStruct'
            }
            context "emitted data" {
                it "defines 'fields.message'" {
                    $emittedData.fields.message | Should -BeExactly 'structured trace message'
                }
                it "defines 'timestamp' for one year ago" {
                    ($emittedData.timestamp -as [datetime]).Date | Should -Be $expectedDate
                }
                it "defines 'level' as 'WARN'" {
                    $emittedData.level | Should -BeExactly 'WARN'
                }
            }
        }
        context "DSC emitted trace message" {
            it "defines the message field" {
                $repro.Trace.fields.message | Should -Not -BeNullOrEmpty
            }
            it "defines the correct message" {
                $repro.Trace.fields.message | Should -match 'structured trace message$'
            }
            it "doesn't define the trace_message field" {
                $repro.Trace.fields.trace_message | Should -BeNullOrEmpty
            }
            it 'bubbles up the expected timestamp' {
                ($repro.Trace.timestamp -as [datetime]).Date | Should -Be $expectedDate
            }
            it 'bubbles up the expected level' {
                $repro.Trace.level | Should -BeExactly 'WARN'
            }
            it "doesn't define the target field" {
                $repro.Trace.target | Should -BeNullOrEmpty
            }
            it "doesn't define the line_number field" {
                $repro.Trace.line_number | Should -BeNullOrEmpty
            }
        }
    }
    describe "Struct with metadata" {
        BeforeAll {
            $repro = Invoke-DscRepro -Instance @{
                case = 'structWithMetadata'
            }
            $actualCase   = $repro.Result.actualState.case
            $emittedData  = $repro.Result.actualState.emittedData
            $expectedDate = (Get-Date).AddYears(-1).Date
            $scriptFile   = Get-Command repro.trace.resource.ps1 | Select-Object -ExpandProperty Source
        }
        context "actual state" {
            it "returns the expected case for actual state" {
                $actualCase | Should -Be 'structWithMetadata'
            }
            context "emitted data" {
                it "defines 'fields.message'" {
                    $emittedData.fields.message | Should -BeExactly 'structured trace message'
                }
                it "defines 'timestamp' for one year ago" {
                    ($emittedData.timestamp -as [datetime]).Date | Should -Be $expectedDate
                }
                it "defines 'level' as 'WARN'" {
                    $emittedData.level | Should -BeExactly 'WARN'
                }
                it "defines 'target' as the path to the script file" {
                    $emittedData.target | Should -Be $scriptFile
                }
                it "defines 'lineNumber' as 94" {
                    $emittedData.lineNumber | Should -be 94
                }
            }
        }
        context "DSC emitted trace message" {
            it "defines the message field" {
                $repro.Trace.fields.message | Should -Not -BeNullOrEmpty
            }
            it "defines the correct message" {
                $repro.Trace.fields.message | Should -match 'structured trace message$'
            }
            it "doesn't define the trace_message field" {
                $repro.Trace.fields.trace_message | Should -BeNullOrEmpty
            }
            it 'bubbles up the expected timestamp' {
                ($repro.Trace.timestamp -as [datetime]).Date | Should -Be $expectedDate
            }
            it 'bubbles up the expected level' {
                $repro.Trace.level | Should -BeExactly 'WARN'
            }
            it "defines the target field" {
                $repro.Trace.target | Should -Not -BeNullOrEmpty
            }
            it "bubbles up the target field as expected" {
                $repro.Trace.target | Should -Be $scriptFile
            }
            it "defines the line_number field" {
                $repro.Trace.line_number | Should -Not -BeNullOrEmpty
            }
            it "bubbles up the line number as expected" {
                $repro.Trace.line_number | Should -Be 94
            }
        }
    }
    describe "Struct with additional fields" {
        BeforeAll {
            $repro = Invoke-DscRepro -Instance @{
                case = 'structWithAdditionalFields'
            }
            $actualCase  = $repro.Result.actualState.case
            $emittedData = $repro.Result.actualState.emittedData
            $expectedDate = (Get-Date).AddYears(-1).Date
        }
        context "actual state" {
            it "returns the expected case for actual state" {
                $actualCase | Should -Be 'structWithAdditionalFields'
            }
            context "emitted data" {
                it "defines 'fields.message'" {
                    $emittedData.fields.message | Should -BeExactly 'structured trace message'
                }
                it "defines 'fields.extraInteger" {
                    $emittedData.fields.extraInteger | Should -Be 10
                }
                it "defines 'fields.extraString" {
                    $emittedData.fields.message | Should -BeExactly 'additional data'
                }
                it "defines 'timestamp' for one year ago" {
                    ($emittedData.timestamp -as [datetime]).Date | Should -Be $expectedDate
                }
                it "defines 'level' as 'WARN'" {
                    $emittedData.level | Should -BeExactly 'WARN'
                }
            }
        }
        context "DSC emitted trace message" {
            it "defines the 'message' field" {
                $repro.Trace.fields.message | Should -Not -BeNullOrEmpty
            }
            it "bubbles up the correct message" {
                $repro.Trace.fields.message | Should -match 'structured trace message$'
            }
            it "defines the 'extraInteger' field" {
                $repro.Trace.fields.extraInteger | Should -Not -BeNullOrEmpty
            }
            it "bubbles up the correct value for 'extraInteger'" {
                $repro.Trace.fields.extraInteger | Should -Be 10
            }
            it "defines the 'extraString' field" {
                $repro.Trace.fields.extraString | Should -Not -BeNullOrEmpty
            }
            it "bubbles up the correct value  for 'extraString'" {
                $repro.Trace.fields.extraString | Should -BeExactly 'additional data'
            }
            it "doesn't define the trace_message field" {
                $repro.Trace.fields.trace_message | Should -BeNullOrEmpty
            }
            it 'bubbles up the expected timestamp' {
                ($repro.Trace.timestamp -as [datetime]).Date | Should -Be $expectedDate
            }
            it 'bubbles up the expected level' {
                $repro.Trace.level | Should -BeExactly 'WARN'
            }
            it "doesn't define the target field" {
                $repro.Trace.target | Should -BeNullOrEmpty
            }
            it "doesn't define the line_number field" {
                $repro.Trace.line_number | Should -BeNullOrEmpty
            }
        }
    }

    describe "Struct with metadata and additional fields" {
        BeforeAll {
            $repro = Invoke-DscRepro -Instance @{
                case = 'structWithMetadataAndAdditionalFields'
            }
            $actualCase  = $repro.Result.actualState.case
            $emittedData = $repro.Result.actualState.emittedData
            $expectedDate = (Get-Date).AddYears(-1).Date
            $scriptFile   = Get-Command repro.trace.resource.ps1 | Select-Object -ExpandProperty Source
        }
        context "actual state" {
            it "returns the expected case for actual state" {
                $actualCase | Should -Be 'structWithMetadataAndAdditionalFields'
            }
            context "emitted data" {
                it "defines 'fields.message'" {
                    $emittedData.fields.message | Should -BeExactly 'structured trace message'
                }
                it "defines 'fields.extraInteger" {
                    $emittedData.fields.extraInteger | Should -Be 10
                }
                it "defines 'fields.extraString" {
                    $emittedData.fields.extraString | Should -BeExactly 'additional data'
                }
                it "defines 'timestamp' for one year ago" {
                    ($emittedData.timestamp -as [datetime]).Date | Should -Be $expectedDate
                }
                it "defines 'level' as 'WARN'" {
                    $emittedData.level | Should -BeExactly 'WARN'
                }
                it "defines 'target' as the path to the script file" {
                    $emittedData.target | Should -Be $scriptFile
                }
                it "defines 'lineNumber' as 94" {
                    $emittedData.lineNumber | Should -be 94
                }
            }
        }
        context "DSC emitted trace message" {
            it "defines the 'message' field" {
                $repro.Trace.fields.message | Should -Not -BeNullOrEmpty
            }
            it "bubbles up the correct message" {
                $repro.Trace.fields.message | Should -match 'structured trace message$'
            }
            it "defines the 'extraInteger' field" {
                $repro.Trace.fields.extraInteger | Should -Not -BeNullOrEmpty
            }
            it "bubbles up the correct value for 'extraInteger'" {
                $repro.Trace.fields.extraInteger | Should -Be 10
            }
            it "defines the 'extraString' field" {
                $repro.Trace.fields.extraString | Should -Not -BeNullOrEmpty
            }
            it "bubbles up the correct value  for 'extraString'" {
                $repro.Trace.fields.extraString | Should -BeExactly 'additional data'
            }
            it "doesn't define the trace_message field" {
                $repro.Trace.fields.trace_message | Should -BeNullOrEmpty
            }
            it 'bubbles up the expected timestamp' {
                ($repro.Trace.timestamp -as [datetime]).Date | Should -Be $expectedDate
            }
            it "defines the target field" {
                $repro.Trace.target | Should -Not -BeNullOrEmpty
            }
            it "bubbles up the target field as expected" {
                $repro.Trace.target | Should -Be $scriptFile
            }
            it "defines the line_number field" {
                $repro.Trace.line_number | Should -Not -BeNullOrEmpty
            }
            it "bubbles up the line number as expected" {
                $repro.Trace.line_number | Should -Be 94
            }
        }
    }
}