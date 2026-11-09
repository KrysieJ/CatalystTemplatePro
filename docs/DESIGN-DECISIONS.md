# Design Decisions

This is the "why" document. If you want to change something here, read the
reason first. If the reason no longer holds, change the decision and say so.
If you just don't like it, tough; it's a personal project and the author
gets to be stubborn.

Date of writing: 2026-11-08. Everything about Cisco software versions,
platforms, and documents is as of that date.

## How to read the evidence labels

Every factual claim below carries one of these:

- **[Verified]**: seen on a primary source (Cisco's site, the linked
  repositories) during this design phase. Cited in the references.
- **[Author]**: stated by the author from hands-on experience with the gear.
  In the source code this appears as **[Engineer]**, because the tool will
  record statements from any engineer, not only the author.
  Treated as a source of truth until the author says otherwise. Not
  independently verified.
- **[Unverified]**: believed true, not checked. These are the things to go
  check before depending on them.

Where sources conflict, the tool and the people building it stop and
discuss. Nobody guesses. See D6.

---

## D1. Rust core, Rust GUI, nothing else

**Decision.** The data model, validation, template generation, and the GUI
are all Rust. No Electron, no embedded browser, no bundled JavaScript
runtime.

**Why.** The author needs this to be auditable by someone who is not the
author, and to run on locked-down machines with no runtimes installed. A
single compiled language with a locked dependency set is far easier to audit
than a Rust core wrapped in a JavaScript dependency tree. Rust's type system
also lets the compiler enforce data definitions that a design-recipe
methodology wants enforced.

**Rejected.** Vendoring a JavaScript block editor behind a Rust core.
Cheaper to start, but it brings an npm dependency tree into an environment
that cares about supply chain risk. A lighter web front end was also
considered and rejected for the same reason.

**Risk.** Native Rust GUI toolkits are less mature than web UI stacks.
Custom Aero-style rendering (glass, gradients, bevels) may be hard. The
toolkit choice is open (see Open Questions).

## D2. Scratch is inspiration, not a code base

**Decision.** No Scratch code is reused. The block-editor idea is borrowed;
the implementation is original.

**Why.** [Verified] scratch-desktop is an Electron application built with
webpack and Babel, and it pulls in `scratch-gui` as a separate dependency
[1]. That is the opposite of D1. [Verified] The repository is AGPL-3.0 and
ships a `TRADEMARK` file [1]; the license text and trademark terms were not
read, only their existence confirmed. Avoiding the code avoids the question.
[Verified] It also includes an opt-in telemetry system [1], which conflicts
with D4.

**Risk.** Block-editor interaction design (snapping, drag and drop,
nesting) has to be built from scratch. That is real work.

## D3. Offline, packaged, Windows 11 first

**Decision.** Ships as a compiled program that runs on Windows 11 with no
runtime installs and no internet access. A packaged installer is preferred.
A Microsoft Store version is possible later and not a driver.

**Why.** The tool is meant for critical-infrastructure networks (utilities,
emergency services, systems handling Controlled Unclassified Information)
where data sovereignty matters and internet-dependent tools get rejected.

**Rejected.** A cloud or web version. Any feature that needs the network at
runtime. Any "phone home" update check.

## D4. No telemetry. Ever.

**Decision.** No telemetry, analytics, crash reporting to a remote server, or
update beacons. Not even opt-in.

**Why.** Opt-in is a policy, not a guarantee, and policies get audited. The
only provable position is that the code has no such path.

## D5. Method: How to Design Programs, translated for Rust

**Decision.** Development follows the HtDP design recipe [2][3]: define the
data, write the signature and purpose, write examples (as tests) before the
function, then write the function. How to Design Classes [4] informs
structure, translated to Rust structs, enums, and traits, because Rust has
no classes.

**Why.** [Author] The goal is that people who are not the original author
can audit the code. A recipe with a fixed shape (data definition, signature,
purpose statement, examples, body) gives auditors something to check
mechanically. It also suits an author who cannot read the code but can read
a data definition and a list of examples.

**Mapping to Rust (proposed, not final).**

