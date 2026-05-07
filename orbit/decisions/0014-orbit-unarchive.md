---
status: accepted
date-created: 2026-05-07
date-modified: 2026-05-07
---
# 0014. Unarchive orbit; reverse ops 0031

## Context and Problem Statement

Decision 0031 in the ops repo cut orbit on the grounds that "the framework wasn't earning its place." Two subsequent diagnostic passes sharpened the picture: the cut was right about surface area (cards plus decisions are the only artefacts that belong on Hugh's desk) but missed the front end (the v1 design skill drove too far into implementation; the lever was *better* front-end thinking, not less). The trimmed shape — strict surface-area discipline plus a tabletop-flavoured front end — earns the framework its place.

## Considered Options

- **A. Stay archived; rebuild the trimmed shape elsewhere (in `ops`).** Sharp break from orbit's lineage; loses the rally + drive + review patterns that worked.
- **B. Stay archived; abandon the workflow framework idea entirely.** Each repo improvises. No leverage on planning discipline; the diagnostics that drove 0031 silently re-emerge.
- **C. Unarchive orbit; install front-loaded thinking; cap the executive surface.** Preserves what worked; fixes the front end; enforces strict surface area to prevent re-bloat.

## Decision Outcome

Chosen option: **C — unarchive orbit, install front-loaded thinking via tabletop, cap the executive surface at two artefacts.** The orbit pipeline (card → tabletop → spec → rally → drive) is sound. Reversing 0031 is cheaper than rebuilding next door. The two failure modes 0031 cut against (surface bloat, front-end drift to implementation) are addressed by load-bearing rules: the two-artefact contract (card 0018) caps surface area; the contract-not-solution rule (decision 0017) prevents tabletop from re-running the design failure.

### Consequences

- Good, because orbit's lineage (rally + drive + review patterns) is preserved without rebuild.
- Good, because the tabletop front-end addresses the *real* failure mode of v1 (under-developed alignment) rather than treating the symptom.
- Good, because the two-artefact contract (card 0018) is the constitutional rule that prevents surface-area regression.
- Bad, because orbit v1 carries skill bloat (~21 skills); a curation pass is required before the unarchive feels clean.
- Bad, because the choices/decisions vocabulary delta (orbit ships choices, ops keeps decisions — see 0016) creates cross-repo friction.
- Neutral, because ops decision 0031 (archival) is reversed and ops decision 0032 (two-artefact / curator pattern) is preserved as the cross-cutting rule orbit imports.
