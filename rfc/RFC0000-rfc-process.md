---
RFC:          RFC0000
Author:       @michaeltlombardi
Sponsor:      @michaeltlombardi
Status:       Draft
SupersededBy: null
Version:      1.0
Area:         Process
CommentsDue:  2025-07-06
---

# DSC RFC Process and Guidelines

A DSC RFC (Request for Comments) is a publication to propose design changes and improvements to
DSC. This provides the community an opportunity to provide feedback before code is written where it
becomes harder to change at the risk of compatibility.

This process was adapted from the [PowerShell RFC process][01].

## Roles

- **Author**: All members of the community are allowed to author new RFCs and can provide feedback
  to any RFC.
- **Working Group (WG)**: A group responsible for deciding whether or not an issue in the
  repository requires a proposal and for providing feedback within an RFC proposal and votes to
  accept or reject an RFC.

  For more information about Working Groups, see [Working Groups][02].
- **Sponsor**: A person who commits to implementing an RFC if the WG accepts it. The sponsor may be
  the Author, a WG member, or any member of the community. The WG won't accept an RFC proposal
  without a Sponsor.

## When to submit an RFC

Generally, you should submit proposals as [issues in the DSC repository][03]. When reviewing
issues, the WG may request an RFC. The issue author, a WG member, or any other memberof the
community may then draft and submit an RFC for the issue.

WG members may also submit an RFC for proposals arising from WG discussions, knowing that the
proposal is complex enough to warrant a full RFC.

## Submitting an RFC

When submitting an RFC, the Author shall:

- Create a file named `RFCNNNN-<Title>.md` in the `rfc/drafts` folder.

  The Author _shall not_ assign the RFC number. The author shall leave the `NNNN` in the
  filename. Example: `RFCNNNN-docs-extension.md`

  The file must use the [RFC template][04]. The Author must fill out the following fields in the
  template frontmatter:

  - `Author` - Set to your GitHub username prefixed with `@`, like `@MyUserName`.
  - `Status` - Set to `Draft`.
  - `Version` - Set to `1.0`.
  - `CommentsDue` - Set to a date at least one month from the date you intend to submit the PR. Use
    [ISO8601][05] format, like `2022-09-27` for September 27, 2022.

  Set the `Sponsor` field to your GitHub username if you're willing to implement the RFC, assuming
  the WG accepts it.

  Define the H1 title. Fill out each section of the template as directed by the template comments.
- Include any additional files, such as code samples, in the `rfc/draft/RFCNNNN/` folder.
- Check `Allow edits from maintainers` option when submitting the PR so that the WG can add the RFC
  number to the draft, update the title, and fix the filename.
- Submit the PR as a [draft PR][06].
- Use the prefix `RFC:` for the PR title.

## Sponsoring an RFC

