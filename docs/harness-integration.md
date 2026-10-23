# Stride harness integration

**Status:** design proposal, 23 October 2026  
**Scope:** local MCP interface for Claude Code, Codex and Gemini CLI using the athlete's existing harness subscription.

## Product contract

The athlete owns and runs the coding harness. Stride is its data source, not an AI proxy: it stores no model key, makes no model calls, and never forwards data to a model vendor. MCP returns only the athlete's Stride data in response to explicit tool calls. The harness vendor may process whatever the athlete chooses to submit to its model; local MCP cannot guarantee how a harness's cloud model handles prompt content. The product can guarantee it sends no background data and exposes no Stride credentials to the model.

The existing API is FastAPI under `/api/v1`. It already supports activity list/detail (including laps, streams, HR zones and GPS), stats summary/weekly/monthly/pace trends/zones/records, plan current/week/results/adherence/today, physiology daily/latest/sleep/HRV/readiness, fingerprint/trend/zones/gear, and multi-run comparison. The analytics and comparison services implement useful calculations, but some statistics are estimates (notably HR zones when Garmin zones are unavailable). MCP should preserve provenance and availability markers rather than imply every value is authoritative.

## Tool set

Use question-shaped aggregates for the common first call, plus a small number of focused raw tools. All inputs use JSON Schema; dates are ISO `YYYY-MM-DD`, distances are metres unless a field says otherwise, and ranges are inclusive. Outputs are JSON objects with `data`, `coverage`, and `limitations` where applicable. `coverage` identifies dates and source fields present; absent values are `null`/empty and never silently treated as zero. Each tool is read-only unless marked **opt-in write**.

| Tool | Purpose and inputs | Output shape | Example prompt |
|---|---|---|---|
| `training_digest` | One-call overview. `{period: "7d"|"28d"|"block"|"custom", from?: date, to?: date}` | `{period, load:{distance_m, runs, elevation_m, weekly_series}, change_vs_previous:{...}, plan:{planned_km, actual_km, adherence_pct, misses:[{session,cause,confidence}]}, physiology:{readiness,drivers,sleep,hrv,rhr}, fingerprint_trends:{...}, records:[...], coverage, limitations}`. Sections can be null where unavailable. | “How is my training going this month? What should I pay attention to?” |
| `search_activities` | Filter/paginate: `{from?,to?,sport?,min_distance_m?,max_distance_m?,q?,sort?:"date"|"distance"|"pace"|"elevation",limit?:1..100,cursor?}`. Defaults to running activities; `sport:"all"` includes all sports. | `{items:[activity summary],next_cursor,total_estimate,filters}` | “Find my hilliest runs over 15 km since June.” |
| `get_activity` | Full one-run detail: `{activity_id,include?:["laps","streams","zones","route"]}`; default laps and zones, streams/route explicitly requested due to size/sensitive location. | `{activity, laps?, streams?:{kind:{unit,samples:[{t,value}]}},zones?,route?,coverage}` | “Show the lap pace, HR and elevation for last Saturday’s race.” |
| `activity_analysis` | Explain a run or race in one call: `{activity_id,compare_with?:"similar_recent"|"previous_best"|"activity_id:<id>",focus?:["pacing","hills","effort","form"]}` | `{run,segments:[{distance_m,pace,hr,elevation_gain,grade,notes}],comparison?,signals:[{finding,evidence,confidence}],missing_data}`. Segments derive from laps/streams; do not label terrain effects causal absent grade/route evidence. | “I raced last weekend and felt like I died on the hills—what actually happened?” |
| `training_statistics` | Aggregate trends: `{from?,to?,group_by?:"week"|"month",metrics?:["volume","pace","zones","records","elevation"]}` | `{totals,series:[{period,run_count,distance_m,pace,hr,elevation}],hr_zones:[{zone,time_s,method}],records:[...],coverage}`. Zone `method` says athlete-configured vs estimated. | “Is my easy pace improving over the last 12 weeks?” |
| `plan_context` | Plan plus execution/cause: `{view:"current"|"week"|"block"|"results",date?:date,block_id?:int,from?:date,to?:date}` | `{has_plan,block,phase,week:{planned_km,actual_km,sessions:[{target,activity,verdict,cause,cause_confidence}]},adherence,coverage}`. `today` recommendation is returned as a suggestion with evidence, not a command. | “I’m behind the 42 km target this week; what was missed and why?” |
| `physiology_context` | Readiness and wellness: `{from?,to?,focus?:"readiness"|"sleep"|"hrv"|"resting_hr"|"all"}` | `{days:[{date,readiness,drivers,sleep:{...},hrv:{value,status,baseline},resting_hr:{value,baseline},source}],coverage,missing_sources}` | “Was my low readiness after Tuesday’s workout unusual?” |
| `fingerprint_trends` | Mechanics and trends: `{from?,to?,metrics?:["cadence_stability","cadence_drift","decoupling","efficiency","pace_variability","negative_split","stride_length","zones"],group_by?:"run"|"week"|"long_run"}` | `{series:[{period,n,metrics:{median,trend},sample_activity_ids}],coverage,limitations}` | “Has my aerobic decoupling improved during this block, especially on long runs?” |
| `compare_runs` | Put 2–5 known runs on a shared distance axis: `{activity_ids:[int,...],points?:20..240,metrics?:[...]}` | `{runs:[{id,name,date,values,profile:[{distance_m,pct,pace_s_per_km,hr,cadence,elevation}]}],parameters,missing,coverage}`. Reject duplicate IDs; disclose incomplete stream coverage. | “Compare my two most recent 10K efforts mile by mile.” |
| `update_activity_notes` **opt-in write** | Athlete-approved edits only: `{activity_id,description?:string,perceived_exertion?:1..10,confirm:true,reason?:string}`. Not exposed unless athlete enables writes. | `{activity_id,changed_fields,before,after,actor,undo_token}` | “Set yesterday’s perceived exertion to 8 and add ‘windy final climb’ to the notes.” |
| `edit_plan` **opt-in write** | Narrow plan changes: `{operation:"adjust_session"|"skip_session"|"change_target",session_id,fields,confirm:true,reason}`. Never invent or apply a proposed adaptation without a separate confirmation. | `{change_id,before,after,actor,undo_token}` | “Move Thursday’s easy run to Friday; keep the rest of the week unchanged.” |

