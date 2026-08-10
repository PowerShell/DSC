# Built-in DSC extension reference

Each release of DSC includes built-in extensions that you can use immediately after you install
DSC. Extensions augment the functionality of DSC itself, such as discovering resources in
locations that DSC doesn't search by default. This document lists the available extensions and
links to the reference documentation for each.

## All built-in extensions

- [Microsoft.PowerShell/Discover](./Microsoft/PowerShell/Discover/index.md)
- [Microsoft.Windows.Appx/Discover](./Microsoft/Windows/Appx/Discover/index.md)

## Built-in discovery extensions

You can use the following built-in extensions to make DSC discover resource manifests in
locations other than the `PATH` and `DSC_RESOURCE_PATH` environment variables:

- [Microsoft.PowerShell/Discover](./Microsoft/PowerShell/Discover/index.md) - Discovers DSC
  resources packaged in PowerShell 7 modules.
- [Microsoft.Windows.Appx/Discover](./Microsoft/Windows/Appx/Discover/index.md) - Discovers DSC
  resources packaged as Appx packages on Windows.

## See also

- [dsc extension list][01]
- [DSC extension manifest schema reference][02]

<!-- Link reference definitions -->
[01]: ../cli/extension/list.md
[02]: ../schemas/extension/manifest/root.md
