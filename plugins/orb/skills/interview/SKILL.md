---
name: interview
description: Socratic interview — clarify requirements via Q&A, save the record as markdown
---

# /orb:interview

Socratic interview workflow that clarifies vague ideas into actionable requirements, with the Q&A record saved as a markdown file.

## Usage

```
/orb:interview [topic]
```

## Instructions

### 1. Setup

- Determine the topic from the user's input
- Check `cards/` for an existing feature card matching the topic. If found, read it — including its scenarios and references.
- Identify the output directory: `specs/YYYY-MM-DD-<topic-slug>/` (create if needed)
- **Assess card quality** to determine interview mode (see below)

### 2. Choose Interview Mode

**Card-aware mode** — when a card exists with ≥ 3 scenarios:

The card already answers *who*, *what*, and *why*. Don't re-ask those. Instead:

1. Open with a summary: "I've read card NNNN. Your scenarios cover X, Y, Z. Let me probe the gaps."
2. If the card has `references`, probe them early: "Your references include A, B, C — these represent different philosophies. Which feel are you targeting?" This naturally surfaces design decisions.
3. Focus questions on:
   - **Technical constraints** — platform, performance, compatibility boundaries
   - **Approach selection** — when references or scenarios imply a choice between strategies, probe it explicitly. Each choice is a potential decision record.
   - **Edge cases** the scenarios don't cover — failure modes, concurrency, empty states
   - **Non-functional requirements** — speed, accessibility, security
4. Target: **4–6 questions**. The card did the heavy lifting.

**Full mode** — when no card exists, or card has < 3 scenarios:

Run the full Socratic interview (current behaviour). Target: **8–12 questions**.

### 3. Conduct the Interview

Adopt the socratic interviewer role (see `/orb:interviewer` for the full persona).

**For each question:**

1. Present the question using **AskUserQuestion** with contextually relevant suggested answers:
   - Binary questions: use the natural choices
   - Technology choices: suggest common options for the context
   - When a card has references: use those references as suggested answers where relevant
   - Open-ended questions: suggest representative answer categories
   - The user can always type a custom response

2. Record the Q&A pair in your working notes

3. After each answer, target the biggest remaining source of ambiguity

### 4. Questioning Strategy

- Always end responses with a question
- Target the biggest source of ambiguity
- Build on previous responses
- Be specific and actionable
- Use ontological questions: "What IS this?", "Root cause or symptom?", "What are we assuming?"
- **Probe references for decisions**: When the user cites prior art, ask what specifically they want to adopt vs. avoid. "You mentioned uv's speed — does that mean suppress intermediate output, or stream only errors?" Each clarification either confirms a constraint or surfaces a decision.
- You are ONLY a questioner — never write code, edit files, or run commands
- Continue until the user says "done" or requirements are clear

### 5. Ambiguity Assessment

After every 3-4 questions (card-aware mode: after every 2-3), assess clarity:
- **Goal Clarity**: Is the objective specific and well-defined?
- **Constraint Clarity**: Are limitations and boundaries specified?
- **Success Criteria Clarity**: Are success criteria measurable?

If all three are clear (ambiguity ≤ 0.2), suggest wrapping up.

### 6. Surface Decisions

During the interview, choices between approaches will surface — especially when probing references. When a clear choice is made:

1. Note it in the interview record under **Decisions Surfaced**
2. Each entry should name the choice, the alternatives considered, and the rationale
3. These become MADR decision records during or after the interview (the spec will reference them)

### 7. Save the Record

Save the Q&A as: `specs/YYYY-MM-DD-<topic-slug>/interview.md`

```markdown
# Interview: <Topic>

**Date:** YYYY-MM-DD
**Interviewer:** <agent name>
**Card:** cards/NNNN-slug.yaml (if applicable)
**Mode:** card-aware | full

---

## Context

<Initial topic/idea description>
<Card summary if one was used: feature name, scenario count, references>

## Interview Q&A

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

**Next step:** `/orb:spec` to generate a structured specification from this interview.
