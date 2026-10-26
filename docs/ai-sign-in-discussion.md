# Discussion: should the Android app offer "Sign in with ChatGPT"?

**Status:** discussion only. Nothing implemented. No code changed.

---

## 0. Why this is a better question than it looks

"Sign in with ChatGPT" is not one decision. It is at least four, and they
collapse differently against our principles. Separating them is most of the
answer.

First, the facts, as they stand today.

**OpenAI's "Sign in with ChatGPT" (SIWC) is real.** It is OpenID Connect based,
OAuth 2.0 Authorization Code with PKCE.

- *Identity-only clients* request `openid profile email` and receive an **ID
  token** for sign-in.
- Clients that also want to spend the user's ChatGPT plan request
  `offline_access resource.invoke chatgpt.tokens.use.direct` and get an
  **access token for inference**. That last scope is the one that spends
  money.
- SIWC does **not** replace an application's own OAuth authorisation server —
  it identifies the user *within* a flow you already own.
- There are published [SIWC Terms](https://openai.com/policies/sign-in-with-chatgpt-terms)
  (notably: the integration must not create a security vulnerability, and must
  not mislead users) and [Brand Guidelines](https://openai.com/brand) governing
  the logo and naming.
- OpenAI has been gauging developer interest in letting third-party apps
  authenticate with a ChatGPT account.

**ChatGPT speaks MCP.** Through the Apps SDK (and the newer plugins model), a
ChatGPT app *is* an MCP server. And — the part that matters for us —
[Secure MCP Tunnel](https://developers.openai.com/api/docs/guides/secure-mcp-tunnels)
(`tunnel_id`, via `openai/tunnel-client`) lets a **local** MCP server be reached
without exposing a port to the internet.

---

## 1. Our principles, stated plainly

These are the ones we have actually built, not aspirations:

1. **Local-first.** The data lives on the athlete's device. The app works with
   no account, no server, no sign-up.
2. **No third-party requests in the default build.** Verified: the browser makes
   zero third-party requests. Fonts are self-hosted, map tiles are proxied by
   our own server.
3. **One credential.** The Garmin Personal API key, wrapped by the Android
   Keystore, never readable by JavaScript.
4. **Sharing is Garmin's business.** We never publish. Every activity links
   back to its original in Garmin Connect, and Garmin decides who may open it.
5. **The customer owns the harness.** Their Claude Code / Codex / Gemini
   subscription, their machine, their harness. We are the data source, not the
   analyst.
6. **Explicit, reversible, attributed actions.** Nothing is inferred from
   "what should I do?"; writes are confirmed, attributed and undoable.

---

## 2. The four readings, judged

### A. "Sign in with ChatGPT" as the app's own sign-in button

*Technical shape:* OIDC + PKCE against OpenAI; identity scopes `openid profile
email`; an ID token proving who the athlete is; store it in the Keystore.

**Verdict: no — and the reason is structural, not stylistic.**

**SIWC solves a problem we deliberately do not have.** It exists to tell a
server "who is this person?" Our app has **no server and no account**. Every
number in it — your runs, your sleep, your plan — is already on the device and
already belongs to you. There is nothing for an identity token to unlock.

Adding it would:

- break the **"no account to sign up for"** promise that the Settings page and
  the README currently make in plain words;
- add the **first account relationship** to a product whose whole claim is that
  it has none — and an account relationship with a model provider is a much
  larger surface than the scoped, single-purpose Garmin key we hold today;
- buy **no functionality**, because there is no server-side resource to
  allocate to the identity;
- cost real work on Android: Chrome Custom Tabs, an app-link redirect, Keystore
  handling of the ID token, Play Store OAuth policy (never lock users into one
  provider), and brand-guidelines compliance for the button;
- and it would be the **single most damaging thing we could add** to a
  privacy-first product. "Sign in with ChatGPT" is a distribution move. It says
  "one tap and you're in" — and it says it by creating a cloud identity for a
  product whose entire value proposition is that it does not need one.

There is a narrower version of A worth naming: request the *identity-only*
scopes and use the ID token purely as a local "which of my own profiles is
this" label. But we do not have multiple profiles, and a local label is worth
nothing that a local preference is not worth.

**A cleaner rule:** if a feature needs to know who you are, it probably belongs
to a different product.

### B. "Ask ChatGPT about this run" *inside* the app

*Technical shape:* the athlete pastes their own OpenAI (or Anthropic, or
Google) key into the Keystore, and the app calls the provider directly with the
run context the athlete selected.

**Verdict: this is worth doing — but as bring-your-own-key, never as "sign in
with ChatGPT".**

BYOK is the shape that fits our principles exactly:

- no account relationship with **us**;
- the customer already owns the subscription;
- the key lives in the Keystore and never reaches JavaScript — the same
  boundary we already drew for the Garmin key;
- **nothing is sent unless the athlete asks**, and the request is attributable;
- we can keep the strict rule we already wrote down for the harness: a
  question never implies a write.

The difference from SIWC is not cosmetic. SIWC gives OpenAI an identity for
the athlete. BYOK gives the athlete a key for the provider they chose. One is
an account; the other is a tool. Our app is a tool.

The cost is that an in-app assistant is a **large** surface, and it should be
v2 at the earliest: the tool set, the per-call approval flow, the audit trail,
and the "a question is never an instruction" rule. That work is already
specified in `docs/harness-integration.md` — this would be the same work aimed
inward instead of outward.

### C. ChatGPT calls Stride (the athlete asks ChatGPT; ChatGPT queries the phone)

*Technical shape:* the phone runs our MCP server; OpenAI's Secure MCP Tunnel
exposes it without opening a port; a ChatGPT app (Apps SDK / plugin) consumes
it.

**Verdict: possible, and it inverts one of our principles — so it must be an
explicit choice, not a default.**

What it gets right: the same MCP tool set we already designed, the same
read-only-by-default rules, the same explicit-approval model, and the athlete
stays in control of what is exposed.

What it changes: **the customer no longer owns the harness.** The question goes
to OpenAI, on OpenAI's infrastructure, under OpenAI's retention. That may be
exactly what some athletes want — it is a legitimate product choice — but it is
a *different product* from the one whose README says "there is no feed, no
followers, no comments, and there is not going to be one: a social layer would
mean the data stopped being yours alone."

There is also a hard Android problem: the phone must be **awake and reachable**
for the tunnel to answer. Background execution on Android fights the OS. This
is not a v1 feature.

If we ever do it, the shape should be: the athlete explicitly connects the app
to ChatGPT, sees exactly which data would be exposed, read-only tools only,
writes off by default, and can disconnect. And it should be **one of several
harnesses**, never the only one — because principle 5 says the customer owns
the harness, and OpenAI owning it is a decision we would be handing away.

### D. "Add to ChatGPT" / share a run into a ChatGPT conversation

**Verdict: no.** Same reason we removed the social layer. We do not publish.
Sharing a run is Garmin's job, and Garmin's link already carries it.

---

## 3. The recommendation

**Do not add "Sign in with ChatGPT" to the Android app, in any form.**

- As an app sign-in (A): it buys nothing, breaks the no-account promise, and
  creates the first account relationship in a product whose premise is not
  having one.
- As share-to-ChatGPT (D): it is a publishing surface, and we do not publish.
- As BYOK inside the app (B): **do this**, but call it what it is — *your key,
  your subscription, your call* — and keep it out of v1.
- As a ChatGPT app the phone serves (C): **later, and only if asked for**,
  read-only, explicitly connected, revocable, and never the only harness.

The decision is actually **downstream**: SIWC only makes sense if we decide to
operate a *server-side account*. We have decided not to. So the question does
not need answering on its own — it dissolves.

---

## 4. The symmetry worth noticing

The nicest thing to come out of this discussion is that the Android app and the
desktop app can share one design.

On the desktop we already planned an **MCP sidecar** so the athlete's own
Claude Code / Codex / Gemini CLI can query their training data. On the phone,
the same MCP server can run locally and be reached by that same harness over
the LAN (with the same capability token and explicit pairing).

So the athlete's laptop-side agent asks a question, the answer comes from the
phone's database, and nothing was published and nothing was copied to a
provider's servers. The phone is the data source; the athlete's own harness is
the analyst; both ends belong to the athlete.

That is the version of "sign in with ChatGPT" that is actually worth having —
and notice it contains no sign-in at all.

---

## 5. What we would need to decide before building any of it

1. **Does the feature need an identity?** If yes, we have the wrong product.
   If no, BYOK.
2. **What is the data-flow?** Every byte that leaves the device should be
   named, justified, and revocable.
3. **Who owns the harness?** If it is the customer's, BYOK or a local MCP
   server. If it is a vendor's, we have changed the product.
4. **What is the v1 scope?** The Android app's first milestone is: install,
   paste the Garmin key natively, pull 30 days, browse an activity offline in
   airplane mode. An assistant is not on that list, and adding it would put
   the largest possible surface on the least-verified part of the build.

---

## Sources

- [Sign in with ChatGPT — quickstart](https://developers.openai.com/siwc/quickstart)
- [SIWC registration and sign-in](https://developers.openai.com/siwc/token-sharing-open-source/sign-in)
- [SIWC terms](https://openai.com/policies/sign-in-with-chatgpt-terms)
- [OpenAI brand guidelines](https://openai.com/brand)
- [MCP servers and Secure MCP Tunnel](https://developers.openai.com/api/docs/guides/tools-connectors-mcp)
- Internal: `docs/harness-integration.md`, `docs/android-v1.md`, `docs/security/CHECKLIST.md`