The names above are MCP tool names; schema definitions should be strict (`additionalProperties:false`, enums/ranges) and descriptions should say what is not inferred. MCP annotations should mark reads as `readOnlyHint:true`; write tools must be visibly marked destructive/write-capable and require an explicit confirmation field plus client approval. Prefer stable IDs, not names, for writes. Return bounded stream samples (for example, downsample to at most 240 points per requested series); the full activity detail stays available for targeted follow-up.

### Why these boundaries

`training_digest`, `activity_analysis`, and `plan_context` answer high-frequency questions in one request. `get_activity`, `physiology_context`, and `fingerprint_trends` provide exact follow-up evidence. `compare_runs` wraps Stride's existing distance-normalised comparison logic. Avoid a one-tool-per-endpoint mirror: it forces the model to discover joins and burns calls. Avoid an all-data dump: it wastes context and exposes route/location detail when irrelevant.

## Runtime and transport

**Ship a separate, small sidecar process, launched by the harness over MCP stdio.** It is isolated from the FastAPI event loop and does not need to import the backend's ORM or pin itself to its internals. Stdio is supported by the target local coding harnesses, has no listening port to expose, and lets the harness own lifecycle and consent. A remote HTTP MCP server would make the athlete's data reachable outside the machine and undercut the local-first promise. An in-process MCP endpoint would couple the API lifecycle and auth/security boundary to a particular server runtime.

For the current desktop app, the sidecar calls Stride's existing HTTP API at `http://127.0.0.1:8000/api/v1`, using a short-lived local pairing token minted by the app's Settings screen. Do not assume the README's VM/Tailscale `0.0.0.0` deployment is private: local MCP must reject non-loopback API destinations and must not inherit arbitrary proxy settings. The API validates the token and scopes it to read-only endpoints by default. If the current API has no auth for local calls, add a dedicated loopback-only capability at the integration boundary before shipping; a port number alone is not authentication. The sidecar has a fixed endpoint allowlist, no generic URL/fetch tool, no shell tool and no file-reading tool.

