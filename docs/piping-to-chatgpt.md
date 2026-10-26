# Piping Stride's data through ChatGPT

**The short answer: yes, it's MCP.** ChatGPT apps *are* MCP servers (the Apps
SDK is built on MCP). But MCP is only the protocol — the design decision is
**where the MCP server runs** and **how ChatGPT authenticates to it**, and those
two answers are where "the data stays on your device" is won or lost.

There are four directions, and only some of them keep our promise.

---

## The mechanics, as they stand today

### Building a ChatGPT app (a "plugin")

- Build an MCP server with the official SDKs — `@modelcontextprotocol/sdk`
  (TypeScript) or `mcp` (Python).
- Transport: **MCP streamable HTTP**, at a stable URL typically ending in `/mcp`.
- Test locally with the MCP Inspector
  (`npx @modelcontextprotocol/inspector`, choose *Streamable HTTP*).
- Optional: custom UI inside ChatGPT, and events.
- Submit for review; the endpoint must stay reachable for review and domain
  verification.

### The authentication rules (the part that matters)

For anything that reads user-specific data, the MCP server must implement
**OAuth 2.1** per the MCP authorization spec:

| Requirement | Detail |
|---|---|
| Protected resource metadata | `GET https://…/.well-known/oauth-protected-resource` |
| Authorization server metadata | `/.well-known/oauth-authorization-server` or OIDC `/.well-known/openid-configuration` |
| Echo `resource` | ChatGPT appends `resource=…` to authorize **and** token requests; bind it into the token's `aud` |
| PKCE | `code_challenge_methods_supported` **must** include `S256` |
| Client registration | CIMD (preferred), DCR, or a predefined OAuth client |
| Client identification | ChatGPT presents an **OpenAI-managed client certificate** → verify with **mTLS**, plus IP allowlists |
| Redirect URI | `https://chatgpt.com/connector_platform_oauth_redirect` (stable) or the callback-ID-specific one |
| Issuer identification | RFC 9207 — return `iss` on every response if you advertise support |
| Tool auth policy | per-tool `securitySchemes`: `noauth` and/or `oauth2` with scopes |
| Triggering the linking UI | metadata **and** runtime errors carrying `_meta["mcp/www_authenticate"]` |

Read-only, anonymous tools are allowed — *"Many plugin MCP servers can operate
in a read-only, anonymous mode."* But ours would not be anonymous, because it
would read a private athlete's history.

### The one rule that decides everything for us

> *"For public plugin submission, deploy the MCP server at a stable, publicly
> reachable HTTPS endpoint. Secure MCP Tunnel can connect a private MCP server
> in ChatGPT, but **it does not satisfy public submission requirements**."*

**A published ChatGPT app must be reachable from OpenAI's cloud.** That is the
fork in the road.

---

## The four directions

### 1 — Published ChatGPT app (public)

```
ChatGPT cloud ──HTTPS──> our MCP server ──> our database
```

- The athlete's data must be reachable by OpenAI's servers, so it lives with
  **us**, or we proxy to their device (which must be online).
- This is a **multi-user SaaS**: identity broker, per-user authorisation,
  billing, deletion requests, and a data-protection story.
- It contradicts our premise. "The data is yours alone" becomes "the data is
  yours, stored by us, reachable by a model provider."
- It is also the most *useful* version for an athlete who wants to ask
  ChatGPT a question without installing anything.

### 2 — Private: the athlete's own device, reached by ChatGPT

```
ChatGPT cloud ──Secure MCP Tunnel──> MCP server on the athlete's laptop/phone
```

- `openai/tunnel-client` opens the tunnel; the server is referenced by
  `tunnel_id`. **No port is exposed to the internet.**
- The data never leaves the athlete's machine and never touches our servers.
- The cost: our MCP server still has to authenticate the *athlete* to
  ChatGPT, which needs an OAuth authorisation server. The honest version of
  this is a tiny **identity broker** we operate that issues tokens for the
  athlete's own server — it holds **no data**, only identity. A broker is a
  much smaller concession than a data store.
- Not eligible for public submission — which is fine, because it is not meant
  to be public. It is one athlete, one device.
