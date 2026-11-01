# The desktop rig

**One binary** that serves the app and the router, on this machine, against
this machine's own database.

It is not a product. It is three things at once:

* the **development rig** — a fast feedback loop: build the frontend, run the
  binary, look at the app;
* the **community tool** — a stranger clones the repository, runs one
  command, pastes their Garmin key, and sees their own runs;
* the **reference host** — the process the parity differ and the harness talk
  to.

The phone app is the product. The desktop is the host it runs on, and it
never grows a screen of its own: the same four destinations, the same
information architecture, the same router, the same numbers.

## Running it

### What you need

* **Rust** (stable — the repository pins the toolchain in
  `rust-toolchain.toml`), and
* **Node** — once, to build the app. The bundle is compiled into the binary,
  so `cargo run` needs no node afterwards.

Nothing else. There is no server to install, no account to create, and no
process to keep alive besides the one you started.

### The steps

```bash
# 1. get the code
git clone https://github.com/caprion/stride.git
cd stride

# 2. build the app once (the bundle is embedded in the binary)
npm --prefix frontend ci
npm --prefix frontend run build

# 3. run the rig
cargo run -p stride-desktop
```

It prints where it is serving:

```text
stride 0.1.0
the app is on http://127.0.0.1:8080
the database is at /home/you/.local/share/stride/stride.db
the rig answers the loopback, and no other network (STRIDE_TRUSTED_NETWORKS is not set)
```

Open <http://127.0.0.1:8080> and the app is there.

> The rig also runs without step 2: it says so, in a sentence, and tells you
> to run `npm --prefix frontend run build`. It never fails obscurely.

### The commands

| command | what it does |
|---|---|
| `stride-desktop serve` | serve the app and the router |
| `stride-desktop migrate` | bring the database up to the frozen schema |
| `stride-desktop doctor` | say what this rig is and where its files are |

`stride-desktop --help` lists the options: `--bind`, `--port`, `--data-dir`,
`--static-dir`, `--tls-cert`, `--tls-key`, `--allow-plain-http`.

## The trust model

**Three lines, and no more.**

| from | what happens |
|---|---|
| the loopback — `127.0.0.1` or `::1` | always answered. No configuration. |
| a network in `STRIDE_TRUSTED_NETWORKS` | answered. The athlete may want the rig on their phone over their own tailnet. |
| anywhere else | **refused**, with a message that names the variable. |

```bash
# a CIDR list, separated by commas or by whitespace
STRIDE_TRUSTED_NETWORKS=192.168.1.0/24,fd00::/8 cargo run -p stride-desktop
```

A plain address is a network of one: `192.168.1.5` means `192.168.1.5/32`.
A list that cannot be read is refused at start-up, with the text that could
not be read — a rig that silently trusts nothing is a rig that looks broken.

**There are no pairing codes, no QR codes, no device tokens and no
capability cookies.** A rig that is open to a network is a rig whose Garmin
key is on that network, and a pairing dance would be a second, slower way of
saying "trust this network" — one that leaves a token lying around.

### A network is a TLS network

The rig refuses to serve plain HTTP on anything but the loopback:

```bash
stride-desktop serve --bind 0.0.0.0 --tls-cert cert.pem --tls-key key.pem
```

The certificate is yours. The rig does not make one, and it does not ask for
one: a certificate the athlete made is one the phone can be told to trust.
If the network is already yours — a tailnet, for example — `--allow-plain-http`
says so out loud and serves it anyway.

## The keys

There are **two** secrets, and they are not the same secret.

### The Garmin key — one credential, and it stays yours

The Garmin Personal API key is the only thing the athlete ever types. It
lives in one of three places:

| where | how |
|---|---|
| **in memory** | pasted into the connect screen; held for the session |
| **in the environment** | `STRIDE_GARMIN_KEY=…` |
| **in a file** | `~/.local/share/stride/garmin.key`, with `0600` |

The rig **never logs it, never returns it from an endpoint, and never writes
it into the database**. A file that other people can read is refused rather
than read.

### The database key — the rig's own

The database is encrypted with SQLCipher, and the key that opens it is
generated on the first run from the operating system's random bytes:

```text
~/.local/share/stride/stride.db          the database, encrypted, with 0600
~/.local/share/stride/db.key              its key, with 0600
```

It is not derived from the Garmin key, and it does not open the Garmin
key's file.

### The database that predates the encryption

The old desktop wrote a plain SQLite file. The first run of this rig
converts it — once — into an encrypted database with the frozen schema, and
**the plain copy is kept beside it**:

```text
~/.local/share/stride/stride.db                  the encrypted database
~/.local/share/stride/stride.db.plaintext-backup  the plain copy, kept
```

The plain copy is not a leak: the file was plain before the conversion, and
the copy is the way back if the conversion is wrong. **It is yours to
delete** once the encrypted database opens.

## Where the files are

`~/.local/share/stride` on Linux, `%LOCALAPPDATA%\stride` on Windows, and
`$XDG_DATA_HOME/stride` if that is set. `STRIDE_DATA_DIR` overrides it.

## What the rig is not

* **not a second implementation.** Every number comes from `stride-core`,
  every reading from `stride-store`, and every request goes through
  `stride-api`'s router. The rig is a host, not a brain: it opens the door,
  decides who is let in, and serves the app. It does not reshape a
  response.
* **not a product surface.** The four destinations are the phone's four
  destinations. Compare and the Plan week are reached from inside the app,
  not as extra tabs.
* **not a service.** There is no daemon, no scheduler and no cloud. Stop the
  binary and the rig is gone; the data stays in the file.

## The router

The router is `stride-api`'s — one router, two hosts, so the phone and the
desktop cannot disagree about a number. That crate is an empty skeleton
today, so the rig answers through a **stub that refuses rather than
guesses**: every route answers with

> the Rust core is not in this build. Nothing is shown until it is.

A stub can refuse, but it cannot be wrong, and an app that shows a number
nobody computed is a lie about somebody's training. When the router lands,
`desktop/src/api.rs` is the only file that changes.