The phone path should keep the same MCP tools and stdio contract while changing the data adapter: Stride mobile app owns the data and exposes a paired, authenticated local interface reachable only on a user-approved LAN/VPN path, or the phone runs the sidecar itself where supported. Pairing is explicit, revocable and scoped; client authentication is mutual or capability-based. There is no cloud relay in the default design. Mobile OS background execution, iOS local service constraints and whether each harness can reach a phone over a private tunnel were not verified; treat phone support as a later adapter, not a property of the desktop stdio transport.

The sidecar should be implemented in a portable runtime available on macOS, Linux and Windows. Use stdio as transport across all three; avoid shell-specific path assumptions, shell interpolation, OS-specific keychain assumptions without an abstraction, and machine-wide port exposure. This document is a design; no server implementation or platform compatibility test was performed.

## Harness configuration

The examples use the documented local stdio server shape: executable plus args. Install a signed Stride sidecar binary or run a pinned package entry point; do not put a Garmin key, model key or Stride pairing secret in the config. The sidecar obtains its scoped local capability through Stride's pairing flow/keychain. Paths below are illustrative (`/path/to/stride-mcp`); Windows users substitute the installed executable path.

**Claude Code** (`~/.claude.json` user scope, or project `.mcp.json` for a project-local server):

```json
{
  "mcpServers": {
    "stride": {
      "type": "stdio",
      "command": "/path/to/stride-mcp",
      "args": ["serve", "--api", "http://127.0.0.1:8000/api/v1"]
    }
  }
}
```

Alternatively, `claude mcp add --transport stdio stride -- /path/to/stride-mcp serve --api http://127.0.0.1:8000/api/v1`; confirm with `claude mcp list` and `/mcp`. Anthropic documents MCP setup and the CLI management flow; exact storage location may vary by scope/version, so the CLI is preferred over hand-editing.

**Codex** (`~/.codex/config.toml`):

```toml
[mcp_servers.stride]
command = "/path/to/stride-mcp"
args = ["serve", "--api", "http://127.0.0.1:8000/api/v1"]
```

Then run `codex mcp list`. Codex also has `codex mcp add` commands; use its CLI to inspect the installed version's generated entry.

**Gemini CLI**: unverified. The intended portable configuration is the documented-by-that-version equivalent of a local stdio MCP server entry; this design does not assert a filename or schema because I could not verify current Gemini CLI MCP configuration from an official primary source during this research. Before publishing a Gemini snippet, check the current official Gemini CLI MCP documentation and test against its released CLI. Do not copy another client's JSON shape by assumption.

