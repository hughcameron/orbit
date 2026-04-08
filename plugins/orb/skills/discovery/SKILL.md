---
name: discovery
description: Socratic discovery — explore a vague idea through Q&A, save the record as markdown
---

# /orb:discovery

Explore a vague idea or requirement through Socratic questioning. Use this when there's no card yet, or the card is thin (< 3 scenarios). The outcome is clarity — an interview record that's ready for `/orb:spec`.

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
- Check `cards/` for any related feature cards. If a thin card exists, read it as starting context.
- Identify the output directory: `specs/YYYY-MM-DD-<topic-slug>/` (create if needed)

### 2. Conduct the Discovery

Adopt the socratic interviewer role (see `/orb:interviewer` for the full persona).

Target: **8–12 questions** covering:
- **Goal**: What are we trying to achieve? What does success look like?
- **Users**: Who has this need? What's their context?
- **Constraints**: Platform, performance, compatibility, budget
- **Success criteria**: How will we know it works?
- **Edge cases**: What could go wrong?
- **Prior art**: Are there existing approaches worth studying?

**For each question:**

1. Present the question using **AskUserQuestion** with contextually relevant suggested answers:
   - Binary questions: use the natural choices
   - Technology choices: suggest common options for the context
   - Open-ended questions: suggest representative answer categories
   - The author can always type a custom response

2. Record the Q&A pair in your working notes

3. After each answer, target the biggest remaining source of ambiguity

### 3. Questioning Strategy

- Always end responses with a question
- Target the biggest source of ambiguity
- Build on previous responses
- Be specific and actionable
- Use ontological questions: "What IS this?", "Root cause or symptom?", "What are we assuming?"
- You are ONLY a questioner — never write code, edit files, or run commands
- Continue until the author says "done" or requirements are clear

### 4. Ambiguity Assessment

After every 3-4 questions, assess clarity across three dimensions:
- **Goal Clarity**: Is the objective specific and well-defined?
- **Constraint Clarity**: Are limitations and boundaries specified?
- **Success Criteria Clarity**: Are success criteria measurable?

If all three are clear (ambiguity ≤ 0.2), suggest wrapping up.

### 5. Surface Decisions

During discovery, choices between approaches will surface. When a clear choice is made:

1. Note it in the record under **Decisions Surfaced**
2. Each entry should name the choice, the alternatives considered, and the rationale
3. These become MADR decision records during or after the session (the spec will reference them)

### 6. Save the Record

Save the Q&A as: `specs/YYYY-MM-DD-<topic-slug>/interview.md`

```markdown
# Discovery: <Topic>

**Date:** YYYY-MM-DD
**Interviewer:** <agent name>
**Card:** cards/NNNN-slug.yaml (if applicable)
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
- <choice made>: chose X over Y because Z (→ decisions/NNNN if recorded)

### Open Questions
- <anything still unclear>
```

---

**Next step:** `/orb:spec` to generate a structured specification from this discovery.
