# Installing the "Nightly" build of DSC CLI

> **Note**: Nightly builds contain the latest development features but may have bugs or breaking
> changes. Only install if you want to test unreleased functionality. If you encounter issues,
> please [open an issue](https://github.com/PowerShell/DSC/issues/new).

## Via script

> **Note**: This script requires the [GitHub CLI](https://cli.github.com/) to have already been
> installed.

### DSC CLI

This will install the latest nightly DSC CLI binary:

- **Windows**: `%LOCALAPPDATA%\dsc\dsc.exe`
- **Linux/macOS**: `~/.dsc/bin/dsc`

1. (macOS/Linux) Run the following:

   ```sh
   bash <(curl -Ls https://raw.githubusercontent.com/PowerShell/DSC/refs/heads/main/sharedScripts/install_cli_nightly.sh)
   ```

1. (Windows) Run the following in a PowerShell window:

   ```powershell
   iex "& { $(irm https://raw.githubusercontent.com/PowerShell/DSC/refs/heads/main/sharedScripts/install_cli_nightly.ps1) }"
   ```

1. Add the installation directory to your PATH environment variable to use `dsc` from any location.

## Manual

We are not currently publishing "nightly" releases, but you can grab the latest bits by viewing
the latest Action workflows for the `main` branch (or any other branch).

The easiest way to get these artifacts is through the GitHub site. Follow
[this link](https://github.com/PowerShell/DSC/actions/workflows/rust.yml?query=branch%3Amain+is%3Asuccess)
to view the latest successful Action workflows for the `main` branch. Select it to show the related
artifacts.

On the details page, select the artifact for your platform:

- `windows-bin` for Windows
- `linux-bin` for Linux
- `macos-bin` for macOS

Extract the archive and place the `dsc` executable (or `dsc.exe` on Windows) in a directory in
your PATH.

## Advanced script options

### DSC CLI

- macOS/Linux

   ```sh
   # install to a custom directory
   bash <(curl -Ls https://raw.githubusercontent.com/PowerShell/DSC/refs/heads/main/sharedScripts/install_cli_nightly.sh) --install-path /usr/local/bin

   # install from a fork repo
   bash <(curl -Ls https://raw.githubusercontent.com/PowerShell/DSC/refs/heads/main/sharedScripts/install_cli_nightly.sh) --repo myusername/DSC

   # install from a custom branch
   bash <(curl -Ls https://raw.githubusercontent.com/PowerShell/DSC/refs/heads/main/sharedScripts/install_cli_nightly.sh) --branch feature-branch

   # install from a specific github action run
   bash <(curl -Ls https://raw.githubusercontent.com/PowerShell/DSC/refs/heads/main/sharedScripts/install_cli_nightly.sh) --run-id 6146657618
   ```

- Windows

   ```powershell
   # install to a custom directory
   iex "& { $(irm https://raw.githubusercontent.com/PowerShell/DSC/refs/heads/main/sharedScripts/install_cli_nightly.ps1) } -InstallPath C:\tools\dsc"

   # install from a fork repo
   iex "& { $(irm https://raw.githubusercontent.com/PowerShell/DSC/refs/heads/main/sharedScripts/install_cli_nightly.ps1) } -Repo myusername/DSC"

   # install from a custom branch
   iex "& { $(irm https://raw.githubusercontent.com/PowerShell/DSC/refs/heads/main/sharedScripts/install_cli_nightly.ps1) } -Branch feature-branch"

   # install from a specific github action run
   iex "& { $(irm https://raw.githubusercontent.com/PowerShell/DSC/refs/heads/main/sharedScripts/install_cli_nightly.ps1) } -RunId 6146657618"
   ```
