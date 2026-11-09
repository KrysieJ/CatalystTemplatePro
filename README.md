# CatalystTemplatePro

A Rust-native, offline desktop tool that lets network engineers build modular
Cisco Catalyst Center configuration templates (Jinja2 and Apache Velocity)
from a visual block editor, without having to be programmers.

**Status: early groundwork.** The core data definitions exist (releases,
sources, licenses, platforms, roles, availability, findings). The platform
list (`data/platforms.toml`) and role tags (`data/roles.toml`) are checked
data files, and firmware file names can be read into image, variant and
release. There is no GUI and no
template generation yet. Read
[docs/DESIGN-DECISIONS.md](docs/DESIGN-DECISIONS.md) to see what has been
decided, why, and what is still open.

## Why this exists

Catalyst Center's built-in configuration modules are weak, and its template
editor assumes you can program. Plenty of competent network engineers can
read and write CLI all day and still do not want to learn a templating
language that a vendor will change next year. This tool is for them.

The engineer drives. The tool reminds, annotates, and warns. It does not
hold anyone's hand and it does not block an export because it thinks it
knows better.

## What it will do

- Block-based editor, inspired by Scratch (inspired, not derived; no Scratch code is used).
- Command catalog for Cisco IOS XE 26, built from Cisco's command references
  and YANG models, annotated per platform, release, and license level.
- Modular templates: AAA, administrative interfaces, ACLs, and so on are
  separate components. A master template assembles them, and components can
  be swapped (for example an AAA component for an 802.1x one) without
  rebuilding the master.
- Export to Jinja2 or Velocity, following the Catalyst Center rules documented
  in the CATC-TEMPLATE-RAG repository.
- Advisory validation: "this will not work on that platform / license /
  workflow", with a printable limitations report for the people who sign
  the purchase orders.

Later, in this order: UML-style modeling, SVG network diagrams, more
platforms, older software trains. NX-OS is a maybe.

## Hard requirements

- Rust core and Rust GUI. No Electron, no bundled JavaScript runtime.
- Runs as a compiled program on Windows 11 with no runtimes to install and no
  internet access. Packaged installer preferred.
- No telemetry. Ever.
- Built with the design recipe from *How to Design Programs* (and *How to
  Design Classes*, translated to Rust) so that someone other than the author
  can audit it.
- Dark theme first. Aero-inspired look. Flat minimalism is not a goal.

## Platform scope

IOS XE 26 first. Priority for testing: Catalyst 9000 family, Catalyst 8300 and
8500, IR1800 series, IE3x00 series. Everything else is added when it costs a
few commands, not a rewrite. See the design document for details and for
what has and has not been verified.

## Building and testing

You need the Rust toolchain (`cargo`). From the repository folder:

```
cargo test
```

That builds the library and runs every example. The first build after a
dependency is added needs internet access once, to download it; builds after
that work offline. All of them should pass. A
failing test means either the code or the example is wrong, and both are
worth reading. The first build also creates `Cargo.lock`; commit it, so every
later build uses exactly the same dependency versions.

Optional checks:

```
cargo fmt --check
cargo clippy
```

## Read this before trusting anything here

- The author is a network engineer, not a programmer. Code in this project is
  written with AI assistance. That is a risk and the design document says
  how the risk is managed: tests first, locked dependencies, mechanical
  checks, and a human Rust reviewer before real use.
- Do not use output from this tool on production gear without testing it
  yourself. You are the engineer.
- This project is not affiliated with, endorsed by, or sponsored by Cisco.
  "Cisco" and "Catalyst" are trademarks of Cisco Systems, Inc.
- Do not put real configurations, addresses, credentials, or controlled
  information (CUI and similar) into any cloud AI tool while working on this
  project. Use synthetic data.

## License

Not chosen yet. Until one is added, all rights are reserved.
