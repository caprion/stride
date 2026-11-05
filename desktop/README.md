# Stride on desktop

## 1. What this is

The desktop host is one Rust
binary that serves the same app and the same router as the phone, against a
local encrypted database.

It runs the shared responsive application, supports local development and lets
someone use the same running log without a Stride account or hosted data store.
Android remains the primary release target; desktop is not a second UI.

[Watch the short demo and browse current screenshots](../docs/media/README.md).
All public media uses fictional HR, HRV, sleep and GPS, not personal history.

## 2. Build it and run it

You need Rust (the repository pins the toolchain) and Git. There is no Python
environment, no Node install, no npm command, and no second server process to
keep alive.

```bash
git clone https://github.com/caprion/stride.git
cd stride
cargo run -p stride-desktop
```

That one command builds the rig and starts it. It prints the URL, normally
`http://127.0.0.1:8080`. The app bundle is carried in the repository and
compiled into the binary, so the command does not need a separate frontend
build.

Stop the rig with `Ctrl-C`. The process goes away; the data stays in the
private data directory.

The other commands are:

| command | what it does |
|---|---|
| `stride-desktop serve` | serve the app and the router |
| `stride-desktop migrate` | bring the database up to the frozen schema |
| `stride-desktop doctor` | say what the rig is and where its files are |
| `stride-desktop export` | write a verified export of the local log |
| `stride-desktop import <directory>` | read a verified export back into the local log |

`stride-desktop --help` lists the options.

## 3. Give it a Garmin key

On the first launch, open the URL. The rig opens one standalone **Connect**
page, with no app navigation around it:

1. Create a [Garmin Personal API key](https://developer.garmin.com/gc-developer-program/personal/personal-faq).
2. Paste the **consumer key** into the one field. It is the consumer key, not
   the consumer secret.
3. Choose **Test the key**. The page says **Checking…**, then **Syncing…**,
   then **Connected · 20 runs** (the count is the number actually in the log).
4. The Connect page disappears and the rig opens on **Today**.

If Garmin refuses the key, the page says what to check: that the value is the
Personal API consumer key, that it is active, and that it has not expired. It
never shows a stack trace.

**Not now** opens the app in an honest empty state. It does not create fake
runs or fake wellness data.

The key stays on this machine. After Garmin accepts it, the rig keeps it in
`~/.local/share/stride/garmin.key` (or the platform equivalent), with mode
`0600` on POSIX systems and private inherited permissions elsewhere. The
next launch goes straight to the app.

The key is never put in a URL, a log line, an API response, a fixture, or any
other page. The Connect field is the only surface that shows it, and it is a
password field.

Automation can inject `STRIDE_GARMIN_KEY` or `GARMIN_API_KEY` into the child
process using a secret manager. Do not type a literal credential into a shell
command, `.env`, source file or transcript. An injected environment key is not
persisted by the host. Maintainer-specific vault launchers are not required for
the first-run UI and are not shipped in this repository.

## 4. The trust model and the phone

The trust model is three lines:

| caller | answer | rule |
|---|---|---|
| the loopback — `127.0.0.1` or `::1` | allowed | always; no configuration |
| an address inside a CIDR in `STRIDE_TRUSTED_NETWORKS` | allowed | the peer address must be inside a listed CIDR |
| anything else | refused | the message names `STRIDE_TRUSTED_NETWORKS` and says what to set |

There is no other way in.

To open the rig on a phone over the athlete's own network, trust the
phone's network and give the rig a certificate the phone trusts:

```bash
STRIDE_TRUSTED_NETWORKS=192.168.1.0/24 \
  cargo run -p stride-desktop -- serve \
  --bind 0.0.0.0 \
  --tls-cert /path/to/cert.pem \
  --tls-key /path/to/key.pem
```

Then open `https://<the-machine's-address>:8080` on the phone. The
certificate belongs to the athlete; the rig neither makes one nor asks for
one.

The rig refuses plain HTTP on a network address. If the network is already
private, such as a tailnet, `--allow-plain-http` is an explicit choice and
the rig says in its output that it made that choice.

## 5. Move the data: export and import

Open **Data & connection > Backup, restore and export** to reach the host-owned
tools at <http://127.0.0.1:8080/rig>. The same operations are available from the
command line.

### Export

`stride-desktop export` writes a directory under the rig's private
`exports/` folder and prints the exact path. The **Write an export** action
on the rig-tools page does the same and shows the path.

An export contains:

| file | what it is |
|---|---|
| `manifest.json` | the export format version, database schema version, row counts and SHA-256 digest |
| `stride.db` | the encrypted database |
| `db.key` | a new key made for this export |

The export database contains the athlete's runs (including their original FIT
files), wellness days, plan blocks, planned sessions, notes and sync state.
It does **not** contain the Garmin key.

