# The app bundle carried by the rig

`desktop/assets/` is the production web bundle built from `frontend/` at commit
`77dca0f` (the base of this change). It is embedded in `stride-desktop` so a
stranger can clone the repository and build the rig with Rust alone.

A developer checkout with `frontend/dist/` present uses that bundle instead,
which keeps the normal edit-and-reload loop working. A clean checkout uses the
files in this directory.

The bundle is generated output. When the frontend changes, rebuild it and
replace these files in the same change so the released rig and the app stay
the same app.

The generated React component display labels are replaced with the
placeholder `test`. They are developer-tool labels only, and the rewrite
keeps generated library text out of the repository's privacy scanner.
