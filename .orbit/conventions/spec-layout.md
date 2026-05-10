# Spec layout convention

Each spec lives in its own folder under `.orbit/specs/<id>/`. The folder
holds the spec's `spec.yaml` plus every sidecar artefact tied to that
spec — drive state, rally state, notes, tasks, interviews, reviews.

## Sidecar inventory

| Path                                                 | Purpose                                                              |
|------------------------------------------------------|----------------------------------------------------------------------|
| `.orbit/specs/<id>/spec.yaml`                        | The spec itself — goal, status, cards, labels, `acceptance_criteria` |
| `.orbit/specs/<id>/tasks.jsonl`                      | Append-only task event stream                                        |
| `.orbit/specs/<id>/notes.jsonl`                      | Append-only timestamped notes                                        |
| `.orbit/specs/<id>/drive.yaml`                       | Drive orchestration state (single-card drive)                        |
| `.orbit/specs/<id>/rally.yaml`                       | Rally orchestration state (multi-card rally lead)                    |
| `.orbit/specs/<id>/decisions.md`                     | Rally per-child decision pack (Stage 2 output)                       |
| `.orbit/specs/<id>/interview.md`                     | Design interview record (feeds the spec's ACs)                       |
| `.orbit/specs/<id>/review-spec-<date>.md`            | Spec review verdict (cycle 1 — no suffix)                            |
| `.orbit/specs/<id>/review-spec-<date>-v2.md`         | Spec review verdict (cycle 2)                                        |
| `.orbit/specs/<id>/review-spec-<date>-v3.md`         | Spec review verdict (cycle 3)                                        |
| `.orbit/specs/<id>/review-pr-<date>.md`              | PR review verdict (cycle 1)                                          |
| `.orbit/specs/<id>/review-pr-<date>-v2.md`           | PR review verdict (cycle 2)                                          |
| `.orbit/specs/<id>/review-pr-<date>-v3.md`           | PR review verdict (cycle 3)                                          |

The cycle-suffix convention: cycle 1 has no suffix; cycles 2 and 3
append `-v2` / `-v3` before the `.md` extension.

## Substrate-scanner rule

`list_spec_files` in `orbit-state/crates/core/src/layout.rs` walks the
immediate subdirectories of `.orbit/specs/` and returns each
`<id>/spec.yaml` it finds. Folders without a `spec.yaml` are skipped
silently; top-level `<id>.yaml` files are ignored. This rule is
consumed by `verify_all`, `Index::rebuild_from_files`, and
`verbs::spec.list`.

## Prior experiment: flat sidecar layout

Between 2026-05-08 and the choice 0021 reversion, specs lived as flat
sidecars at the top of `.orbit/specs/`:

- `.orbit/specs/<id>.yaml`                 (now `.orbit/specs/<id>/spec.yaml`)
- `.orbit/specs/<id>.drive.yaml`           (now `.orbit/specs/<id>/drive.yaml`)
- `.orbit/specs/<id>.rally.yaml`           (now `.orbit/specs/<id>/rally.yaml`)
- `.orbit/specs/<id>.notes.jsonl`          (now `.orbit/specs/<id>/notes.jsonl`)
- `.orbit/specs/<id>.tasks.jsonl`          (now `.orbit/specs/<id>/tasks.jsonl`)
- `.orbit/specs/<id>.interview.md`         (now `.orbit/specs/<id>/interview.md`)
- `.orbit/specs/<id>.decisions.md`         (now `.orbit/specs/<id>/decisions.md`)
- `.orbit/specs/<id>.review-spec-<date>.md` (now `.orbit/specs/<id>/review-spec-<date>.md`)
- `.orbit/specs/<id>.review-pr-<date>.md`   (now `.orbit/specs/<id>/review-pr-<date>.md`)

The flat layout was abandoned for three reasons (per choice 0021):
visual mess in `ls .orbit/specs/`, prefix-collision risk between spec
ids that prefix each other, and non-atomic rename when a spec id
changes (every sidecar must move separately). The folder shape solves
all three.

The one-shot `orbit spec migrate-layout` verb folds any remaining
flat sidecars into per-spec folders.
