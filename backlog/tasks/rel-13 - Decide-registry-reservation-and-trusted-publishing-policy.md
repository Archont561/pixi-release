---
id: REL-13
title: Decide registry reservation and trusted-publishing policy
status: To Do
assignee: []
created_date: '2026-10-23 15:01'
labels:
  - release
  - architecture
milestone: m-3
dependencies:
  - REL-6
references:
  - 'https://github.com/Archont561/pixi-release/issues/7'
  - 'https://github.com/Archont561/pixi-release/issues/8'
priority: high
type: spike
ordinal: 13000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Issues #7 and #8 (near-duplicate proposals) ask for `pixi release reserve` to publish 0.0.0 placeholder packages to crates.io, PyPI, npm, NuGet, Maven Central, pkg.go.dev, Conan and CRAN, plus OIDC trusted-publishing bootstrapping. A reservation is a registry write, so decision-9 holds it behind REL-6; and any archive it builds is repository work, not controller work (decision-2). This task produces the decision before any code: which registries are admitted, what a reservation is, how it is evidenced and resumed, and whether the trust configuration is rendered or documented.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A decision is recorded and accepted naming every requested registry as admitted or rejected, with a reason per registry and its artifact kind.
- [ ] #2 Reservation lifecycle is defined against plan -> prepare -> cut -> verify -> publish -> resume, including the evidence record, the per-target outcome and the resume behaviour that decision-9 requires before a target is added.
- [ ] #3 Archive construction is assigned to consumer `pixi run` tasks and no compiler, toolchain or packaging knowledge enters the controller (decision-2).
- [ ] #4 The OIDC trust boundary is reconciled with the consumer-owned token slot of decision-5, and each admitted registry is named as trusting GitHub OIDC, a scoped token, or both.
- [ ] #5 No command, template or scaffold is merged under this task.
<!-- AC:END -->