| HtDP step | Rust form |
|---|---|
| Data definition | `struct` / `enum` with a doc comment saying what it represents |
| Signature | The function type |
| Purpose statement | Doc comment, one sentence |
| Examples | Unit tests written first |
| Template / body | The function |

**Risk.** [Unverified] This design was written without reading the HtDP
editions or HtDC in full. The mapping above is a proposal from general
knowledge of the recipe. HtDC's class-oriented material has no direct Rust
equivalent, so how it is translated is a judgment call that should be
reviewed by someone who knows both.

## D6. Sources of truth, and what happens when they disagree

**Decision.** Each kind of question has one authority:

| Question | Authority |
|---|---|
| Does command X exist, and what is its syntax? | Cisco command reference for that platform and release |
| What does the config structure look like? | Cisco YANG models for that release (skeleton only) |
| Is this template legal in Catalyst Center? | CATC-TEMPLATE-RAG [5] |
| What is true about the author's own network or hardware? | The author |

Conflicts between any two become a `SourceConflict` finding. The tool shows
both sides and does not choose.

**Why.** [Author] The RAG controls template accuracy but is an example and
a source of truth, not the only factor. Cisco's references are authoritative
for commands. [Author] YANG models exist for 26.1.1, 26.1.2, and 26.2.1.

**Caveats on the RAG.** [Verified] It is a single-author repository with a
handful of commits [5]. [Author] It is endorsed by Cisco projects; that
endorsement was not independently verified. [Verified] It states it is
distilled from Cisco documentation [5], so where it and Cisco disagree,
Cisco wins and the discrepancy gets reported. Small internal inconsistencies
were seen in its README (broken-looking links to `AGENTS.md` and `INDEX.md`;
one Velocity example answer that cites a Jinja2-only directive) [5]. Report
them upstream; do not silently work around them.

**Risk.** [Unverified] YANG describes structure and may not map one-to-one to
every CLI command. That is exactly why it supplies the skeleton and the
command references confirm the CLI.

## D7. The command catalog is data, not code

**Decision.** Supported commands, arguments, modes, and per-platform notes
live in data files generated from the sources in D6. Nobody types a command
into Rust source by hand.

**Why.** Cisco's command set is large, platform-specific, and changes every
release. Hand-typed commands rot and hide mistakes. Data with a recorded
source can be diffed between releases and audited.

## D8. Platforms follow Catalyst Center's levels, plus image and reference

**Decision.** The hardware levels are Catalyst Center's [Author]:

- **Family**: the broad device type, such as Switches and Hubs.
- **Series**: such as Catalyst 9300 Series or Catalyst 9350 Series.
- **Platform**: a product ID, such as C9300-48P.

Alongside these, each series records:

