# CI Hardenings + Treefmt Priorities Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Port Sort-Markdown-Tables CI hardenings into Profile-Pulse and align treefmt formatter priorities/excludes, without reshaping the existing multi-OS job topology.

**Architecture:** Keep Profile-Pulse’s current CI jobs (`build`, `check`, `clippy`, `doc`, `fmt`, `security-audit`, `spell-check`, `test`). Add zizmor-oriented defaults (`permissions: contents: read`, `persist-credentials: false`), pin mutable action refs to commit SHAs, remove manual cargo `actions/cache` blocks, and add a `nix flake check` job matching SMT. Separately update `nix/modules/tools/treefmt.nix` so `rustfmt` runs at priority 10 and prettier/typos/smt/global excludes match SMT patterns that apply here.

**Tech Stack:** GitHub Actions, `dtolnay/rust-toolchain`, DeterminateSystems Nix installer + magic-nix-cache, `crate-ci/typos`, Nix flake + treefmt-nix, Sort-Markdown-Tables as reference (`/home/mr-fw16/Projects/Source-Codes/Sort-Markdown-Tables`).

## Global Constraints

- Approach 1 only: harden existing CI; do not add Codecov/tarpaulin; do not add `targets.yml` / `release.yml`; do not drop the multi-OS build/test matrix.
- Do not change [`.github/workflows/mega-linter.yml`](../../.github/workflows/mega-linter.yml).
- Pin actions to full commit SHAs with version comments (zizmor / supply-chain).
- Use these exact pins:
  - `actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1`
  - `dtolnay/rust-toolchain@e97e2d8cc328f1b50210efc529dca0028893a2d9 # v1`
  - `DeterminateSystems/nix-installer-action@ef8a148080ab6020fd15196c2084a2eea5ff2d25 # v22`
  - `DeterminateSystems/magic-nix-cache-action@908b263ff629f4cc17666315b7fd3ec127c6244d # v14`
  - `crate-ci/typos@8a48f81b6c64dcfea44b3633223084c4be58ac5f # v1.49.0`
  - Keep existing `actions/cache` only if somehow still needed — default is **delete all cargo cache steps**
  - Keep existing `actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a # v7.0.1` and `rustsec/audit-check@69366f33c96575abad1ee0dba8212993eecbe998 # v2.0.0`
- Commits must use Conventional Commits and include `Co-authored-by: Composer via Cursor <cursoragent@cursor.com>`.
- Prefer `rtk` prefix for shell commands when available.
- Do not copy SMT-only treefmt excludes that Profile-Pulse does not have (`_site/**`, `site/**`) unless those directories exist at implement time.

## File Structure

| File                                                                                                               | Responsibility                                                                     |
| ------------------------------------------------------------------------------------------------------------------ | ---------------------------------------------------------------------------------- |
| [`.github/workflows/ci.yml`](../../.github/workflows/ci.yml)                                                       | GitHub Actions CI: permissions, checkout hardening, pins, no cargo caches, nix job |
| [`nix/modules/tools/treefmt.nix`](../../nix/modules/tools/treefmt.nix)                                             | Formatter enablement, priorities, excludes                                         |
| [`docs/superpowers/specs/2026-08-22-ci-harden-treefmt-design.md`](../specs/2026-08-22-ci-harden-treefmt-design.md) | Short design record (written in Task 0)                                            |

---

### Task 0: Write design spec

**Files:**

- Create: `docs/superpowers/specs/2026-08-22-ci-harden-treefmt-design.md`

**Interfaces:**

- Consumes: Approved approach 1 from brainstorming (harden CI + treefmt; no Codecov; no release/targets)
- Produces: Spec document that Tasks 1–3 implement

- [ ] **Step 1: Create the design spec**

Write this exact file:

