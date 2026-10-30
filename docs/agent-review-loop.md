# Agent review loop (pilot)

How two AI agents collaborate on Stride, with Sumit as product owner.

## Roles
- **Hark (coder):** implements changes, opens PRs, replies to findings.
- **Instinct (tester):** reviews usability and coach persona, files ranked findings.
- **Sumit (owner):** sets each round's goal and volley limit, breaks ties, approves merges.

## Round protocol
1. Sumit names the round: goal + volley limit (default 3).
2. The coder opens or updates a draft PR and emails a short handoff: PR link, round number, what to test.
3. The reviewer reads the PR, comments on it, and emails the next handoff.
4. Each finished turn ends with one marker in the email subject: `DONE` (your move) or `SHIP` (no blocking findings).
5. The round stops at the volley limit, on `SHIP`, or after 30 quiet minutes. Sumit gets one summary: shipped, disputed, needs his call.

## Rules
- Notify on finished turns only, never on edits.
- Every handoff carries: the ask, the link, done-when.
- Disagreements: two lines from each side; Sumit decides.
- **This repo is public.** Comments hold code and product findings and synthetic examples only. Never real running, health, or Garmin data.

## Persona and usability checklist (reviewer)
- Does the copy sound like a calm coach a runner trusts, not a nag or a cheerleader?
- Is the next action obvious within five seconds on the Today screen?
- Are numbers explained in plain words (why it matters, what to do tomorrow)?
- Does a missed day or a bad run get a forgiving, practical response?
- Is anything shown that the user can't act on? Flag it.
- Rank findings: P0 blocks ship, P1 fix this round, P2 later.
