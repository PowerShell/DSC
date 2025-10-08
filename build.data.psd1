@{
    PackageFiles = @{
        Linux = @(
            'bicep.dsc.extension.json',
            'dsc',
            'dsc_default.settings.json',
            'dsc.settings.json'
            'dscecho',
            'echo.dsc.resource.json',
            'assertion.dsc.resource.json',
            'apt.dsc.resource.json',
            'apt.dsc.resource.sh',
            'group.dsc.resource.json',
            'include.dsc.resource.json',
            'NOTICE.txt',
            'osinfo',
            'osinfo.dsc.resource.json',
            'powershell.dsc.resource.json',
            'psDscAdapter/',
            'psscript.ps1',
            'psscript.dsc.resource.json',
            'RunCommandOnSet.dsc.resource.json',
            'runcommandonset',
            'sshdconfig',
            'sshd_config.dsc.resource.json'
        )
        MacOS = @(
            'bicep.dsc.extension.json',
            'dsc',
            'dsc_default.settings.json',
            'dsc.settings.json'
            'dscecho',
            'echo.dsc.resource.json',
            'assertion.dsc.resource.json',
            'brew.dsc.resource.json',
            'brew.dsc.resource.sh',
            'group.dsc.resource.json',
            'include.dsc.resource.json',
            'NOTICE.txt',
            'osinfo',
            'osinfo.dsc.resource.json',
            'powershell.dsc.resource.json',
            'psDscAdapter/',
            'psscript.ps1',
            'psscript.dsc.resource.json',
            'RunCommandOnSet.dsc.resource.json',
            'runcommandonset',
            'sshdconfig',
            'sshd_config.dsc.resource.json'
        )
        Windows = @(
            'appx.dsc.extension.json',
            'appx-discover.ps1',
            'bicep.dsc.extension.json',
            'dsc.exe',
            'dsc_default.settings.json',
            'dsc.settings.json',
            'dscecho.exe',
            'echo.dsc.resource.json',
            'assertion.dsc.resource.json',
            'group.dsc.resource.json',
            'include.dsc.resource.json',
            'NOTICE.txt',
            'osinfo.exe',
            'osinfo.dsc.resource.json',
            'powershell.dsc.resource.json',
            'psDscAdapter/',
            'psscript.ps1',
            'psscript.dsc.resource.json',
            'winpsscript.dsc.resource.json',
            'reboot_pending.dsc.resource.json',
            'reboot_pending.resource.ps1',
            'registry.dsc.resource.json',
            'registry.exe',
            'RunCommandOnSet.dsc.resource.json',
            'RunCommandOnSet.exe',
            'sshdconfig.exe',
            'sshd-windows.dsc.resource.json',
            'sshd_config.dsc.resource.json',
            'windowspowershell.dsc.resource.json',
            'wmi.dsc.resource.json',
            'wmi.resource.ps1',
            'wmiAdapter.psd1',
            'wmiAdapter.psm1',
            'windows_baseline.dsc.yaml',
            'windows_inventory.dsc.yaml'
        )
        Executable = @(
            'apt.dsc.resource.sh',
            'brew.dsc.resource.sh'
        )
    }
    Projects = @(
        @{
            Name = 'root'
            RelativePath = '.'
            Kind = 'Root'
            CopyFiles = @{
                All = @(
                    'NOTICE.txt'
                )
            }
        }
        @{
            Name = 'resources/brew'
            RelativePath = 'resources/brew'
            Kind = 'Resource'
            SupportedPlatformOS = 'MacOS'
            CopyFiles = @{
                macOS = @(
                    'brew.dsc.resource.sh'
                    'brew.dsc.resource.json'
                )
            }
        }
        @{
            Name = 'resources/apt'
            RelativePath = 'resources/apt'
            Kind = 'Resource'
            SupportedPlatformOS = 'Linux'
            CopyFiles = @{
                Linux = @(
                    'apt.dsc.resource.sh'
                    'apt.dsc.resource.json'
                )
            }
        }
        @{
            Name = 'dsc-lib-pal'
            RelativePath = 'lib/dsc-lib-pal'
            Kind = 'Library'
            IsRust = $true
            SupportedPlatformOS = 'Windows'
        }
        @{
            Name = 'dsc-lib-registry'
            RelativePath = 'lib/dsc-lib-registry'
            Kind = 'Library'
            IsRust = $true
            SupportedPlatformOS = 'Windows'
        }
        @{
            Name = 'registry'
            RelativePath = 'resources/registry'
            Kind = 'Resource'
            IsRust = $true
            SupportedPlatformOS = 'Windows'
            Binaries = @('registry')
            CopyFiles = @{
                Windows = @(
                    'registry.dsc.resource.json'
                )
            }
        }
        @{
            Name = 'reboot_pending'
            RelativePath = 'resources/reboot_pending'
            Kind = 'Resource'
            SupportedPlatformOS = 'Windows'
            CopyFiles = @{
                Windows = @(
                    'reboot_pending.resource.ps1'
                    'reboot_pending.dsc.resource.json'
                )
            }
        }
        @{
            Name = 'wmi'
            RelativePath = 'adapters/wmi'
            Kind = 'Adapter'
            SupportedPlatformOS = 'Windows'
            CopyFIles = @{
                WIndows = @(
                    'wmi.resource.ps1'
                    'wmiAdapter.psd1'
                    'wmiAdapter.psm1'
                    'wmi.dsc.resource.json'
                )
            }
        }
        @{
            Name = 'configurations/windows'
            RelativePath = 'configurations/windows'
            Kind = 'Configuration'
            SupportedPlatformOS = 'Windows'
            CopyFiles = @{
                Windows = @(
                    'windows_baseline.dsc.yaml'
                    'windows_inventory.dsc.yaml'
                )
            }
        }
        @{
            Name = 'extensions/appx'
            RelativePath = 'extensions/appx'
            Kind = 'Extension'
            SupportedPlatformOS = 'Windows'
            CopyFiles = @{
                Windows = @(
                    'appx-discover.ps1'
                    'appx.dsc.extension.json'
                )
            }
        }
        @{
            Name = 'tree-sitter-dscexpression'
            RelativePath = 'grammars/tree-sitter-dscexpression'
            Kind = 'Grammar'
            IsRust = $true
            ClippyUnclean = $true
            SkipTest = @{
                Windows = $true
            }
        }
         @{
            Name = 'tree-sitter-ssh-server-config'
            RelativePath = 'grammars/tree-sitter-ssh-server-config'
            Kind = 'Grammar'
            IsRust = $true
            ClippyUnclean = $true
            # SKipTestProject = $IsWindows
        }
        @{
            Name = 'dsc-lib-security_context'
            RelativePath = 'lib/dsc-lib-security_context'
            Kind = 'Library'
            IsRust = $true
        }
        @{
            Name = 'dsc-lib-osinfo'
            RelativePath = 'lib/dsc-lib-osinfo'
            Kind = 'Library'
            IsRust = $true
        }
        @{
            Name = 'osinfo'
            RelativePath = 'resources/osinfo'
            Kind = 'Resource'
            IsRust = $true
            Binaries = @('osinfo')
            CopyFiles = @{
                All = @(
                    'osinfo.dsc.resource.json'
                )
            }
        }
        @{
            Name = 'dsc-lib'
            RelativePath = 'lib/dsc-lib'
            Kind = 'Library'
            IsRust = $true
        }
        @{
            Name = 'dsc'
            RelativePath = 'dsc'
            Kind = 'CLI'
            IsRust = $true
            Binaries = @('dsc')
            CopyFiles = @{
                All = @(
                    'dsc.settings.json'
                    'dsc_default.settings.json'
                    'assertion.dsc.resource.json'
                    'group.dsc.resource.json'
                    'include.dsc.resource.json'
                )
            }
        }
        @{
            Name = 'dscecho'
            RelativePath = 'resources/dscecho'
            Kind = 'Resource'
            IsRust = $true
            Binaries = @('dscecho')
            CopyFiles = @{
                All = @(
                    'echo.dsc.resource.json'
                )
            }
        }
        @{
            Name = 'bicep'
            RelativePath = 'extensions/bicep'
            Kind = 'Extension'
            CopyFIles = @{
                All = 'bicep.dsc.extension.json'
            }
        }
        @{
            Name = 'powershell-adapter'
            RelativePath = 'adapters/powershell'
            Kind = 'Adapter'
            CopyFiles = @{
                All = @(
                    'psDscAdapter/powershell.resource.ps1'
                    'psDscAdapter/psDscAdapter.psd1'
                    'psDscAdapter/psDscAdapter.psm1'
                    'powershell.dsc.resource.json'
                    )
                Windows = @(
                    'psDscAdapter/win_psDscAdapter.psd1'
                    'psDscAdapter/win_psDscAdapter.psm1'
                    'windowspowershell.dsc.resource.json'
                )
            }
        }
        @{
            Name = 'PSScript'
            RelativePath = 'resources/PSScript'
            Kind = 'Resource'
            CopyFiles = @{
                All = @(
                    'psscript.ps1'
                    'psscript.dsc.resource.json'
                )
                Windows = @(
                    'winpsscript.dsc.resource.json'
                )
            }
        }
        @{
            Name = 'process'
            RelativePath = 'resources/process'
            Kind = 'Resource'
            IsRust = $true
            Binaries = @('process')
            CopyFiles = @{
                All = @(
                    'process.dsc.resource.json'
                )
            }
        }
        @{
            Name = 'runcommandonset'
            RelativePath = 'resources/runcommandonset'
            Kind = 'Resource'
            IsRust = $true
            Binaries = @('runcommandonset')
            CopyFiles = @{
                All = @(
                    'RunCommandOnSet.dsc.resource.json'
                )
            }
        }
        @{
            Name = 'sshdconfig'
            RelativePath = 'resources/sshdconfig'
            Kind = 'Resource'
            IsRust = $true
            Binaries = @('sshdconfig')
            CopyFiles = @{
                All = @(
                    'sshd_config.dsc.resource.json'
                )
                Windows = @(
                    'sshd-windows.dsc.resource.json'
                )
            }
        }
        @{
            Name = 'dsctest'
            RelativePath = 'tools/dsctest'
            Kind = 'Resource'
            IsRust = $true
            TestOnly = $true
            Binaries = @('dsctest')
            CopyFiles = @{
                All = @(
                    'dscdelete.dsc.resource.json'
                    'dscexist.dsc.resource.json'
                    'dscexitcode.dsc.resource.json'
                    'dscexport.dsc.resource.json'
                    'dscexporter.dsc.resource.json'
                    'dscget.dsc.resource.json'
                    'dscindesiredstate.dsc.resource.json'
                    'dscoperation.dsc.resource.json'
                    'dscsleep.dsc.resource.json'
                    'dsctrace.dsc.resource.json'
                    'dscwhatif.dsc.resource.json'
                    'metadata.dsc.resource.json'
                    'resourceadapter.dsc.resource.json'
                    'version1.1.2.dsc.resource.json'
                    'version1.1.3.dsc.resource.json'
                    'version1.1.dsc.resource.json'
                    'version2.1p1.dsc.resource.json'
                    'version2.1p2.dsc.resource.json'
                    'version2.dsc.resource.json'
                )
            }
        }
        @{
            Name = 'test_group_resource'
            RelativePath = 'tools/test_group_resource'
            Kind = 'Resource'
            IsRust = $true
            TestOnly = $true
            Binaries = @('test_group_resource')
            CopyFIles = @{
                All = @(
                    'testGroup.dsc.resource.json'
                )
            }
        }
        @{
            Name = 'y2j'
            RelativePath = 'y2j'
            Kind = 'CLI'
            IsRust = $true
            Binaries = @('y2j')
        }
    )
}