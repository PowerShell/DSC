# Contributing to DSC

We welcome and appreciate contributions from the community.
There are many ways to become involved with DSC:
including filing issues,
joining in design conversations,
and writing and improving documentation.
Please read the rest of this document to ensure a smooth contribution process.

## Intro to Git and GitHub

* Make sure you have a [GitHub account](https://github.com/signup/free).
* Learning GitHub:
  * GitHub Help: [Good Resources for Learning Git and GitHub][https://help.github.com/articles/good-resources-for-learning-git-and-github/]
* [GitHub Flow Guide](https://guides.github.com/introduction/flow/):
  step-by-step instructions of GitHub Flow

## Contributing to Issues

* Check if the issue you are going to file already exists in our [GitHub issues](https://github.com/powershell/DSC/).
* If you can't find your issue already,
  [open a new issue](https://github.com/PowerShell/DSC/issues/new/choose),
  making sure to follow the directions as best you can.

## Contributing to Documentation

### Contributing to documentation related to DSC

You can contribute to documentation either in the `docs` folder of this repository
or in the [PowerShell-Docs-DSC](https://github.com/MicrosoftDocs/PowerShell-Docs-DSC/) repository.

> [!NOTE]
> Documentation contributed to the `docs` folder in this repository is periodically synced to the [PowerShell-Docs-DSC](https://github.com/MicrosoftDocs/PowerShell-Docs-DSC/) repository.

### Contributing to documentation related to maintaining or contributing to the DSC project

* When writing Markdown documentation, use [semantic linefeeds](https://rhodesmill.org/brandon/2012/one-sentence-per-line/).
  In most cases, it means "one clause/idea per line".
* Otherwise, these issues should be treated like any other issue in this repository.

#### Spellchecking documentation

Documentation is spellchecked. We use the
[textlint](https://github.com/textlint/textlint/wiki/Collection-of-textlint-rule) command-line tool,
which can be run in interactive mode to correct typos.

To run the spellchecker, follow these steps:

* install [Node.js](https://nodejs.org/en/) (v10 or up)
* install [textlint](https://github.com/textlint/textlint/wiki/Collection-of-textlint-rule) by
  `npm install -g textlint textlint-rule-terminology`
* run `textlint --rule terminology <changedFileName>`,
  adding `--fix` will accept all the recommendations.

If you need to add a term or disable checking part of a file see the [configuration sections of the rule](https://github.com/sapegin/textlint-rule-terminology).

#### Checking links in documentation

Documentation is link-checked. We make use of the
`markdown-link-check` command-line tool,
which can be run to see if any links are dead.

To run the link-checker, follow these steps:

* install [Node.js](https://nodejs.org/en/) (v10 or up)
* install `markdown-link-check` by
  `npm install -g markdown-link-check@3.8.5`
* run `find . \*.md -exec markdown-link-check {} \;`

## Contributing to Code

### Contributor License Agreement (CLA)

Contributions to the DSC repository requires signing the [Contributor License Agreement (CLA)][01].
You can manually sign the CLA at any time. When you submit a pull request to this repository, the
continuous integration will check whether you have already signed the CLA. If you haven't, the
automation will prompt you to do so as a comment on your PR.

We can't accept any contributions without the author signing the CLA. Signing the CLA is a one-time
requirement and applies to contributions to Microsoft open source repositories.

For more information, see [Contributor License Agreements][01].

[01]: https://cla.microsoft.com/

### Building and testing

#### Prerequisites

Building the DSC project requires PowerShell 7.2 or later to execute the build script. For more
information about installing PowerShell, see [Install PowerShell on Windows, Linux, and macOS][02].

DSC requires other tools for building the project. The build script (`build.new.ps1`) installs the
tools in the following table if they aren't already installed on your system:

| Tool                                                    |    Version    |       Platforms       | Projects                                             |
|:--------------------------------------------------------|:-------------:|:---------------------:|:-----------------------------------------------------|
| [Rust toolchain][03]                                    | Latest stable | Linux, macOS, Windows | All Rust crates                                      |
| [Visual Studio Build Tools for C++][04]                 | Latest stable |        Windows        | All Rust crates                                      |
| [`cargo audit`][05]                                     | Latest stable | Linux, macOS, Windows | All Rust crates                                      |
| [Clippy][06]                                            | Latest stable | Linux, macOS, Windows | All Rust crates                                      |
| [Tree-sitter CLI][07]                                   | Latest stable | Linux, macOS, Windows | All grammars                                         |
| [NodeJS][08]                                            |     `v24`     | Linux, macOS, Windows | All grammars                                         |
| [**Pester** PowerShell module][09]                      | Latest stable | Linux, macOS, Windows | All acceptance tests                                 |
| [**PSDesiredStateConfiguration** PowerShell module][10] |   `v2.0.7`    |        Windows        | `Microsoft.Windows/WindowsPowerShell` adapter  tests |

[02]: https://learn.microsoft.com/powershell/scripting/install/installing-powershell
[03]: https://rustup.rs/
[04]: https://learn.microsoft.com/cpp/windows/latest-supported-vc-redist
[05]: https://crates.io/crates/cargo-audit
[06]: https://doc.rust-lang.org/clippy/index.html
[07]: https://github.com/tree-sitter/tree-sitter/blob/master/crates/cli/README.md
[08]: https://nodejs.org/
[09]: https://pester.dev/
[10]: https://www.powershellgallery.com/packages/PSDesiredStateConfiguration/2.0.7

#### Quick start

```powershell
# Build the project
./build.new.ps1

# Build with linting (recommended)
./build.new.ps1 -Clippy

# Build and run all tests
./build.new.ps1 -Clippy -Test

# Build in release mode (optimized)
./build.new.ps1 -Release
```

#### Running tests

DSCv3 includes Rust unit tests and PowerShell Pester tests:

```powershell
# Run all tests
./build.new.ps1 -Test

# Run only Rust tests
./build.new.ps1 -Test -ExcludePesterTests

# Run only Pester tests
./build.new.ps1 -Test -ExcludeRustTests

# Run specific Pester test groups
./build.new.ps1 -SkipBuild -Test -ExcludeRustTests -PesterTestGroup dsc
```

Available Pester test groups include:

- `dsc` - Acceptance tests for the `dsc` executable.
- `adapters` - Acceptance tests for projects in the `adapters` folder.
- `extensions` - Acceptance tests for projects in the `extensions` folder.
- `resources` - Acceptance tests for projects in the `resources` folder.
- `grammars` - Acceptance tests for projects in the `grammars` folder.

#### Cross-platform builds

By default, the build script compiles code for the current platform and architecture. You can build
for other platforms and architectures with the `-Architecture` parameter:

```powershell
# Windows ARM
./build.new.ps1 -Architecture aarch64-pc-windows-msvc

# macOS Apple Silicon
./build.new.ps1 -Architecture aarch64-apple-darwin

# Linux x64
./build.new.ps1 -Architecture x86_64-unknown-linux-gnu
```

#### Additional Information

For detailed build instructions, troubleshooting, and CI/CD workflows, see the
[Build and Test Instructions](.github/instructions/instructions.md).

### Pull Request Guidelines

- Always create a pull request to the `main` branch of this repository.
- Avoid making big pull requests. Before you invest a large amount of time, file an issue and start
  a discussion with the community.
- Add a meaningful title to the PR describing what change you want to check in.
- When you create a pull request, include a summary about your changes in the PR description. If
  the changes are related to an existing GitHub issue, please reference the issue in the PR
  description (e.g. `Fix #123`).
- Please use the present tense and imperative mood when describing your changes:

  - Instead of `Adding support for new feature`, write `Add support for new feature`.
  - Instead of `Fixed bug in parser`, write `Fix bug in parser`.

- If your change adds a new source file, ensure the appropriate copyright and license headers are
  included at the top of the file:
  
  - For `.rs` files, use the copyright header with an empty line after it:

    ```rust
    // Copyright (c) Microsoft Corporation.
    // Licensed under the MIT License.

    ```

  - For `.ps1` and `.psm1` files, use the copyright header with an empty line after it:

    ```powershell
    # Copyright (c) Microsoft Corporation.
    # Licensed under the MIT License.

    ```

* Create and update relevant tests when making code changes.
* Run tests and ensure they're passing before opening a pull request.
* All pull requests **must** pass CI checks before they can be merged.

## Code of Conduct Enforcement

Reports of abuse will be reviewed by the PS-Committee and if it has been determined that violations of the
[Code of Conduct](CODE_OF_CONDUCT.md) has occurred, then a temporary ban may be imposed.
The duration of the temporary ban will depend on the impact and/or severity of the infraction.
This can vary from 1 day, a few days, a week, and up to 30 days.
Repeat offenses may result in a permanent ban from the PowerShell org.