- **Image**: the software file family it runs, such as cat9k_iosxe.
- **Command reference**: the Cisco command reference it follows. Several
  series can share one (the 9350 follows the 9300's).
- **Default Catalyst Center role**: a starting suggestion (D24).

One image does not mean one command set: a feature can exist on some series
and not others. Those differences are callouts in the command catalog.

**Images, from the author installing firmware [Author, 2026-11-08]:**

| Image | Series | Variants |
|---|---|---|
| cat9k_iosxe | C9300, C9350, C9400, C9500, C9550, C9600, C9610 | |
| cat9k_lite_iosxe | C9200, C9200CX | |
| C9800-universalk9 | C9800 (C91xx AP images are packaged inside it) | |
| cat3k_caa-universalk9 | C3650, C3850 (legacy, outside the 26 scope) | |
| ie3x00-universalk9 | IE3400, IE3500 | |
| c8000be-universalk9 | Catalyst 8300 Edge Platform | npe |
| c8kg2be-universalk9 | Catalyst 8300 and 8200 Secure Router Gen2 | |
| c8000aep-universalk9 | Catalyst 8500 Edge Platform, 8500 Secure Router Gen2 | npe, noli |
| c8000aes-universalk9 | Catalyst 8500L Edge Platform | |
| ir1800-universalk9 | IR1833, IR1835 | |

Variants: `npe` is No Payload Encryption, `noli` is No Lawful Intercept, and
they can combine (`_npe_noli`).

**PIDs [Author].** In a Catalyst 9300 PID such as C9300-48P-A, the letter
after the port count gives power: P is PoE+, U is UPoE, H is UPoE+, T is no
PoE. A trailing -A means the device originally shipped with Network
Advantage. Not yet used by the code; applying it to other series is
unverified.

**C9200CX [Author].** Uses the C9200 command reference, but its feature set
may be limited. Its data sheet is the source for feature limits.

**Why.** [Author] The IE3500 runs the same image as the IE3400: "x" in
IE3x00 is a wildcard in the image name. The IR1833 and IR1835 share an image
and a command reference and differ by an I/O header. The C9350, C9550, and
C9610 run the same image as the "00" Catalyst 9k models but add features,
such as power-management features not on the "00" models. The command
references should be the same. Modelling by image means one catalog serves
many models, and per-model differences are callouts.

**Evidence notes.** [Verified] Cisco's 26.x release notes group
"IE3x00, IE3100, IE3400 Heavy Duty, ESS3300" and "IR1101, IR1800, IR8140,
IR8340" [6]. [Verified] The newer C9350, C9550, and C9610 have their own 26.x
release notes [6]; command references were seen for the 9200, 9300, 9400,
9500, and 9600 only [6]. [Verified] A search for IE3500 release notes
returned 17.x notes and no 26.x ones [7]. The author's statement that it
shares the IE3x00 image is treated as authoritative here, with the gap noted.

## D9. Every claim about availability is one of three things

**Decision.** `Supported`, `NotSupported`, or `Unverified`, each with an
optional license requirement and a source (document, release, retrieval
date).

**Why.** "I don't know" must be a representable answer. A tool that defaults
unknown to "works" is lying, and a tool that defaults unknown to "broken" is
annoying. Accuracy over every other metric.

## D10. Licensing is feature-level, surfaced as a callout

**Decision.** When Cisco documentation ties a feature to a license level, all
commands for that feature inherit that requirement. If the target device
does not have the license, the tool raises a callout. It does not block
anything.

**Why.** [Verified] Cisco's data sheets list features by package (Network
Essentials versus Network Advantage) rather than by individual command [8].
[Verified] Cisco's 9200 guide lists the base levels (Network Essentials,
Network Advantage) and DNA add-ons (DNA Essentials, DNA Advantage) [9]. The
author's examples: BGP and NetFlow are the sort of feature typically gated
by license level. [Author]

**Interpretation to confirm.** The author said the tool should "throw an
error" when the license is missing and then clarified it should be a callout
instead. This document records it as a callout, consistent with D11. Say so
if that is wrong.

**Risks.**
- [Unverified] Cisco Feature Navigator is a likely source for
  feature-to-license data, but it is a web tool. Whether it can feed an
  offline catalog, and under what terms, is not checked.
- [Unverified] License models for routers and IE/IR switches were not
  checked. Anything without a Cisco source stays `Unverified`.
- Command-to-feature mapping is a curated step and the likeliest place for
  errors to creep in.

## D11. The tool advises. The engineer decides.

**Decision.** The tool never blocks an export. It produces findings: platform,
license, workflow, template-syntax, source-conflict, and unverified (added in
D21). It can print a limitations report grouped by platform and license.

**Why.** [Author] The users are engineers. They need reminders, not
hand-holding. Catalyst Center failing a workflow because a replaced device is
a different platform is the cautionary tale: the engineer already decided,
and the software argued. The limitations report also gives an engineer
something concrete to hand to leadership when a feature needs an upgrade.

## D12. Features are modular; composition is done by the tool

**Decision.** Each feature (AAA, admin interfaces, ACLs, and so on) is a
component. A master is an ordered list of components. Components declare
device roles, platforms, and the workflows they are valid in. Swapping one
component for another (for example AAA for an 802.1x variant) is a single
change to the master. At export the tool flattens, or uses native includes
where proven safe.

