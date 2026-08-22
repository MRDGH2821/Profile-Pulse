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
