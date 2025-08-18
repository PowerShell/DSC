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

## Issues for new contributors

* Check if the issue you are going to file already exists in our [GitHub issues](https://github.com/powershell/DSC/).
* If you can't find your issue already,
  [open a new issue](https://github.com/PowerShell/DSC/issues/new/choose),
  making sure to follow the directions as best you can.

## Contribution Workflow

Whether you’re fixing bugs, developing new functionality, or improving tests,
contributions are managed through a GitHub-centered workflow.
After setting up your GitHub account, follow these instructions:

1. Fork the repo you plan to work on.

  In GitHub, click Fork on the project page to create your copy under your account.

1. Clone your fork locally

  ```powershell
  git clone https://github.com/<your-github-username>/DSC.git
  cd DSC
  ```

1. Add the upstream remote and sync main

  ```powershell
  git remote add upstream https://github.com/PowerShell/DSC.git
  git fetch upstream
  git checkout main
  git pull --ff-only upstream main
  ```

1. Create a new branch to hold your work.

  ```powershell
  git checkout -b new-branch-name
  ```

1. Work on your new code. Write tests

  ```powershell
  .\build.ps1 -UseCratesIO -Test
  ```

1. Commit your changes with a clear message

  ```powershell
  git add -A
  git commit -m "<concise summary of the change>"
  ```

1. Push your branch and open a Pull Request

  ```powershell
  git push -u origin feature/<short-description>
  ```

  In GitHub, open a PR from your branch to the upstream main branch. Link related issues
  and include a brief summary and rationale.

1. Keep your branch up to date

  ```powershell
  git fetch upstream
  git rebase upstream/main
  # resolve conflicts if any, then continue the rebase
  git push --force-with-lease
  ```

> [!NOTE]
> `-UseCratesIO` ensures the build uses crates.io rather than internal replaced sources.

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

## Code of Conduct Enforcement

Reports of abuse will be reviewed by the PS-Committee and if it has been determined that violations of the
[Code of Conduct](CODE_OF_CONDUCT.md) has occurred, then a temporary ban may be imposed.
The duration of the temporary ban will depend on the impact and/or severity of the infraction.
This can vary from 1 day, a few days, a week, and up to 30 days.
Repeat offenses may result in a permanent ban from the PowerShell org.
