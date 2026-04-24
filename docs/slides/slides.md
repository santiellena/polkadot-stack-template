<div class="hero-slide">
  <div class="hero-brand">Aperio</div>
  <div class="hero-meaning">Latin: to make accessible</div>
  <h1>Open source needs a source of truth that cannot be switched off.</h1>
  <p class="hero-sub">
    Aperio is a censorship-resistant repository platform. Git stays where developers
    work. Aperio makes the official repository state public, portable, and hard for
    any single platform to control.
  </p>
</div>

Note:
Open with the problem and the product in one sentence. This is not "GitHub on-chain." It is infrastructure that protects the official repository state from centralized platform risk while keeping developers in Git.

---

<div>
  <div class="eyebrow">Why now</div>
  <h2>The failure mode is already visible.</h2>
  <div class="case-stage r-stack">
    <div class="case-copy fragment fade-out" data-fragment-index="0">
      <p class="lead">
        <span class="inline-emphasis">Tornado Cash</span> showed that you do not need
        to break a protocol to disrupt a project. Pressure can move through the
        repository, the developers, and the platform hosting the source of truth.
      </p>
      <p class="quiet">
        When one platform controls the official repo, that platform becomes a choke
        point.
      </p>
    </div>
    <div class="timeline-card">
      <div class="timeline-step fragment fade-in" data-fragment-index="1">
        <span>Aug 8, 2022</span>
        <strong>OFAC sanctions</strong>
        <em>Pressure begins</em>
      </div>
      <div class="timeline-step fragment fade-in" data-fragment-index="2">
        <span>hours later</span>
        <strong>Repos and accounts removed</strong>
        <em>GitHub becomes a choke point</em>
      </div>
      <div class="timeline-step fragment fade-in" data-fragment-index="3">
        <span>Aug 10, 2022</span>
        <strong>Developer arrested</strong>
        <em>Human layer pressured</em>
      </div>
      <div class="timeline-step fragment fade-in" data-fragment-index="4">
        <span>Mar 21, 2025</span>
        <strong>Sanctions delisted</strong>
        <em>Damage already done</em>
      </div>
    </div>
  </div>
</div>

