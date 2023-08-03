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

* Check if the issue you are going to file already exists in our [GitHub issues](https://github.com/powershell/powershell-docs-DSC/).
* If you can't find your issue already,
  [open a new issue](https://github.com/PowerShell/DSC/issues/new/choose),
  making sure to follow the directions as best you can.

## Contributing to Documentation

### Contributing to documentation related to DSC

Please see the [PowerShell-Docs-DSC](https://github.com/MicrosoftDocs/PowerShell-Docs-DSC/) repository for details.

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
Code of Conduct has occurred, then a temporary ban may be imposed.
The duration of the temporary ban will depend on the impact and/or severity of the infraction.
This can vary from 1 day, a few days, a week, and up to 30 days.
Repeat offenses may result in a permanent ban from the PowerShell org.