`db.key` is an export-only key, not the key for the rig's live database. The
folder can therefore open the export without opening the live database. It is
still private data: the key and the athlete's rows travel together.

Keep the whole folder off public storage and move it only over a private
channel. Android has a native passphrase-encrypted recovery archive workflow,
but that ZIP has additional recovery metadata and is not this desktop directory
format. Do not assume the desktop bundle can be opened by the phone's restore
picker. See the [Android format and acceptance boundaries](../android/README.md#backup-restore-and-readable-exports).

### Import an export

The rig reads an export in either of two ways:

- type the export directory on the rig-tools page; or
- drop the export folder on the page's drop target.

`stride-desktop import <directory>` does the same from a shell.

The answer says how many rows were read, the export format version, the
database schema version, the row counts by kind, and anything refused. A
successful import refuses nothing.

The import is a **merge**, not a replacement. Existing rows are updated, new
rows are added, and importing the same bundle twice adds nothing the second
time.

The import refuses the whole bundle when:

- the manifest is missing, unreadable, or not a Stride export;
- the export format version or database schema version is not one this build
  reads;
- the named database is not a plain file inside the export directory;
- the database's size or SHA-256 digest does not match the manifest;
- an included `db.key` is readable by other people; or
- an uploaded export is over 128 MiB.

A refused import changes nothing.

### Import a FIT file

The rig-tools page has a separate drop target for one `.FIT` file. The rig
sends the bytes to the API's FIT-import route; the API decodes the file and
owns the duplicate rule.

A new run is reported with its date and distance. A duplicate is reported as
one line, for example:

```text
already have this run: 12 Oct, 10.2 km
```

A duplicate adds no row. A file that is not a readable FIT file, or one over
20 MiB, is refused with a sentence rather than a stack trace.

## 6. Shared screens and platform boundaries

Today, Runs, Progress and Body are shared at every width, with contextual Plan
and Compare. Wider screens use columns and a navigation rail; compact screens
use bottom navigation. Host capabilities differ: Android owns Keystore,
locking, background work and document delivery; desktop owns its local HTTP
server and directory-based tools.

## Where the files are

`~/.local/share/stride` on Linux, `%LOCALAPPDATA%\stride` on Windows, and
`$XDG_DATA_HOME/stride` when that is set. `STRIDE_DATA_DIR` overrides it.

```text
stride.db          the live encrypted database
db.key             the live database key, private to the account
garmin.key         the Garmin key, private to the account, when one is kept
first-run          a non-secret marker that remembers the first-run choice
exports/           private export bundles
imports/           temporary upload staging, removed after an import
```

Converting an older plain database leaves `stride.db.plaintext-backup` beside
it, also private to the account (`0600` on POSIX systems); it is the
recovery copy and the athlete's to delete once the encrypted database opens.

The Garmin key and the database key are different secrets. Neither opens the
other.

## What the rig is not

- **not a second implementation.** Every number comes from `stride-core`, every
  reading from `stride-store`, and every product request goes through
  `stride-api`'s router. The rig is a host, not a brain.
- **not a standalone PWA.** Opening the browser UI still requires the Rust host.
- **not a service.** There is no cloud copy, no account and no scheduler. Stop
  the binary and the rig is gone; the data stays in the file.