Note:
Use this as a concrete example of centralized hosting becoming a choke point. Sources: U.S. Treasury sanctioned Tornado Cash on August 8, 2022 (https://home.treasury.gov/news/press-releases/jy0916); EFF described GitHub takedowns and account suspensions after the order (https://www.eff.org/deeplinks/2023/04/update-tornado-cash); Dutch FIOD announced on August 12, 2022 that it arrested a suspected developer on August 10, 2022 (https://www.fiod.nl/arrest-of-suspected-developer-of-tornado-cash/); Treasury delisted Tornado Cash on March 21, 2025 (https://home.treasury.gov/news/press-releases/sb0057). Avoid claiming Aperio solves prosecution. It protects repository availability, continuity, and the official repository record.

---

<div>
  <div class="eyebrow">Product</div>
  <h2>Keep Git. Remove the choke point.</h2>
  <div class="diagram-shell architecture-shell">
    <div class="arch-bottom-row">
      <div class="flow-box git-box fragment fade-up" data-fragment-index="0">
        <div class="box-title box-title-only">Git repo</div>
      </div>
      <div class="flow-arrow-chip fragment fade-up" data-fragment-index="1">bundle</div>
      <div class="flow-box cli-box fragment fade-up" data-fragment-index="1">
        <div class="box-title box-title-only">Aperio workflow</div>
      </div>
      <div class="flow-arrow-chip fragment fade-up" data-fragment-index="2">upload</div>
      <div class="flow-box bulletin-box fragment fade-up" data-fragment-index="2">
        <div class="box-title box-title-only">Bundle storage</div>
      </div>
    </div>
    <div class="arch-center-note fragment fade-up" data-fragment-index="6">The code stays where developers work. The official repo state becomes public and portable.</div>
    <div class="arch-bottom-row">
      <div class="flow-box wallet-box fragment fade-up" data-fragment-index="3">
        <div class="box-title box-title-only">Project approval</div>
      </div>
      <div class="flow-arrow-chip fragment fade-up" data-fragment-index="3">authorizes</div>
      <div class="flow-box cli-box fragment fade-up" data-fragment-index="3">
        <div class="box-title box-title-only">Aperio workflow</div>
      </div>
      <div class="flow-arrow-chip fragment fade-up" data-fragment-index="4">submits</div>
      <div class="flow-box contract-box fragment fade-up" data-fragment-index="4">
        <div class="box-title box-title-only">Public registry</div>
      </div>
      <div class="flow-arrow-chip fragment fade-up" data-fragment-index="5">read</div>
      <div class="flow-box frontend-box fragment fade-up" data-fragment-index="5">
        <div class="box-title box-title-only">Live repo view</div>
      </div>
    </div>
    <div class="cid-link fragment fade-up" data-fragment-index="6">Developers keep Git. Projects gain resilient coordination.</div>
  </div>
</div>

Note:
Explain the product in one line: developers keep Git, Aperio stores the repository artifact, and a public registry records the official result. The benefit is continuity, auditability, and less dependence on a single host.

---

<div>
  <div class="eyebrow">Business</div>
  <h2>Start where repository failure is expensive.</h2>
  <div class="diagram-shell">
    <div class="econ-rail">
      <div class="flow-box compact-box center-box dao-box fragment fade-up" data-fragment-index="0">
        <div class="box-title box-title-only">Maintainers / DAO</div>
      </div>
      <div class="flow-pill pink-pill fragment fade-up" data-fragment-index="0">governs</div>
    </div>
    <div class="econ-main-row">
      <div class="flow-box compact-box center-box community-box fragment fade-up" data-fragment-index="1">
        <div class="box-title box-title-only">Protocols / funds</div>
      </div>
      <div class="flow-arrow-chip fragment fade-up" data-fragment-index="1">funds</div>
      <div class="flow-box compact-box center-box treasury-box fragment fade-up" data-fragment-index="2">
        <div class="box-title box-title-only">Repo treasury</div>
      </div>
      <div class="flow-arrow-chip fragment fade-up" data-fragment-index="2">routes via</div>
      <div class="flow-box compact-box center-box contract-box fragment fade-up" data-fragment-index="3">
        <div class="box-title box-title-only">Public registry</div>
      </div>
    </div>
    <div class="econ-workers-label fragment fade-up" data-fragment-index="4">Accepted work gets paid. Strong repos attract stronger contributors.</div>
    <div class="econ-workers">
      <div class="worker-column">
        <div class="worker-link fragment fade-up" data-fragment-index="4">merge pay</div>
        <div class="flow-box compact-box center-box contributor-box fragment fade-up" data-fragment-index="4">
          <div class="box-title box-title-only">Contributors</div>
        </div>
      </div>
      <div class="worker-column">
        <div class="worker-link fragment fade-up" data-fragment-index="5">review pay</div>
        <div class="flow-box compact-box center-box reviewer-box fragment fade-up" data-fragment-index="5">
          <div class="box-title box-title-only">Reviewers</div>
        </div>
      </div>
    </div>
  </div>
  <p class="market-line fragment fade-up" data-fragment-index="6">Fund the repo. Attract talent. Preserve continuity.</p>
</div>

Note:
Present this as the target customer, not a proven market fact: treasury-backed, high-stakes open source. These projects already have capital, governance, contributors, and continuity risk. Aperio gives them coordination infrastructure around the repository itself, not just another code host.

---

<div class="qa-slide">
  <div class="hero-mark">Demo</div>
  <h2>Proposal → review → merge → canonical history.</h2>
  <div class="diagram-shell architecture-shell">
    <div class="arch-top-row">
      <div class="flow-box git-box">
        <div class="box-title box-title-only">Create repo</div>
      </div>
      <div class="flow-arrow-chip">propose</div>
      <div class="flow-box bulletin-box">
        <div class="box-title box-title-only">Upload bundle</div>
      </div>
      <div class="flow-arrow-chip">review</div>
      <div class="flow-box wallet-box">
        <div class="box-title box-title-only">Approve</div>
      </div>
      <div class="flow-arrow-chip">merge</div>
      <div class="flow-box contract-box">
        <div class="box-title box-title-only">Update HEAD</div>
      </div>
    </div>
    <div class="cid-link">The demo should spend most time in the live app, not on this slide.</div>
  </div>
</div>

Note:
Use this slide as the handoff into the live demo. Open the deployed frontend, show a repository, show HEAD and CID, submit or inspect a proposal, download the bundle for review, record approval, merge as maintainer, and return to history to show the accepted canonical commit. If the network is fragile, use a recorded fallback but still show the live DotNS URL.

---

<div>
  <div class="eyebrow">What broke</div>
  <h2>The stack worked, but the edges were sharp.</h2>
  <div class="diagram-shell">
    <div class="econ-main-row">
      <div class="flow-box compact-box center-box wallet-box">
        <div class="box-title box-title-only">Wallet signing</div>
      </div>
      <div class="flow-arrow-chip">hardest edge</div>
      <div class="flow-box compact-box center-box bulletin-box">
        <div class="box-title box-title-only">Bulletin / DotNS deploy</div>
      </div>
      <div class="flow-arrow-chip">endpoint friction</div>
      <div class="flow-box compact-box center-box contract-box">
        <div class="box-title box-title-only">Bundle verification</div>
      </div>
    </div>
    <p class="market-line">The lesson: test the whole decentralized path early.</p>
  </div>
</div>

Note:
This is the section reviewers care about. Be direct: frontend/CLI signing was harder than expected, especially the path from wallet auth to extrinsic approval and submission. Bulletin and DotNS worked, but defaults/endpoints/signing context needed hands-on debugging. The important product compromise is that bundle/commit consistency is still enforced by reviewer workflow, not automatically by the CLI.

---

<div>
  <div class="eyebrow">Next</div>
  <h2>Make Aperio feel native to Git.</h2>
  <div class="diagram-shell">
    <div class="econ-workers">
      <div class="worker-column">
        <div class="worker-link">near term</div>
        <div class="flow-box compact-box center-box cli-box">
          <div class="box-title box-title-only">Verify bundle ↔ commit</div>
        </div>
      </div>
      <div class="worker-column">
        <div class="worker-link">near term</div>
        <div class="flow-box compact-box center-box frontend-box">
          <div class="box-title box-title-only">Release creation</div>
        </div>
      </div>
      <div class="worker-column">
        <div class="worker-link">direction</div>
        <div class="flow-box compact-box center-box dao-box">
          <div class="box-title box-title-only">DAO maintainership</div>
        </div>
      </div>
    </div>
    <p class="market-line">The product should become closer to git push polkadot main.</p>
  </div>
</div>

Note:
Close with concrete next steps: automatic bundle/commit verification, CLI wallet signing instead of SURI import, release creation in web/CLI, Statement Store for review discussion, DAO-controlled maintainership, and a more native Git remote style flow.

---

<div class="qa-slide">
  <div class="hero-mark">Q&A</div>
  <h2>What should critical open source depend on?</h2>
  <a class="qa-link" href="https://aperio.dot.li/">
    <span class="live-dot" aria-hidden="true"></span>
    <span class="qa-link-text">https://aperio.dot.li/</span>
  </a>
</div>
