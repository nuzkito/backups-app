# How to write the changelog

`CHANGELOG.md` tells users what is new in each version of Backups App. When you publish a version,
the release workflow copies its section to the GitHub release. Thus, this text is also the release
announcement. The steps to publish a version are in the "Releases" section of the
[README](../README.md).

## Format

The release workflow reads the file, so keep this structure:

```markdown
# Changelog

The description of the app. This page tells you what is new in each version.

## Unreleased

## 0.2.0 - 2027-01-15

One or two sentences about the most important change in this version.

### A title that says what the user can do now

A short paragraph about the change.
```

- Each version has a `## <version> - <YYYY-MM-DD>` heading. The version must be the same as in
  `Cargo.toml`, without the `v`.
- Put the changes inside the version section with `###` headings. Do not use `##`, because a `##`
  heading ends the section that the workflow copies.
- Put the newest version first.
- If the version section is missing or empty, the release workflow fails.

## While you work

Add an entry to the `## Unreleased` section when you make a change that users can see. Do not wait
for the release: it is easier to explain a change when you have just made it.

## Choose what to include

Include only the changes that make a difference to users. Ask these questions:

- What can users do now that they could not do before?
- What is easier now?
- What is faster now?

Do not include:

- Refactoring, tests, build changes and dependency updates, if users see no difference.
- Implementation details (names of functions, crates, files inside the project).
- "Various improvements and bug fixes". If you cannot say how a change helps users, leave it out.

## Write for the user

- **Speak to the user.** Use "you". Say what the user can do and which button to click.

  > Not this: Added exclusion patterns.
  >
  > This: Add names such as `node_modules` or `*.tmp` to **Exclusions**, and Backups App skips them.
  > Your backups are smaller and finish sooner.

- **Write the title as a benefit.** "Skip the files you do not need" is better than "Exclusions".

- **Tell what is better, not what was broken.** Describe a fix as an improvement.

  > Not this: Fixed a bug that made the window freeze for 2 seconds when you added a folder.
  >
  > This: Folders appear in the list as soon as you add them.

- **Give measured numbers.** Measure before and after the change with the same files. If you did
  not measure, do not write a number.

  > Not this: Backups of photos are faster.
  >
  > This: Backups of photos are 3 times faster.

- **Start with the most important change.** Put it in the first sentences of the version and as the
  first `###` section.

- **Use the names that users see.** Use the button and section names of the application, for
  example **Back up now** and **Exclusions**, in bold.