- Android caveat: the phone must be **awake and reachable** for the tunnel to
  answer. Background execution on Android fights the OS. This is a
  desktop-first shape.

### 3 — The athlete's own harness (Claude Code / Codex / Gemini CLI)

```
their CLI ──stdio──> our MCP sidecar ──loopback HTTP──> our local API
```

- Already designed in `docs/harness-integration.md`. **No server, no tunnel,
  no OpenAI.**
- The customer owns the harness and the subscription. We are the data source.
- This is the shape that needs **zero new infrastructure**, and it is the one
  our principles point at.

### 4 — Our app calls the model (the in-app assistant)

```
our app ──Responses API──> model ──tool call──> our local MCP server
```

- In the Responses API, MCP is a *tool type*:
  `{type: "mcp", server_url: …}` for a remote server, or `{type: "mcp",
  tunnel_id: …}` for a local one through Secure MCP Tunnel.
- `require_approval` defaults to `"always"` for MCP tool calls — good, and it
  matches our rule that a question never implies an action.
- This is the BYOK assistant: the athlete's own key, their own subscription,
  their data reaching their chosen provider. Data **does** leave the device
  here — that is the trade we would be making deliberately.

---

## Comparison

| | Data location | Needs a server | Identity | Fits our principles |
|---|---|---|---|---|
| **1. Published app** | ours | yes, substantial | OAuth + our IdP | **no** |
| **2. Private tunnel** | the athlete's device | a small identity broker | OAuth (broker issues tokens) | mostly |
| **3. Own harness** | the athlete's device | no | local pairing token | **yes** |
| **4. In-app assistant** | device → their provider | no (or a broker for a remote server) | BYOK | with a stated trade |

---

## What I would build, in order

**First: Direction 3.** The MCP sidecar we already designed, made to work for
the desktop app and the Android app's local server. Zero new infrastructure,
zero new trust relationships, and it satisfies the "customer owns the harness"
principle completely.

**Then, if the athlete wants *ChatGPT specifically*: Direction 2.** Their device
hosts the MCP server; the tunnel exposes it; ChatGPT connects. We run an
identity broker and nothing else — no data, no profiles, no retention.

**Direction 4** as a later, explicitly-labelled option: "send this run to my
provider" with a per-call approval, because that one *does* move data off the
device and the athlete should know it.

**Direction 1 only if we ever want a product for other people** — and that is a
different company, with a data-protection officer and a deletion workflow.

---

## The risks, stated plainly

OpenAI's own documentation warns:

> *"It is very important that developers trust any remote MCP server they use
> with the Responses API. A malicious server can exfiltrate sensitive data from
> anything that enters the model's context."*

The reciprocal risk is ours: **anything we return to a model enters a context
we do not control.** So the rules we already wrote down are not optional
decoration, they are the security boundary:

- read-only by default; write tools absent unless deliberately enabled;
- every tool call confirmed per change, never inferred from a question;
- an audit trail: actor, timestamp, before/after, reason, undo token;
- GPS tracks and raw heart-rate streams are **not** returned unless explicitly
  requested — a route tells someone where you live;
- and every response says what it was derived from, and what is missing.

One more thing worth saying plainly: **revoking access cannot un-send the
data.** If a run has been returned to a model, that text may sit in the
provider's logs and in the athlete's chat history. The app can explain that; it
cannot undo it. That is the honest version of "share", and it is why the
default must be local.

---

## Sources

- [Build an MCP server (ChatGPT plugins)](https://developers.openai.com/plugins/build/mcp-server)
- [Authenticate users](https://developers.openai.com/plugins/build/auth)
- [MCP servers and Secure MCP Tunnel](https://developers.openai.com/api/docs/guides/tools-connectors-mcp)
- [Secure MCP Tunnel](https://developers.openai.com/api/docs/guides/secure-mcp-tunnels)
- [MCP authorization specification](https://modelcontextprotocol.io/specification/2025-11-25/basic/authorization)
- Internal: `docs/harness-integration.md`, `docs/android-v1.md`, `docs/ai-sign-in-discussion.md`
