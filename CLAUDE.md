# Working on CatalystTemplatePro

Read this first, then `README.md` and `docs/DESIGN-DECISIONS.md`. The design
document is the record of what has been decided and why; its "Open
questions" section is the to-do list.

## Who you are working for

The author is a network engineer, not a programmer, and wants to do as
little hands-on work as possible. So:

- **Do the work.** Run the commands, build, test, commit and push yourself.
  Do not hand the author commands to type unless something genuinely needs
  them (a sign-in, a password prompt, a choice only they can make).
- **Explain in plain words**, in network-engineering terms where that helps.
  No jargon without a one-line explanation.
- **The author's statements are a source of truth** about Cisco hardware,
  images and their own network, until they say otherwise. If a statement
  conflicts with a Cisco document or with an earlier statement, stop and
  ask. Never quietly pick one.
- **Accuracy beats speed.** If something is not known, say so and record it
  as unverified. Never guess.
- **Ask before anything that cannot be undone**: force-pushing, rewriting
  history, deleting branches or files, changing repository settings.
- Professional writing in this repo uses IEEE reference style.

## Environment

- Windows 11, PowerShell. Rust (`cargo`) and Git for Windows are installed.
- Git already works on this machine. GitHub sign-in uses the author's
  Microsoft 365 single sign-on with passwordless login. If a push or pull
  asks for credentials, stop and tell the author; do not try to set up
  tokens, SSH keys or credential helpers yourself.

## How the code is written

Follow *How to Design Programs* (design decision D5):

1. **Data definition first**: a struct or enum with a doc comment saying
   what it represents and an example of the data.
2. **Purpose statement**: every function's doc comment starts with
   `Purpose:` and one sentence.
3. **Examples as tests**, written before the function body. Test names are
   plain-English sentences (`refuses_a_duplicate_series`).
4. **Then the body.**

Rules that hold everywhere:

- **Make wrong states impossible to write down** (D19). A claim of support
  needs a source; checked types are built only through functions that check
  them; struct fields stay private when that is what keeps data valid.
- **Every claim carries its source** (D6, D9). In comments and docs, label
  facts **[Verified]** (seen in a primary source, cited), **[Engineer]**
  (stated by the author) or **[Unverified]**.
- **Data files are TOML** (D23), read in two steps: raw structs with
  `#[serde(deny_unknown_fields)]`, then checked into real types. A data file
  with any problem is refused whole, with a `DataError` that names the entry.
- **Unknown is a value**, not a gap: leave a field out or mark it
  unverified rather than filling it in.
- **No new dependency** without recording why in the design document (D18).
  `Cargo.lock` is committed.
- `#![forbid(unsafe_code)]` stays. No telemetry, no network access at
  runtime (D3, D4).

## This repository is public

Never commit anything about a real network: versions in service, upgrade
plans, inventory, addresses, hostnames, credentials, configurations, or how
a particular organization assigns roles. Test data and examples are
synthetic. If the author shares such details while working, use them to
understand the task and keep them out of the repo.

## Before every commit

Run, from the repository folder:

```
cargo fmt
cargo clippy --all-targets
cargo test
```

All tests must pass and clippy should have no warnings. If a test fails,
find out whether the code or the example is wrong; never delete or weaken a
test just to make it pass.

## Branches, commits and merging

- Work on a branch named for the change (`command-catalog`, for example),
  never directly on `main`.
- Commit messages: a short summary line, then a plain-English body saying
  what changed and why.
- When `cargo test` passes on the branch, merge it into `main` and push,
  then tell the author in a sentence or two what changed. Use a normal
  merge or fast-forward; never force-push.
- Record each new decision in `docs/DESIGN-DECISIONS.md` as the next D-number,
  with the decision, the reason, what was rejected, and evidence labels.
  Update "Open questions" when one is settled or a new one appears.

## Layout

- `src/` – the library: `release`, `source`, `data`, `license`, `platform`,
  `roles`, `availability`, `finding`.
- `data/platforms.toml` – families, series, platforms, images, variants,
  command references, with sources.
- `data/roles.toml` – role tags, meant to be edited.
- `docs/DESIGN-DECISIONS.md` – every decision and open question.