At any time while the RFC is in the [Draft](#draft) or [Reviewing](#reviewing) state, the Author, a
WG member, or community member may choose to sponsor the RFC by committing to implement it, if
accepted.

When a Sponsor commits to the RFC, a WG member will:

1. Apply the `RFC - Sponsored` label to the PR.
1. Assign the PR to the Sponsor.
1. Set the `Sponsor` field of the draft RFC to the GitHub username of the Sponsor.

The WG will only accept RFCs with a Sponsor.

## Commenting on an RFC

When providing feedback or otherwise commenting on an RFC proposal, focus your feedback and
discussion on the proposed experience and functionality. The WG may close conversations that are
distracting from the core purpose of the RFC, such as bikeshedding around names for proposed APIs.

As always, you must adhere to the [Code of Conduct][coc] when participating in discussions in the
DSC repository.

[coc]: ../CODE_OF_CONDUCT.md

## RFC Status

An RFC may be in any of the following states:

- [Draft](#draft)
- [Reviewing](#reviewing)
- [Accepted](#accepted)
- [Experimental](#experimental)
- [Rejected](#rejected)
- [Withdrawn](#withdrawn)
- [Final](#final)

### Draft

When an RFC is initially submitted as a draft PR, it's in the `Draft` state. RFCs remain in this
state until the Author marks the PR as ready for review.

While an RFC is in this state, we encourage contributors and WG members to read, discuss, and
comment on the RFC. Discussion and iteration during the drafting stage provides information and
context for the WG during the reviewing stage.

After one month, the Author may mark their PR as ready for formal review, taking it out of draft. A
WG member will then apply the `RFC - Reviewing` label to the PR.

### Reviewing

After the author marks their PR as ready for review, the RFC moves into the formal review state.

While the RFC is in review, the WG members are responsible for providing comments and feedback on
the PR. The WG meets at least once a month, during which they will review open RFCs. The WG will
always indicate to the Author a date by which the Author can expect feedback. The WG is responsible
for communicating with the Author during the review process to negotiate timelines for addressing
feedback and for updating the Author on the review status of their proposal.

The RFC remains in this state until one of the following conditions is met:

- The WG decides to reject the RFC, changing the state to [Rejected](#rejected).
- The RFC has a [Sponsor](#sponsoring-an-rfc) and the WG requests an experimental implementation,
  changing the state to [Experimental](#experimental).
- The RFC has a [Sponsor](#sponsoring-an-rfc) and the WG decides to accept the RFC as-is, changing
  the state to [Accepted](#accepted).
- The Author decides to withdraw their RFC, changing the state to [Withdrawn](#withdrawn).

### Rejected

If the WG decides not to proceed with the RFC, a WG member shall:

1. Add a comment to the PR indicating the rationale for rejecting the RFC.
1. Add the `RFC - Rejected` label to denote that the WG rejected the RFC.
1. Close the PR instead of merging it.
1. Update the [RFC History][07] table to reflect the changed status.

In the future, this can be done automatically with GitHub Actions.

### Experimental

Experimental implementations are used to provide a working example of proposed designs in order for
the WG and other users to understand real-world usage of the proposal.

If the WG decides to request an experimental implementation, a WG member shall:

1. Ensure the `Status` in the frontmatter of the RFC document is set to `Experimental`.
1. Apply the label `RFC - Experimental` to the PR.
1. Update the [RFC History][07] table to reflect the changed status.
1. Merge the PR.

The Author may be asked to continue to update the RFC as the usage of the experimental feature
drives new insight into how the feature should be designed.

When the WG is satisfied with the experimental implementation, a WG member will start
the process to finalize the RFC, moving it into the [Final](#final) state.

### Accepted

If the WG decides to accept the proposal as-is without requesting an experimental
implementation, a WG member shall:

1. Ensure the `Status` in the frontmatter of the RFC document is set to `Accepted`.
1. Apply the label `RFC - Accepted` to the PR.
1. Update the [RFC History][07] table to reflect the changed status.
1. Merge the PR.

When the WG is satisfied with the implementation, a WG member will start the process
to finalize the RFC, moving it into the [Final](#final) state.

### Withdrawn

If an Author decides to withdraw their RFC, either the Author or a WG member shall close the
PR without merging it.

A WG member shall apply the `RFC - Withdrawn` label to the PR, indicating that the author
withdrew the RFC.

### Final

When the WG is satisfied with the implementation for an RFC, a WG member will begin
the process to finalize the RFC.

To finalize an RFC, a WG member shall submit a PR which:

1. Ensures the `Status` in the frontmatter of the RFC document is set to `Final`.
1. Move the RFC document from the `rfc/draft` folder to `rfc/final`.
1. Update the [RFC History][07] table to reflect the changed status.

Any proposed changes should be made through a new RFC or as an issue in the
[PowerShell/DSC repository][08]. New RFCs should reference old RFCs where applicable.

## History

v1.0 - Initial draft.

[01]:https://github.com/PowerShell/PowerShell-RFC/blob/master/RFC0000-RFC-Process.md
[02]: https://github.com/PowerShell/PowerShell/blob/master/docs/community/working-group.md
[03]: https://github.com/PowerShell/DSC/issues/new/choose
[04]: RFCNNNN-New-RFC-Template.md
[05]: 2022-09-27
[06]: https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/proposing-changes-to-your-work-with-pull-requests/about-pull-requests#draft-pull-requests
[07]: readme.md#rfc-history
[08]: https://github.com/powershell/dsc
