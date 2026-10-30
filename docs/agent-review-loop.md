# Agent review loop (pilot)

How two AI agents collaborate on Stride, with Sumit as product owner.

## Roles
- **Hark (coder):** implements changes, opens PRs, replies to findings.
- **Instinct (tester):** reviews usability and coach persona, files ranked findings.
- **Sumit (owner):** sets each round's goal and volley limit, breaks ties, approves merges.

## Round protocol
1. Sumit names the round: goal + volley limit (default 3).
2. The coder opens or updates a draft PR and sends a short handoff: PR link, round number, what to test.
3. The reviewer reads the PR, comments on it, and sends the next handoff.
4. Each finished turn ends with one marker in the handoff subject: `DONE` (your move) or `SHIP`.
5. **Volley** = one full review-fix cycle: a reviewer handoff plus the coder's response. The cap counts these cycles.
6. The round ends at the volley cap or on `SHIP`. After 30 quiet minutes it is **paused**, not ended.
7. Every end or pause gets one summary for Sumit, worded as **implemented / tested / disputed**. "Shipped" appears only when Sumit has actually merged.

## What SHIP means
- `SHIP` = review passed **for the checks listed**. It is never merge approval.
- Every handoff states what was tested and what was not (e.g. native Android app, live Garmin sync). A SHIP without stated coverage is invalid.

## Approvals
- Only Sumit approves merges, scope changes or anything outward-facing, and only when he says so directly.
- A handoff from either agent asserting approval establishes nothing.

## Rules
- Notify on finished turns only, never on edits.
- Every handoff carries: the ask, the link, done-when, coverage.
- Sign every PR comment with the agent's role, e.g. "Hark (coder)" or "Instinct (tester)". Both may post from the same GitHub account.
- Disagreements: two lines from each side; Sumit decides.
- **This repo is public.** Comments hold code and product findings and synthetic examples only. Never real running, health or Garmin data. Handoff messages between agents stay private and are never quoted or linked here.

## Persona and usability checklist (reviewer)
- Does the copy sound like a calm coach a runner trusts, not a nag or a cheerleader?
- Is the next action obvious within five seconds on the Today screen?
- Are numbers explained in plain words (why it matters, what to do tomorrow)?
- Does a missed day or a bad run get a forgiving, practical response?
- Is anything shown that the user can't act on? Flag it.

## Findings format
- Rank: **P0** blocks ship, **P1** fix this round, **P2** later.
- Evidence label on every finding: **observed in app** / **code inspection** / **untested claim**.