```markdown
# Design: CI Hardenings + Treefmt Priorities

Date: 2026-08-22
Status: Approved (approach 1)
Source of learnings: https://github.com/MRDGH2821/Sort-Markdown-Tables

## Problem

Profile-Pulse CI lacks SMT hardenings: no top-level `permissions`, checkout keeps credentials, `dtolnay/rust-toolchain@stable` and `crate-ci/typos@master` are mutable refs, manual cargo caches are fragile, and there is no `nix flake check` job. Treefmt enables rustfmt without `priority = 10`, so formatter order can fight other Rust formatters.

## Decision

Keep the existing CI job topology (including multi-OS build/test). Apply SMT security and Nix hygiene to those jobs. Align treefmt priorities/excludes with SMT where relevant to this repo. Do not port Codecov, release, or targets workflows.

## Changes

1. `.github/workflows/ci.yml` — permissions, persist-credentials, SHA pins, remove cargo caches, add nix job
2. `nix/modules/tools/treefmt.nix` — rustfmt priority 10; prettier/typos/sort-markdown-tables/global excludes

## Non-goals

- MegaLinter workflow rewrite
- Coverage reporting
- Cross-compilation release matrix
```

- [ ] **Step 2: Commit**

```bash
cd /home/mr-fw16/Projects/Source-Codes/Profile-Pulse
rtk git add docs/superpowers/specs/2026-08-22-ci-harden-treefmt-design.md
rtk git commit -m "$(cat <<'EOF'
docs(ci): record CI harden + treefmt design

Co-authored-by: Composer via Cursor <cursoragent@cursor.com>
EOF
)"
```

Expected: commit succeeds (hooks may reformat; if hooks modify the file, stage again and create a **new** commit — do not amend unless amend rules are fully met).

---

### Task 1: Harden CI workflow (permissions, pins, drop caches, nix job)

**Files:**

- Modify: `.github/workflows/ci.yml` (replace entire contents)
- Reference: `/home/mr-fw16/Projects/Source-Codes/Sort-Markdown-Tables/.github/workflows/ci.yml` (nix job shape)

**Interfaces:**

- Consumes: Action SHAs from Global Constraints; design from Task 0
- Produces: Hardened CI workflow used by GitHub Actions on push/PR to `main`/`develop`

- [ ] **Step 1: Snapshot current workflow for regression checks**

```bash
cd /home/mr-fw16/Projects/Source-Codes/Profile-Pulse
cp .github/workflows/ci.yml /tmp/profile-pulse-ci.yml.bak
rg -n 'dtolnay/rust-toolchain@stable|typos@master|actions/cache|persist-credentials|permissions:|nix flake' .github/workflows/ci.yml
```

Expected: shows `@stable`, `@master`, many `actions/cache`, and **no** `persist-credentials` / top-level `permissions` / nix job.

- [ ] **Step 2: Replace `.github/workflows/ci.yml` with the hardened workflow**

Write the full file as:

```yaml
---
env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1
jobs:
  build:
    name: Build Release
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
        with:
          persist-credentials: false
      - name: Install Rust
        uses: dtolnay/rust-toolchain@e97e2d8cc328f1b50210efc529dca0028893a2d9 # v1
        with:
          targets: ${{ matrix.target }}
      - name: Build release
        run: cargo build --release --verbose
      - name: Upload artifact
        uses: actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a # v7.0.1
        with:
          if-no-files-found: ignore
          name: profile-pulse-${{ matrix.os }}
          path: |
            target/release/profile-pulse
            target/release/profile-pulse.exe
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
          - os: windows-latest
            target: x86_64-pc-windows-msvc
          - os: macos-latest
            target: x86_64-apple-darwin
  check:
    name: Check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
        with:
          persist-credentials: false
      - name: Install Rust
        uses: dtolnay/rust-toolchain@e97e2d8cc328f1b50210efc529dca0028893a2d9 # v1
      - name: Check compilation
        run: cargo check --all-targets --all-features
  clippy:
    name: Clippy
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
        with:
          persist-credentials: false
      - name: Install Rust
        uses: dtolnay/rust-toolchain@e97e2d8cc328f1b50210efc529dca0028893a2d9 # v1
        with:
          components: clippy
      - name: Run clippy
        run: cargo clippy --all-targets --all-features -- -D warnings
  doc:
    name: Documentation
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
        with:
          persist-credentials: false
      - name: Install Rust
        uses: dtolnay/rust-toolchain@e97e2d8cc328f1b50210efc529dca0028893a2d9 # v1
      - env:
          RUSTDOCFLAGS: -D warnings
        name: Check documentation
        run: cargo doc --no-deps --all-features
  fmt:
    name: Rustfmt
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
        with:
          persist-credentials: false
      - name: Install Rust
        uses: dtolnay/rust-toolchain@e97e2d8cc328f1b50210efc529dca0028893a2d9 # v1
        with:
          components: rustfmt
      - name: Check formatting
        run: cargo fmt --all -- --check
  nix:
    name: Nix Flake Check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
        with:
          persist-credentials: false
      - uses: DeterminateSystems/nix-installer-action@ef8a148080ab6020fd15196c2084a2eea5ff2d25 # v22
      - uses: DeterminateSystems/magic-nix-cache-action@908b263ff629f4cc17666315b7fd3ec127c6244d # v14
      - name: Run Nix Flake Check
        run: nix flake check
  security-audit:
    name: Security Audit
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
        with:
          persist-credentials: false
      - name: Run cargo-audit
        uses: rustsec/audit-check@69366f33c96575abad1ee0dba8212993eecbe998 # v2.0.0
        with:
          token: ${{ secrets.GITHUB_TOKEN }}
  spell-check:
    name: Spell Check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
        with:
          persist-credentials: false
      - name: Check spelling
        uses: crate-ci/typos@8a48f81b6c64dcfea44b3633223084c4be58ac5f # v1.49.0
        with:
          files: .
  test:
    name: Test
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
        with:
          persist-credentials: false
      - name: Install Rust
        uses: dtolnay/rust-toolchain@e97e2d8cc328f1b50210efc529dca0028893a2d9 # v1
        with:
          toolchain: ${{ matrix.rust }}
      - name: Run tests
        run: cargo test --verbose --all-features
    strategy:
      matrix:
        os:
          - ubuntu-latest
          - windows-latest
          - macos-latest
        rust:
          - stable
name: CI
on:
  pull_request:
    branches:
      - main
      - develop
  push:
    branches:
      - main
      - develop
permissions:
  contents: read
```

- [ ] **Step 3: Verify hardening invariants**

```bash
cd /home/mr-fw16/Projects/Source-Codes/Profile-Pulse
# Must be empty / no matches:
rg -n 'rust-toolchain@stable|typos@master|actions/cache' .github/workflows/ci.yml || true
# Must all be present:
rg -n 'permissions:|persist-credentials: false|nix flake check|typos@8a48f81b6c64dcfea44b3633223084c4be58ac5f|rust-toolchain@e97e2d8cc328f1b50210efc529dca0028893a2d9' .github/workflows/ci.yml
# Every checkout must have persist-credentials on the next lines:
rg -n -A2 'actions/checkout@' .github/workflows/ci.yml
# Count jobs include nix:
rg -n '^(  )?[a-z].*:$' .github/workflows/ci.yml | head -40
```

Expected:

- No `@stable`, `@master`, or `actions/cache`
- Top-level `permissions: contents: read`
- Nine checkout blocks each followed by `persist-credentials: false`
- Jobs still include `build`, `check`, `clippy`, `doc`, `fmt`, `nix`, `security-audit`, `spell-check`, `test`

- [ ] **Step 4: Lint the workflow if tools are available**

```bash
cd /home/mr-fw16/Projects/Source-Codes/Profile-Pulse
command -v actionlint >/dev/null && actionlint .github/workflows/ci.yml || echo 'actionlint not on PATH; rely on treefmt/zizmor in Task 3'
command -v zizmor >/dev/null && zizmor .github/workflows/ci.yml || echo 'zizmor not on PATH; rely on treefmt later'
```

Expected: no errors if tools exist; otherwise note deferred to Task 3 `nix fmt` / treefmt zizmor.

- [ ] **Step 5: Commit**

```bash
cd /home/mr-fw16/Projects/Source-Codes/Profile-Pulse
rtk git add .github/workflows/ci.yml
rtk git commit -m "$(cat <<'EOF'
ci(github): harden workflow pins and add nix flake check

Port SMT learnings: contents:read, persist-credentials:false,
SHA-pinned rust-toolchain/typos, drop cargo caches, nix flake check.

Co-authored-by: Composer via Cursor <cursoragent@cursor.com>
EOF
)"
```

