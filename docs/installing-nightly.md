# Installing the "Nightly" build of DSC CLI

> [!NOTE]
> Nightly builds contain the latest development features but may have bugs or breaking changes.
> Only install if you want to test unreleased functionality. If you encounter issues, please
> [open an issue](https://github.com/PowerShell/DSC/issues/new).

## Using a script

> [!NOTE]
> This script requires the [GitHub CLI](https://cli.github.com/) to have already been installed.

### DSC CLI

This will install the latest nightly DSC CLI binary to the following location depending on your
platform:

- **Windows**: `%LOCALAPPDATA%\dsc\dsc.exe`
- **Linux/macOS**: `~/.dsc/bin/dsc`

1. Retrieve the appropriate script. You can use the PowerShell script for Windows and the Bash script for Linux and macOS.

   - Bash (Linux and macOS):

     ```sh
     curl -o https://raw.githubusercontent.com/PowerShell/DSC/refs/heads/main/sharedScripts/install_cli_nightly.sh
     ```

   - PowerShell (Windows):

     ```powershell
     Invoke-WebRequest -Uri https://raw.githubusercontent.com/PowerShell/DSC/refs/heads/main/sharedScripts/install_cli_nightly.ps1 -OutFile install_cli_nightly.ps1
     ```

1. Review the script before invoking it.
1. Invoke the script to install `dsc`:

   - Bash (Linux and macOS):

     ```sh
     ./install_cli_nightly.sh
     ```

   - PowerShell (Windows):

     ```powershell
     ./install_cli_nightly.ps1
     ```

1. Add the installation directory to your `PATH` environment variable.

## Manual

We don't currently publish nightly releases, but you can get the latest builds by viewing the most
recent Action workflows for the `main` branch.

The easiest way to get these artifacts is through the GitHub site. The following link directs you
to the latest successful Action workflows for merges to the `main` branch:

<https://github.com/PowerShell/DSC/actions?query=workflow%3ARust+is%3Asuccess+branch%3Amain+event%3Apush>

Select the first link in the list to show the related artifacts. At the bottom of the page,
download the artifact by selecting the artifact for your platform:

- `windows-bin` for Windows
- `linux-bin` for Linux
- `macos-bin` for macOS

Extract the archive using the following steps:

- The artifact will be downloaded as `<platform>-bin.zip`. You can invoke the following commands to
  extract the contents into a folder called `dsc` in the current directory:

  - Bash (Linux, macOS)

    ```sh
    # Be sure to set this to the appropriate platform: 'linux' or 'macos'
    platform="linux"
    artifact="$platform-bin.zip"
    # requires `unzip`, install if needed to expand the zip file
    unzip $artifact
    # Expand the tar file
    tar -xvf bin.tar
    # Move the subfolder containing the binaries and manifests
    mv ./bin/debug dsc
    ```

  - PowerShell (Linux, macOS, Windows):

    ```powershell
    # Be sure to set this to the appropriate platform: 'linux', 'macos', or 'windows'
    $platform = 'linux'
    $artifact = "$platform-bin.zip"
    # Expand the zip file
    Expand-Archive -Path $artifact -DestinationPath .
    # Expand the tar file
    tar -xvf bin.tar
    # Move the subfolder containing the binaries and manifests
    Move-Item -Path ./bin/debug -Destination dsc
    ```

- Optionally, you can clean up the downloaded archive and intermediary files and folders:

  - Bash (Linux, macOS)

    ```sh
    rm -rf ./bin ./bin.tar ./linux-bin.zip
    ```

  - PowerShell (Linux, macOS, Windows)

    ```powershell
    Remove-Item -Path ./$artifact, ./bin.tar, ./bin -Recurse -Force
    ```

- Finally, make sure to add the folder containing the extracted binaries and resource manifests to
  your `PATH` environmental variable

## Advanced script options

### CLI installation options

- Install to a custom directory:

  - Bash (Linux, macOS):

    ```bash
    install_cli_nightly.sh --install-path /usr/local/bin
    ```

  - PowerShell (Windows):

    ```powershell
    install_cli_nightly.ps1 -InstallPath C:\tools\dsc
    ```

- Install from a forking repository:

  - Bash (Linux, macOS):

    ```bash
    install_cli_nightly.sh --repo myusername/DSC
    ```

  - PowerShell (Windows):

    ```powershell
    install_cli_nightly.ps1 -Repo myusername/DSC"
    ```

- Install from a branch other than `main`:

  - Bash (Linux, macOS):

    ```bash
    install_cli_nightly.sh --branch feature-branch
    ```

  - PowerShell (Windows):

    ```powershell
    install_cli_nightly.ps1 -Branch feature-branch"
    ```

- Install from a specific GitHub Action run:

  - Bash (Linux, macOS):

    ```bash
    install_cli_nightly.sh --run-id 6146657618
    ```

  - PowerShell (Windows):

    ```powershell
    install_cli_nightly.ps1 -RunId 6146657618"
    ```
