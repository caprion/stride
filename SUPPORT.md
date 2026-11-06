# Support

**Issues are welcome. Features are the maintainer's call.**

This is one athlete's project, maintained by one person. There is no company behind it, no support
contract, and no service level. What that means in practice:

## What you can expect

- **Bugs will be looked at.** A reproducible report — what you did, what you expected, what happened,
  and the version — will get a response.
- **A clear answer about the roadmap.** If a feature is planned, it is in
  [PLAN.md](PLAN.md). If it is not, the answer is "not planned", and that is a complete answer.
- **Credit.** Contributors are credited in the repository history, and in the release notes where a change
  is user-visible.

## What you cannot expect

- **A response time.** There is no SLA and no on-call. A report may sit for weeks; that is not neglect,
  it is one person.
- **Features on request.** "I would use this if it did X" is welcome as an idea and will usually be
  declined as a request. The scope is set in [PLAN.md](PLAN.md), and it is kept small on purpose: the
  Android is the primary release target and desktop hosts the same responsive UI.
- **A managed hosting or account service.** None is provided. The current browser
  app needs its local Rust host; no standalone PWA is delivered by this release.
- **Commercial support.** The licence is AGPL-3.0; if you need something the project does not do, you are
  welcome to build it yourself.

## Asking for help

1. **Check the plan** — the design is specified in [PLAN.md](PLAN.md) and
   its approved amendments. The [current media gallery](docs/media/README.md)
   shows the implemented screens; `docs/plan.html` preserves the original design.
2. **Search the issues** — including closed ones.
3. **Open an issue** with the version, platform, steps and expected result.
   Do not attach real routes, health records, credentials, backups or database files.
   Use synthetic examples or a redacted description.

## The intake

Three templates, one queue. Each asks for what it needs and nothing else, and
each is labelled on arrival — so a report that used one is already sorted.

| | template | what it is for |
|---|---|---|
| **Bug** | [open a bug report](https://github.com/caprion/stride/issues/new?template=1-bug_report.yml) | Something did not do what it should have. |
| **Feedback** | [give feedback](https://github.com/caprion/stride/issues/new?template=2-feedback.yml) | What works, what does not, and what it should have said instead. |
| **Request** | [make a request](https://github.com/caprion/stride/issues/new?template=3-feature_request.yml) | Something you want the app to do. The scope is the maintainer's call, and "not planned" is a complete answer. |

The version and the platform are asked for because they are usually the cause.
Redact before you attach anything: real routes, health records, credentials,
backups and database files do not belong in a public tracker.

## Linking to it from the app

The template names and the field ids are public and stable, so a link from
inside the app can open one of them directly and pre-fill what the app
already knows:

```
https://github.com/caprion/stride/issues/new?template=2-feedback.yml&version=1.0.0&area=Phone%20app%20(Android)
```

`area`, `version` and `kind` are the ids a report can be pre-filled with;
`area` matches the label of the option exactly, so the spaces are encoded.
The app knows the version and the surface it is running on, so a report that
starts in the app arrives with the two facts that matter already filled in,
and the person typing it is left with the part only they can write.

## Reporting a security problem

Do not open a public issue. Use
[private vulnerability reporting](SECURITY.md#reporting-a-vulnerability).