---

### Task 2: Align treefmt priorities and excludes

**Files:**

- Modify: `nix/modules/tools/treefmt.nix`
- Reference: `/home/mr-fw16/Projects/Source-Codes/Sort-Markdown-Tables/nix/treefmt.nix`

**Interfaces:**

- Consumes: Design from Task 0; SMT treefmt patterns
- Produces: Formatter config with `rustfmt.priority = 10` and aligned excludes

- [ ] **Step 1: Confirm current rustfmt and exclude gaps**

```bash
cd /home/mr-fw16/Projects/Source-Codes/Profile-Pulse
rg -n 'rustfmt|prettier =|sort-markdown-tables|typos =|global =|excludes' nix/modules/tools/treefmt.nix
```

Expected: `rustfmt.enable = true;` without priority; prettier has only `priority = 100`; typos excludes lack `.agents/logs/**`; global excludes only `**/skills/**`.

- [ ] **Step 2: Update `prettier` block**

In `nix/modules/tools/treefmt.nix`, replace:

```nix
    prettier = {
      enable = true;
      priority = 100;
    };
```

with:

```nix
    prettier = {
      enable = true;
      excludes = [
        # keep-sorted start
        "*.*ignore"
        "*.aac"
        "*.docx"
        "*.envrc"
        "*.gitkeep"
        "*.jinja"
        "*.jpg"
        "*.lock"
        "*.md"
        "*.mp4"
        "*.nix"
        "*.pdf"
        "*.png"
        "*.pptx"
        "*.py"
        "*.rs"
        "*.sh"
        "*.toml"
        "*.txt"
        "*.typ"
        ".envrc"
        "LICENCE"
        "LICENSE"
        "docs/**"
        "justfile"
        # keep-sorted end
      ];
      includes = ["*"];
      priority = 100;
    };
```

- [ ] **Step 3: Set rustfmt priority to 10**

Replace:

```nix
    rustfmt.enable = true;
```

with:

```nix
    rustfmt = {
      enable = true;
      priority = 10;
    };
```

- [ ] **Step 4: Update `sort-markdown-tables` excludes (keep priority 3)**

Replace:

```nix
    sort-markdown-tables = {
      enable = true;
      priority = 3;
    };
```

with:

```nix
    sort-markdown-tables = {
      enable = true;
      excludes = [
        "**/docs/**"
        "**/openspec/**"
        "docs/superpowers/**"
        "openspec/**"
        "tests/fixtures/**"
      ];
      priority = 3;
    };
```

- [ ] **Step 5: Extend `typos.excludes`**

Replace:

```nix
    typos = {
      enable = true;
      excludes = [
        # keep-sorted start
        "**/.cspell.json"
        ".cspell.json"
        "CHANGELOG.md"
        # keep-sorted end
      ];
    };
```

with:

```nix
    typos = {
      enable = true;
      excludes = [
        # keep-sorted start
        "**/.cspell.json"
        "**/.typos.toml"
        ".agents/logs/**"
        ".cspell.json"
        ".typos.toml"
        "CHANGELOG.md"
        # keep-sorted end
      ];
    };
```

- [ ] **Step 6: Extend `settings.global.excludes`**

Replace:

```nix
    global = {
      allow-missing-formatter = true;
      excludes = ["**/skills/**"];
    };
```

with:

```nix
    global = {
      allow-missing-formatter = true;
      excludes = [
        # keep-sorted start
        "**/openspec/**"
        "**/skills/**"
        "openspec/**"
        "tests/fixtures/**"
        # keep-sorted end
      ];
    };
```

- [ ] **Step 7: Verify treefmt file invariants**

```bash
cd /home/mr-fw16/Projects/Source-Codes/Profile-Pulse
rg -n 'rustfmt|priority = 10|includes = \["\*"\]|agents/logs|\*\*/openspec' nix/modules/tools/treefmt.nix
```

Expected: `rustfmt` block with `priority = 10`; prettier `includes = ["*"]`; typos and global include openspec/agents excludes.

