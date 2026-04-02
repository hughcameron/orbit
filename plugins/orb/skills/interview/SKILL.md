---
name: interview
description: Socratic interview — clarify requirements via Q&A, save the record as markdown
disable-model-invocation: true
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
- Check `cards/` for an existing feature card matching the topic. If found, read it and use its scenarios as starting requirements to refine.
- Identify the output directory: `specs/YYYY-MM-DD-<topic-slug>/` (create if needed)

### 2. Conduct the Interview

Adopt the socratic interviewer role (see `/orb:interviewer` for the full persona).

**For each question:**

1. Present the question using **AskUserQuestion** with contextually relevant suggested answers:
   - Binary questions: use the natural choices
   - Technology choices: suggest common options for the context
   - Open-ended questions: suggest representative answer categories
   - The user can always type a custom response

2. Record the Q&A pair in your working notes

3. After each answer, target the biggest remaining source of ambiguity

### 3. Questioning Strategy

- Always end responses with a question
- Target the biggest source of ambiguity
- Build on previous responses
- Be specific and actionable
- Use ontological questions: "What IS this?", "Root cause or symptom?", "What are we assuming?"
- You are ONLY a questioner — never write code, edit files, or run commands
- Continue until the user says "done" or requirements are clear

### 4. Ambiguity Assessment

After every 3-4 questions, assess clarity across three dimensions:
- **Goal Clarity**: Is the objective specific and well-defined?
- **Constraint Clarity**: Are limitations and boundaries specified?
- **Success Criteria Clarity**: Are success criteria measurable?

If all three are clear (ambiguity ≤ 0.2), suggest wrapping up.

### 5. Save the Record

Save the Q&A as: `specs/YYYY-MM-DD-<topic-slug>/interview.md`

```markdown
# Interview: <Topic>

**Date:** YYYY-MM-DD
**Interviewer:** <agent name>

---

## Context

<Initial topic/idea description>
<Card reference if one was used as starting context>

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

### Open Questions
- <anything still unclear>
```

---

**Next step:** `/orb:spec` to generate a structured specification from this interview.
