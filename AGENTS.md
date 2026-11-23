# Code comments

Write a comment only when the code and its context do not give the information. Good reasons for a
comment are:

- the reason for a decision (for example, a security or data-safety constraint)
- units, value ranges or the meaning of `None`
- a short example of a format
- the steps of an algorithm that is not clear from the code

Do not write comments that repeat the name of a module, type, function, field or constant, or that
describe what the next lines obviously do.

# Changelog

When a change is visible to users, add an entry to the `## Unreleased` section of `CHANGELOG.md`.
Follow the rules in `docs/how-to-write-the-changelog.md`. Do not add entries for changes that users
do not see (refactoring, tests, build, dependencies).

# UI screenshots

When you modify the UI (`src/ui/`), generate screenshots of the affected screens and check that the
UI looks correct (complete text, alignment, spacing, colors, and no clipped or overlapping
elements). If something is wrong, fix it and regenerate the screenshots.

```sh
make screenshots S=copying,failed  # only the scenes whose name contains "copying" or "failed"
make screenshots                   # all scenes
```

Images are saved to `screenshots/<scene>.png`. Scenes are defined in `scenes()` in
`src/ui/screenshots.rs`. If the modified part does not appear in any scene, add a scene.

When you finish, show the user links to only the screenshots that show the change (for example,
`[copying.png](screenshots/copying.png)`).
