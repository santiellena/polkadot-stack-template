# Aperio Presentation Speech

## Slide 1

Aperio starts from a simple problem:
for many open-source projects, the de facto source of truth still depends on a centralized platform.

That is fine until that platform becomes a point of pressure.

So Aperio is a censorship-resistant repository platform.
Git still manages the code and history.
What Aperio changes is who gets to define the official repository state.

The goal is not to put code on-chain.
The goal is to make the source of truth for a repository public, portable, and much harder for any single platform to control.

## Slide 2

Why does that matter?
Because the failure mode is already visible.

Tornado Cash is the clearest example.
After OFAC sanctioned Tornado Cash on August 8, 2022, GitHub took down the main repository and suspended related developer accounts.
Then, on August 10, 2022, a suspected developer was arrested in the Netherlands.
And only later, on March 21, 2025, Treasury removed the sanctions.

That is the broader lesson:
when one platform effectively controls the official repository, that platform becomes a choke point.

And if open source is critical infrastructure, then that choke point matters.

## Slide 3

So the product idea is simple:
keep Git, remove the choke point.

Developers still work the way they already work.
Aperio packages the repository artifact, stores it, and records the official result in a public registry.
Then anyone can read that same public state through the product interface.

So the value proposition is:
no forced change to the developer workflow,
better continuity for the project,
and a public, auditable source of truth for the repository.

## Slide 4

Our first target market is high-stakes, treasury-backed open source.

These projects already have contributors, reviewers, governance, and capital.
What they often do not have is infrastructure that ties repository continuity, coordination, and funding together.

Aperio gives them that layer.
Capital can move into the repository treasury.
Useful work can be rewarded when it is merged.
Review labor can be rewarded too.

So this is not only a censorship story.
It is also coordination infrastructure for serious open-source projects.

## Slide 5

For the demo, I would show the happy path directly.

First: open the deployed web app at aperio.dot.li.
Show the registry reading canonical repository state from the contract.

Then: create or open a repository, show the current HEAD commit and CID, and explain that the CID points to a Git bundle stored off-chain.

Next: submit a proposal bundle.
The important part is that the proposed commit and CID are recorded, but the code itself stays in Git/Bulletin.

Then: switch to the reviewer view, download the bundle, inspect it locally, and record an approval.

Finally: as maintainer, merge the proposal and show the canonical HEAD changing.

That is the full Aperio loop:
proposal, review, merge, and canonical history.

## Slide 6

What broke?
Mostly the edges between young pieces of the stack.

The hardest path was signing from a frontend and CLI-like flow.
I wanted a smooth loop where a user authenticates with a wallet, the app builds an extrinsic, requests approval, receives the signature, and submits it.
That flow works in the web app, but getting there was less obvious than expected.

Bulletin and DotNS deployment also required learning which endpoints and signing contexts actually worked together.
The useful lesson was that these tools are real, but the path is still alpha: you need to test assumptions end-to-end instead of trusting that every default is wired together.

The biggest product compromise is that the current system records CIDs and commits, but it still relies on off-chain review to verify that the bundle really contains the commit being proposed.
That is acceptable for this MVP because the reviewer role exists specifically to inspect code, but I would tighten the CLI around that next.

## Slide 7

So the bet behind Aperio is that important open source needs better infrastructure:
not just to exist,
but to remain visible, governable, and fundable.

If that is true, then the official repository should not depend on one platform to stay official.

That is the problem Aperio is trying to solve.

The project is live at:
aperio.dot.li

The next steps are clear:
make the CLI more Git-native,
add release creation to the web and CLI,
verify bundle/commit consistency automatically,
move CLI signing to an external wallet,
and use Statement Store for proposal discussion and review feedback.