Configuration docs checked **23 October 2026**: [Anthropic Claude Code MCP documentation](https://docs.anthropic.com/en/docs/claude-code/mcp) (the page URL returned 404 to the fetcher; the official [Claude Code CLI reference](https://docs.anthropic.com/en/docs/claude-code/cli-usage) confirms the `claude mcp` command but not the complete JSON schema in the fetched excerpt); [OpenAI Codex MCP setup](https://developers.openai.com/learn/docs-mcp) (documents `~/.codex/config.toml`, `[mcp_servers.<name>]`, and `codex mcp add`). The Claude JSON fields above follow the current Claude Code MCP stdio shape as understood, but the official MCP page was not retrievable in this run, so verify field-level details against the installed Claude Code version before release. No Gemini CLI official config source was successfully verified.

## Access, safety and revocation rules

1. **Read scope:** tools may return only information that the athlete's own Stride app/API can read for that paired profile. The sidecar does not query Garmin directly and cannot enumerate other Stride users.
2. **Confinement:** no network egress from the sidecar other than calls to the configured loopback Stride API; no arbitrary URLs, filesystem access, shell execution or other services. Enforce this in code and packaging, not merely in a system prompt. The harness itself may have wider permissions; Stride cannot truthfully guarantee the model process has no other network/filesystem access. Keep Stride's MCP process separately constrained and tell the athlete the distinction.
3. **Secrets:** Garmin keys, OAuth tokens and refresh tokens stay in Stride's OS keychain/secure store. The sidecar never receives, returns or logs them. Harness subscription credentials remain in the harness. The pairing capability is stored in OS secure storage, scoped, expiring and revocable.
4. **Default read-only:** writes are absent unless the athlete deliberately enables them in Stride settings. When enabled, only the listed narrow tools appear. Every call requires explicit per-change confirmation and client approval; no write can be inferred from “what should I do?”
5. **Attribution and undo:** record actor (`local MCP`, paired client identity), timestamp, fields, before/after and reason in an append-only audit record. Return an undo token and provide an app-side undo action. A reversal is another attributed event, not deletion of history. Plan edits must preserve the prior version.
6. **Revocation:** Settings → Connected harnesses → Revoke immediately invalidates that capability, removes the local client pairing, and makes subsequent API requests fail. The athlete can also disable the integration or remove the config. Existing harness conversation context cannot be remotely erased by Stride; the app should explain that previously returned text may remain in that harness's history. Revoke does not delete the athlete's Stride data or disconnect Garmin.
7. **Sensitive location:** route/GPS streams are not returned unless explicitly requested. Logs contain tool names, latency and status only; never tool arguments, values, route points or credentials.
8. **Honest evidence:** include source, date range, sample count and method for derived metrics; disclose missing Garmin wellness, missing streams, estimated HR zones, small samples, and uncertain plan-miss causes. Separate correlation from causation and present recommendations as suggestions, not medical advice.

## Worked examples

### 1. “I raced last weekend and felt like I died on the hills — what actually happened?”

**Trajectory:** `search_activities({from,to,sport:"running",q:"race"})` → `activity_analysis({activity_id,focus:["pacing","hills","effort"],compare_with:"similar_recent"})` → if the route/grade or HR trace has gaps, `get_activity({activity_id,include:["laps","streams","route"]})` for the relevant evidence. **Answer shape:** summarize where pace/HR changed relative to climbs and descents, quantify lap/grade/effort differences, compare only if duration, distance and terrain are reasonably comparable, and name missing GPS/HR evidence. If the activity has no grade or stream data, say the data cannot establish that hills caused the fade; report only observable pace/HR changes.

### 2. “Am I getting fitter or just more tired? Compare this block to the last one.”

**Trajectory:** `plan_context({view:"current"})` → `training_digest({period:"block"})` → `fingerprint_trends({from:block_start,to:block_end,metrics:["decoupling","efficiency","cadence_stability"],group_by:"long_run"})` and equivalent prior-block range → `physiology_context({from:prior_start,to:current_end,focus:"all"})`; use `training_statistics` if block volume or pace needs a matched summary. **Answer shape:** show block-to-block volume, comparable-run pace/HR efficiency and long-run decoupling, then readiness/sleep/HRV/RHR direction and sample sizes. State “fitness likely improving / fatigue signals rising / mixed / insufficient evidence,” with evidence and caveats. If blocks differ in terrain, workout mix, data coverage or have too few comparable runs, do not call a pace change fitness.

### 3. “My plan says 42 km this week and I’m behind — should I make up the distance?”

**Trajectory:** `plan_context({view:"week",date:today})` → `training_digest({period:"7d"})` → `physiology_context({from:week_start,to:today,focus:"readiness"})` → `fingerprint_trends({from:week_start,to:today,metrics:["decoupling","cadence_stability"]})` only if enough runs exist. **Answer shape:** report planned vs completed km, sessions missed and recorded causes/confidence, remaining days and readiness context; give a conservative suggestion framed around the next planned session and recovery, not an automatic catch-up prescription. If plan or physiology coverage is missing, say so and avoid recommending a mileage increase. No plan edit occurs without a separate explicit write confirmation.

## Recommendation

Start desktop with the read-only stdio sidecar and the aggregate-first tools. Add the local pairing capability and audit/undo design before exposing it to real athlete data; do not advertise “no vendor data sharing” as an absolute because the harness may send selected tool results to its model provider as part of answering.