- [ ] **Step 8: Commit**

```bash
cd /home/mr-fw16/Projects/Source-Codes/Profile-Pulse
rtk git add nix/modules/tools/treefmt.nix
rtk git commit -m "$(cat <<'EOF'
chore(treefmt): set rustfmt priority and align excludes

Match SMT learnings: rustfmt priority 10, prettier includes/excludes,
and openspec/typos/global exclude lists.

Co-authored-by: Composer via Cursor <cursoragent@cursor.com>
EOF
)"
```

---

### Task 3: Format and verify

**Files:**

- Modify: any files auto-touched by `nix fmt` / hooks (only if formatters change them)
- Verify: `.github/workflows/ci.yml`, `nix/modules/tools/treefmt.nix`

**Interfaces:**

- Consumes: Tasks 1–2 outputs
- Produces: Confirmed local formatting/evaluation; ready for PR/push

- [ ] **Step 1: Run formatter**

```bash
cd /home/mr-fw16/Projects/Source-Codes/Profile-Pulse
rtk nix fmt
```

Expected: exit 0; may rewrite YAML/Nix whitespace — review `rtk git diff`.

- [ ] **Step 2: Re-check CI invariants after format**

```bash
cd /home/mr-fw16/Projects/Source-Codes/Profile-Pulse
rg -n 'rust-toolchain@stable|typos@master|actions/cache' .github/workflows/ci.yml && echo 'FAIL: mutable refs or caches returned' || echo 'OK: no mutable refs/caches'
rg -n 'permissions:|nix flake check|persist-credentials: false' .github/workflows/ci.yml
rg -n 'priority = 10' nix/modules/tools/treefmt.nix
```

Expected: `OK: no mutable refs/caches`; permissions/nix/persist-credentials present; rustfmt priority 10 present.

- [ ] **Step 3: Evaluate flake (preferred) or at least formatter check**

```bash
cd /home/mr-fw16/Projects/Source-Codes/Profile-Pulse
# Full check can be long; start with eval/build of formatter if flake check is too heavy locally:
rtk nix flake check
```

Expected: exit 0. If `nix flake check` fails for unrelated pre-existing reasons, capture the failure log, fix only issues introduced by Tasks 1–2, and note pre-existing failures in the agent log.

- [ ] **Step 4: Commit formatter-only churn if any**

```bash
cd /home/mr-fw16/Projects/Source-Codes/Profile-Pulse
rtk git status
# If dirty after nix fmt:
rtk git add -u
rtk git commit -m "$(cat <<'EOF'
style: apply nix fmt after CI and treefmt updates

Co-authored-by: Composer via Cursor <cursoragent@cursor.com>
EOF
)"
```

If clean, skip the commit.

- [ ] **Step 5: Final status**

```bash
cd /home/mr-fw16/Projects/Source-Codes/Profile-Pulse
rtk git log --oneline -6
rtk git status
```

Expected: commits for design, CI, treefmt, and optional style; working tree clean (aside from intentional untracked files).

---

## Spec Coverage Checklist

| Requirement                                               | Task               |
| --------------------------------------------------------- | ------------------ |
| Top-level `permissions: contents: read`                   | Task 1             |
| `persist-credentials: false` on every checkout            | Task 1             |
| Pin `dtolnay/rust-toolchain` to SMT SHA                   | Task 1             |
| Pin `crate-ci/typos` (no `@master`)                       | Task 1             |
| Remove cargo `actions/cache` steps                        | Task 1             |
| Add `nix flake check` job with DeterminateSystems actions | Task 1             |
| Keep multi-OS build/test matrix                           | Task 1 (preserved) |
| Do not add Codecov / release / targets                    | Global Constraints |
| Leave MegaLinter workflow alone                           | Global Constraints |
| `rustfmt.priority = 10`                                   | Task 2             |
| Prettier includes/excludes from SMT                       | Task 2             |
| sort-markdown-tables excludes + keep priority 3           | Task 2             |
| typos + global openspec/agents excludes                   | Task 2             |
| Verify with `nix fmt` / `nix flake check`                 | Task 3             |
| Design doc recorded                                       | Task 0             |
