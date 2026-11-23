# Backups App

A portable folder backup tool for Windows. It saves your folders to a ZIP file.

## Use

Requires Windows 10 or later (64-bit). The application is portable: it has no installer.

1. Download `backups-<version>-portable.zip` and extract `backups.exe`.
2. Run `backups.exe`.
3. Add the source folders, the exclusions (optional) and the destination folder.
4. Click **Back up now**.

The application creates `backup-YYYY-MM-DD-HH-MM-SS.zip` in the destination folder. Each source
folder is a top-level folder in the ZIP. The settings are saved in `config.toml`, next to
`backups.exe`.

### Exclusions

Each pattern is compared with the name of each file and folder, not with the full path. `*`
matches any text and `?` matches one character. Upper case and lower case are the same. If a
folder matches, the application skips all its contents.

Examples: `node_modules`, `*.tmp`, `~$*` (Office lock files), `Thumbs.db`.

### Good to know

- The ZIP is written to a temporary `.zip.tmp` file and renamed only when the backup is complete.
- If the application cannot read a file (for example, a locked file), it skips the file and shows
  it in a list at the end.
- Windows Explorer cannot open a ZIP that contains a path of 260 characters or more. Use 7-Zip to open that ZIP.
- Symbolic links are not copied.

## Development

The application is written in Rust with [egui](https://github.com/emilk/egui). It builds in a
Docker container that cross-compiles to Windows. You need Docker Compose and `make`, but not Rust.
To run the application, use Windows or WSL.

Run `make` to see all the commands. Before you commit, run `make check-all`.

### Releases

When you push a version tag (for example, `0.2.0`), the workflow `.github/workflows/release.yml`
creates a GitHub release with the portable ZIP. The tag must match the version in `Cargo.toml`.
The release notes come from that version's section in `CHANGELOG.md`.

1. In `CHANGELOG.md`, remove the entries that users do not see from `## Unreleased`, and put the
   most important change first. Below the heading, write one or two sentences about that change.
   Follow [docs/how-to-write-the-changelog.md](docs/how-to-write-the-changelog.md).
2. Change `## Unreleased` to `## X.Y.Z - YYYY-MM-DD`.
3. Change `version` in `Cargo.toml` and run `make build` to update `Cargo.lock`.
4. Commit the three files, then run `git tag X.Y.Z` and `git push origin X.Y.Z`.
