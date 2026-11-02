# loudness-looks-magnificent

A native, OS-agnostic audio visualizer: it renders **the system's own audio
output buffer** — whatever your computer is playing — into real-time GPU
visuals. Built for commodity Linux hardware first. Written exclusively by LLM
coding agents under human product supervision (hence the name).

## Status

🚜 **Spec-first, pipeline live.** PR #1 shipped the founding documents; PR #2
wired the local build/test/e2e loop (hooks + lint gates + `just verify`);
CI, release automation, and agent commands come next. Read in this order:

| Read | What it is |
| ---- | ---------- |
| [`docs/PRODUCT.md`](docs/PRODUCT.md) | Product spec: functional + nonfunctional requirements (`FR-*`, `NFR-*`), MVP definition, milestones |
| [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) | System structure: trait seams (audio capture, rendering, presets), threading, data flow |
| [`docs/CONVENTIONS.md`](docs/CONVENTIONS.md) | Engineering rules: lint policy, error handling, unsafe/unwrap bans, docs, tests, commits |
| [`AGENTS.md`](AGENTS.md) | The agent constitution: golden loop, definition of done, subagents, change control |

## Quickstart (identical for humans and agents)

```sh
rustup update   # rust-toolchain.toml pins the exact toolchain version
just setup      # installs the git hooks (co-author, commit grammar, gates)
just verify     # local CI mirror: fmt + clippy + tests + e2e
cargo run -- --headless --fake-audio 7 --frames 180 --fps-report
```

The headless loop (deterministic synthetic source → loudness → scale mapping →
frame counter) is the bootstrap render path; window rendering arrives with the
M1 milestone (FR-M1-1).

## Highlights of the spec

- **v1.0.0-alpha.1 (MVP)** — a single GPU-rendered cube that grows and shrinks
  with system audio loudness, Linux (`winit` + `wgpu`), zero local `unsafe`.
- **60 fps floor and ≤ 50 ms audio→visual latency are hard MUSTs** (NFR-1,
  NFR-2), with defined measurement procedures and CI-runnable headless e2e.
- **Host-agnostic by construction** — audio capture, rendering, and preset
  persistence sit behind Rust traits; new OSes are new implementations, not
  rewrites (NFR-4).
- **M2** adds shape selection, per-vertex/edge audio mappings (loudness, band
  energy, peak), palettes, and TOML presets with a lightweight UI.

## Agent-first development

Every change flows through the same automated loop: requirements → failing
tests → implementation → local verification → co-authored conventional commit
→ PR → CI gates (formatting, clippy, unit + e2e, coverage, secret scan) →
auto-merge → automated release. Humans specify, review, and steer; agents do
the rest. The rules of engagement for agents live in [`AGENTS.md`](AGENTS.md).

## License

Released to the **public domain** via the [Unlicense](LICENSE) — no copyright,
no attribution, use it freely for anything.