**Why.** [Author] AAA is mostly identical across PE, P, core, CE, and access
devices, and sites migrate to port authentication at different times.
Catalyst Center cannot be relied on to do the composing, because
[Verified] its own rules differ by engine and workflow [5]:

- Velocity: `#parse` and `#include` are disabled.
- Jinja2: `{% include %}`, `{% extends %}`, and `{% import %}` are supported
  and resolve against the Template Editor project tree.
- Composite templates are valid only in DayN workflows.
- PnP onboarding templates cannot use bind variables or `__*` system
  variables.

So export has three modes: flattened (required for Velocity and PnP), Jinja2
with includes (where the RAG confirms it), and Composite (DayN only).

## D13. Target output: Jinja2 and Velocity for Catalyst Center

**Decision.** Both engines, following the RAG's constraints [5]. The tool
must handle Catalyst Center specifics the RAG documents, such as `!` being
forwarded into IOS config rather than treated as a comment, `#MODE_ENABLE`
blocks, `<MLTCMD>` multi-line blocks, and compliance-ignore markers [5].

## D14. How an author who can't read the code keeps the code honest

**Decision.** Checking is mechanical, not based on trust.

- **Tests first.** [Verified] The RAG ships 85+ Jinja2 and 14 Velocity real
  templates [5]. They become the regression corpus.
- **Separate validator** against the RAG's bad/good patterns.
- **Locked dependencies**, a minimal dependency list, and `cargo audit` and
  `cargo deny` in the build. No network at runtime.
- **Human review.** A Rust developer reviews the code before anything touches
  production.
- **Disclosure.** Code is written with AI assistance. The README says so.

**Why.** [Author] Vibe coding carries bug and vulnerability risk. A
defense-in-depth mindset means not relying on any single control, including
"the AI got it right."

## D15. Synthetic data only

**Decision.** No real configurations, addresses, credentials, or CUI in any
cloud AI tool while working on this project. Test fixtures are synthetic.

**Why.** Networks this tool is meant for carry emergency-services and
utility traffic, and CUI as defined by the National Archives. A leak
through a development tool is still a leak.

**This repository is public.** It holds no details of any real network:
no versions in service, upgrade plans, inventory, addressing, or how a
particular organization assigns roles. Examples are generic or synthetic.

## D16. Look and feel

**Decision.** Dark theme as the native mode. Windows 7 Aero as the design
language, as inspiration rather than a copy. The colors are the older
off-teal and red. The logo is an original mark in the same spirit (a bridge
motif), not Cisco's.

**Why.** [Author] Dark themes are easier on the eyes, and the current
flat-minimalist trend is a rejected design direction. Using the Cisco logo
risks a trademark problem; an original homage does not. [Unverified]
Cisco's trademark terms were not read. The README disclaims any
affiliation.

## D17. Phasing

1. Catalog structure for the Catalyst 9300 (largest family, command
   reference exists).
2. Prove the pipeline end to end with AAA on the 9300 and 9200:
   catalog, block, component, export, validate.
3. Add 8300/8500, IR1800, and IE3x00 one at a time. Each addition tests the
   "only a few commands" assumption.
4. Add 9800 and the newer Catalyst 9k models with platform callouts.
5. Other IOS XE 26 platforms: 8000V, 8000 Series Secure Routers, ESR6300,
   IE9300, ESS9300 [6].
6. Later, in order: UML-style modelling, SVG diagrams, IOS XE 16/17, IOS 15,
   NX-OS (low priority).

[Author] Platform priority follows what the author can test on. Older
platforms and software trains are out of scope for the IOS XE 26 work.

## D18. Dependencies only on purpose

**Decision.** The core library started with only the Rust standard library.
A dependency is added only when it earns its place, it is written down here
with the reason, and its exact version is locked in the committed
`Cargo.lock`. Dates stay three plain numbers instead of a date library.

**Why.** Every dependency is someone else's code that has to be audited and
kept patched.

**Dependencies so far.** `serde` and `toml`, for reading data files (D23).

## D19. Make wrong states impossible to write down

**Decision.** The data definitions are shaped so the compiler refuses claims
that should not exist:

- A command cannot be marked supported or not supported without a source.
  Only "unverified" may have no source.
- A license requirement cannot be empty. A feature with no requirement simply
  has none.
- Release numbers and dates can only be made through functions that check
  them, so "26.x.1" or 31 April cannot get into the catalog.

**Why.** Accuracy over everything else, and an author who cannot read the
code. A rule the compiler enforces does not depend on anyone remembering it.

## D20. Platform families live in their own data file

**Decision (2026-11-08, author).** Platform families and models move out of
the Rust source into their own data file. Adding a device must not need a
code change or a rebuild.

**Why.** [Author] Supporting a new device should be cheap when it only takes
a few commands. A closed list in the code makes every new family a code
change.

**What this gives up, and the replacement.** With a fixed list, the compiler
pointed at every place that had to handle a new family. With a data file it
cannot. The replacement check: the tool validates the platform file when it
loads it, and refuses the whole file if anything is wrong: a duplicate name
or id, a blank name, a model pointing at a family or source that does not
exist, a date or release that is not real, a misspelled or unknown field, or
a newer file layout than the program understands. A broken TOML file or a
misspelled field gives an error naming the line. The other errors name the
entry and field instead, because by then the file has already been read.

**Status.** Built. The list is `data/platforms.toml`, which is built into
the program so it works offline. Every family, image, command reference,
series, platform and default role names the source behind it.

## D21. "Unverified" is its own kind of finding

**Decision.** When nothing confirms a command works on the target, the tool
says so with a finding of kind *unverified*, which carries no source.

**Status.** Confirmed by the author, 2026-11-08.

**Why.** Folding it into "platform" would make "Cisco says no" and "nobody
has checked" look the same. They are different, and an engineer, or the
leadership reading a limitations report, should be able to tell them apart.

## D22. Licensing as written in the first code

**Decision.** The first license types cover Catalyst 9000 switches only:
Network Essentials and Advantage as base levels, DNA Essentials and Advantage
as add-ons. A higher level covers the lower one. The base and add-on are
checked separately.

**Why.** [Verified] Cisco's 9200 guide lists exactly those levels and states
that each Advantage level includes its Essentials level [9]. [Engineer] The
add-ons are also called Catalyst Essentials and Advantage.

**Not encoded, because not verified.** Whether an add-on requires a
particular base level. License models for routers, IE/IR switches, and the
9800. Licenses that are not recorded give a reminder, not silence.

## D23. Data files are TOML

**Decision (2026-11-08, author).** The platform list, and later the command
catalog, are TOML files. They are read with the `serde` and `toml` Rust
libraries.

**Why.** TOML is plain text an engineer can read and edit, it handles nested
data (a command with its arguments), and changes show up cleanly in Git.

**Rejected.**
- CSV: editable in Excel, but nesting needs extra sheets.
- A custom format: no dependencies, but a hand-written reader is exactly
  where bugs hide.

**Cost.** Two libraries, and what they pull in, become code to audit.
[Verified] `toml` is at 1.x and implements TOML spec 1.1 (docs.rs, toml
1.0.3+spec-1.1.0, built 2026-02-18). Versions are pinned by `Cargo.lock`.
`cargo audit` and `cargo deny` (D14) should be added to the build before
release.

**How files are read.** In two steps. First, the TOML is read into structures
that mirror the file exactly, refusing unknown or misspelled fields. Second,
those are checked and turned into the program's own types (D19). Nothing
unchecked gets past the loader.

## D24. One Catalyst Center role, any number of role tags

**Decision (2026-11-08, author).** A device has two kinds of role.

- **Catalyst Center role.** Exactly one per device: Border Router, Core,
  Distribution or Access. It is set to whatever Catalyst Center needs for
  that device, and never worked out from anything else. Each series carries
  Cisco's usual role as a suggestion (9200, 9200CX, 9300, 9400: Access;
  9500: Distribution; 9600: Core; xx50 and xx10 take their parent's), with
  its source.
- **Role tags.** Any number per device, from `data/roles.toml`, which is
  meant to be edited. Components attach to tags, and a device gets every
  component its tags call for. Starting tags: Border, Core, P-Router,
  Distribution, PE, CE / Site Router, Access.

