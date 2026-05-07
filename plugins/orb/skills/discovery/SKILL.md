---
name: discovery
description: Discovery session — explore a vague idea through Q&A, capture what good looks like
---

# /orb:discovery

Explore a vague idea or requirement through questioning. Use this when there's no card yet, or the card is thin (< 3 scenarios). The outcome is clarity — what good looks like, captured as an interview record ready for `/orb:spec`.

## Usage

```
/orb:discovery [topic]
```

## When to Use

- No card exists for this topic
- A card exists but has < 3 scenarios (not enough to design against)
- The problem space is unclear and needs open exploration

If a well-formed card exists (≥ 3 scenarios), use `/orb:design` instead.

## Instructions

### 1. Setup

- Determine the topic from the author's input
- Check `.orbit/cards/` for any related feature cards. If a thin card exists, read it as starting context.
- Identify the output directory: `.orbit/specs/YYYY-MM-DD-<topic-slug>/` (create if needed)

### 2. Search for Prior Art

Before asking questions, run a keyword scan (see `/orb:keyword-scan`) against `.orbit/specs/` and `.orbit/choices/` using terms from the topic. If prior specs or decisions already explored this area, front-load what they found — the discovery session should build on existing knowledge, not rediscover it.

### 3. Conduct the Discovery

Adopt the interviewer role (see `/orb:interviewer` for the full persona and the decision-level gate).

**The author's job is to define what good looks like.** Discovery questions stay at the intent level — goals, users, constraints, success criteria. Implementation questions are recorded as notes, not asked.

Target: **6–10 questions** covering:
- **Goal**: What are we trying to achieve? What does success look like?
- **Users**: Who has this need? What's their context?
- **Constraints**: Platform, performance, compatibility, budget
- **Success criteria**: How will we know it works?
- **Scope**: What's in, what's explicitly out?
- **Prior art**: Are there existing approaches worth studying?

**For each question:**

1. Present the question using **AskUserQuestion** with contextually relevant suggested answers:
   - Binary questions: use the natural choices
   - Technology choices: suggest common options for the context
   - Open-ended questions: suggest representative answer categories
   - The author can always type a custom response

2. Record the Q&A pair in your working notes

3. After each answer, target the biggest remaining source of ambiguity

### 4. Questioning Strategy

- Always end responses with a question
- Target the biggest source of ambiguity
- Build on previous responses
- Be specific and actionable
- Use ontological questions: "What IS this?", "Root cause or symptom?", "What are we assuming?"
- You are ONLY a questioner — never write code, edit files, or run commands
- Continue until the author says "done" or requirements are clear

### 5. Ambiguity Assessment

After every 3-4 questions, assess clarity across three dimensions:
- **Goal Clarity**: Is the objective specific and well-defined?
- **Constraint Clarity**: Are limitations and boundaries specified?
- **Success Criteria Clarity**: Are success criteria measurable?

If all three are clear (ambiguity ≤ 0.2), suggest wrapping up.

### 6. Surface Decisions

During discovery, choices between approaches will surface. When a clear choice is made:

1. Note it in the record under **Decisions Surfaced**
2. Each entry should name the choice, the alternatives considered, and the rationale
3. These become MADR decision records during or after the session (the spec will reference them)

### 7. Save the Record

Save the Q&A as: `.orbit/specs/YYYY-MM-DD-<topic-slug>/interview.md`

```markdown
# Discovery: <Topic>

**Date:** YYYY-MM-DD
**Interviewer:** <agent name>
**Card:** .orbit/cards/NNNN-slug.yaml (if applicable)
**Mode:** discovery

---

## Context

<Initial topic/idea description>

## Q&A

### Q1: <Short label>
**Q:** <question>
**A:** <answer>

[...]

---

## Summary

### Goal
<Extracted goal statement>

### Constraints
- <constraint 1>

### Success Criteria
- <criterion 1>

### Decisions Surfaced
- <choice made>: chose X over Y because Z (→ .orbit/choices/NNNN if recorded)

### Implementation Notes
- <means-level observations from exploration — starting context for the implementing agent>

### Open Questions
- <anything still unclear — intent-level only>
```

---

**Next step:** `/orb:spec` to generate a structured specification from this discovery.
