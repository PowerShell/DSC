---
RFC:          RFC0000
Author:       Mikey Lombardi
Status:       Draft
SupercededBy: N/A
Version:      1.0
Area:         Process
---

# DSC RFC Process and Guidelines

A DSC RFC (Request for Comments) is a publication to propose design changes and improvements to
DSC. This provides the community an opportunity to provide feedback before code is written where it
becomes harder to change at the risk of compatibility.

This process was adapted from the PowerShell RFC process, which itself was adapted from the Chef
RFC process and the DMTF.org process.

## Roles

- **Author**: All members of the community are allowed to author new RFCs and can provide feedback
  to any RFC.
- **DSC Committee**: The design committe that votes to accept or reject an RFC. Currently, the DSC
  committee includes the members of the DSC team.
- **Committee Member**: An individual member of the DSC Committee.
- **Working Group (WG)**: A group responsible for deciding whether or not an issue in the
  repository requires a proposal and for providing feedback within an RFC proposal.
  
  For more information about Working Groups, see [Working Groups][01].

## Submitting an RFC

When submitting an RFC, the Author shall:

- Create a file named `RFCNNNN-<Title>.md` in the `.rfc/drafts` folder.

  The Author _shall not_ assign the RFC number. The author shall leave the `NNNN` in the
  filename.

  The file must use the [RFC template][02].

  Example: `RFCNNNN-docs-extension.md`
- Include any additional files, such as code samples, in the `.rfc/draft/RFCNNNN/` folder.
- Check `Allow edits from maintainers` option when submitting the PR so that the Committee can add
  the RFC number to the draft, update the title, and fix the filename.
- Submit the PR as a [draft PR][03].
- Use the prefix `RFC:` for the PR title.

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

While an RFC is in this state, we encourage contributors and community members to read, discuss,
and comment on the RFC. Discussion and iteration during the drafting stage provides information
and context for the committee during the reviewing stage.

After one month, the Author may mark their PR as ready for formal review, taking it out of draft. A
Committee member will then apply the `RFC - Reviewing` label to the PR.

### Reviewing

After the author marks their PR as ready for review, the RFC moves into the formal review state.
The RFC remains in this state until one of the following conditions is met:

- The Committee decides to reject the RFC, changing the state to [Rrejected](#rejected).
- The Committee requests an experimental implementation for the RFC, changing the state to
  [Experimental](#experimental).
- The Committee decides to accept the RFC as-is, changing the state to [Accepted](#accepted).
- The Author decides to withdraw their RFC, changing the state to [Withdrawn](#withdrawn).

> [!NOTE]
> The Committee may be slower to respond to RFCs where the Author has indicated that they don't
> plan to implement the RFC.

### Rejected

If the Committee decides not to proceed with the RFC, a Committee member shall close the PR instead
of merging it. The Committee should also add the `RFC - Rejected` label to denote that the
Committee rejected the RFC.

In the future, this can be done automatically with GitHub Actions.

### Experimental

Experimental implementations are used to provide a working example of proposed designs in order for
the Committee and other users to understand real-world usage of the proposal.

If the Committee decides to request an experimental implementation, a Committee member shall:

1. Ensure the `status` in the frontmatter of the RFC document is set to `experimental`.
1. Apply the label `RFC - Experimental` to the PR.
1. Update the [RFC History](readme.md#rfc-history) table to reflect the changed status.
1. Merge the PR.

The Author may be asked to continue to update the RFC as the usage of the experimental feature
drives new insight into how the feature should be designed.

When the Committee is satisfied with the experimental implementation, a Committee member will start
the process to finalize the RFC, moving it into the [Final](#final) state.

### Accepted

If the Committee decides to accept the proposal as-is without requesting an experimental
implementation, a Committee member shall:

1. Ensure the `status` in the frontmatter of the RFC document is set to `accepted`.
1. Apply the label `RFC - Accepted` to the PR.
1. Update the [RFC History](readme.md#rfc-history) table to reflect the changed status.
1. Merge the PR.

When the Committee is satisfied with the implementation, a Committee member will start the process
to finalize the RFC, moving it into the [Final](#final) state.

### Withdrawn

If an Author decides to withdraw their RFC, either the Author or a Committee member shall close the
PR without merging it.

A Committee member shall apply the `RFC - Withdrawn` label to the PR, indicating that the author
withdrew the RFC.

### Final

When the Committee is satisfied with the implementation for an RFC, a Committee member will begin
the process to finalize the RFC.

To finalize an RFC, a Committee member shall submit a PR which:

1. Ensures the `status` in the frontmatter of the RFC document is set to `final`.
1. Move the RFC document from the `.rfc/draft` folder to `.rfc/final`.
1. Update the [RFC History](readme.md#rfc-history) table to reflect the changed status.

Any proposed changes should be made through a new RFC or via an issue in the
[PowerShell/DSC repository][04]. New RFCs should reference old RFCs where applicable.

## History

v1.0 - Initial draft.

[01]: https://github.com/PowerShell/PowerShell/blob/master/docs/community/working-group.md
[02]: RFCNNNN-New-RFC-Template.md
[03]: https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/proposing-changes-to-your-work-with-pull-requests/about-pull-requests#draft-pull-requests
[04]: https://github.com/powershell/dsc