**Why.** [Author] Cisco's model says one device fills one role, and real
networks do not work that way. A site router can be border, distribution
and access at once. Catalyst Center also uses its role for telemetry and
QoS, so a device sometimes has to carry a Catalyst Center role that
understates what it does. Keeping the two apart lets each be right.

**Status.** The role types and the tag file are built. Attaching components
to tags comes with the component work. Loading an edited tag file from
outside the program comes with the GUI; for now it is built in.

**Rejected.** A single role per device (Cisco's model). Inferring the
Catalyst Center role from the tags.

## D25. Firmware file names are read, not typed

**Decision.** The tool can read a firmware file name such as
`c8000aep-universalk9_npe_noli.17.15.05.SPA.bin` and give back the image,
the variants and the release (17.15.5).

**Why.** [Author] File names follow `<image>[_variant].<MM>.<mm>.<rr>.SPA.bin`
across releases. Pasting a name you already have is faster than picking
three fields, and cannot be mistyped.

**Accuracy rule.** Only images and variants in the platform list are
accepted. A name that is almost right is refused, not guessed at. Image
names can contain underscores (`cat9k_iosxe`), so variants are found by
matching the longest known image name first, not by splitting on `_`.

---

## Open questions

1. **GUI toolkit.** Which native Rust toolkit supports Aero-style rendering,
   offline packaging, and accessibility? Needs a cited comparison first.
2. **License data source.** Can Cisco Feature Navigator data be used offline?
   What do the Cisco docs say for routers and IE/IR switches?
3. **Command references for the newer 9k models.** Do the C9350, C9550, and
   C9610 have separate references, or only the shared one?
4. **IE3500 on 26.x.** Confirm on a real device and note the release-note
   title gap.
5. **License callout wording.** Confirm D10's interpretation.
6. **Project license.** Not chosen.
7. **Command catalog layout** in TOML (D7, D23): how a command, its
   arguments, its mode, and its per-platform availability are written.
8. **Catalyst Center family names** for routers, wireless controllers and
   access points, and command references and default roles for the router,
   IE and 9800 series (D8).
9. **PID decoding** (D8): whether the PoE and license letters apply beyond
   the Catalyst 9300, and what letters other than -A mean.

## Non-goals

- Replacing Catalyst Center.
- Pushing configs to devices. This tool produces templates.
- Hand-holding or blocking engineers.
- Any online or cloud feature.

## References

[1] Scratch Foundation, "scratch-desktop," GitHub. [Online]. Available: https://github.com/scratchfoundation/scratch-desktop

[2] M. Felleisen, R. B. Findler, M. Flatt, and S. Krishnamurthi, *How to Design Programs*, 1st ed. [Online]. Available: https://htdp.org/2003-09-26/

[3] ———, *How to Design Programs*, 2nd ed. [Online]. Available: https://htdp.org/2026-5-28//Book/index.html

[4] M. Felleisen et al., *How to Design Classes*. [Online]. Available: https://felleisen.org/matthias/HtDC/htdc.pdf

[5] K. Baldwin, "CATC-TEMPLATE-RAG," GitHub. [Online]. Available: https://github.com/kebaldwi/CATC-TEMPLATE-RAG

[6] Cisco Systems, "Cisco IOS XE 26," Cisco Support. [Online]. Available: https://www.cisco.com/c/en/us/support/ios-nx-os-software/ios-xe-26/products-release-notes-list.html

[7] Cisco Systems, "Cisco IE3500 Rugged Series: Release Notes," Cisco Support. [Online]. Available: https://www.cisco.com/c/en/us/support/switches/ie3500-rugged-series/products-release-notes-list.html

[8] Cisco Systems, "Cisco Catalyst 9500 Series Switches Data Sheet," Cisco, 2025.

[9] Cisco Systems, "Configuring the License Level," Cisco Catalyst 9200 Configuration Guide.

*Author attributions in [2]-[4] are from general knowledge and were not checked against the books' title pages.*
