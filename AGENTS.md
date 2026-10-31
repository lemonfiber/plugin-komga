# AGENTS.md — plugin-komga

> **Start at the roadmap and board on [lemonfiber.app](https://lemonfiber.app),
> rendered from the report of where every unreleased version stands. Then the
> rules** every repository shares:
> [working in the repositories](https://github.com/lemonfiber/spec/blob/main/50-governance/working-in-the-repositories.md)
> and [the rules for agents](https://github.com/lemonfiber/spec/blob/main/50-governance/ai-contributors.md).
> This file holds only what is true of this repository.

## What this repo is

Komga as a first-party lemonfiber plugin: `plugin.toml` and the adapter that
speaks the `owner.claim@1` and `media.serve@1` contracts in front of Komga. The
adapter is the Rust crate at the root, built on `lemonfiber-adapter-kit` from the
core at the commit `Cargo.toml` pins, into the image the `Dockerfile` builds.
`release.yml` publishes that image from every commit on `main` and from the
train's tag, through spec's reusable `adapter-release`. `plugin.toml` runs Komga
alone and does not run the adapter yet; its proofs' recordings are under
`fixtures/`.

## The rules you cannot break

- **The plugin id is the service's id** (`ARCH-R266`).
- **No Python here.** The reader is spec's `first-party-plugin` workflow's.
- **No `unwrap`, `expect`, `panic` or indexing** outside tests. The lints in
  `Cargo.toml` are the core's.
- **The adapter never writes its credentials.** lemonfiber mints the
  administrator's address and password and writes them into the adapter's
  `upstream.json` as `username` and `password` (`ARCH-R315`, `ARCH-R317`). The
  address is minted as `"email"`, the shape Komga's claim accepts. The adapter
  reads them for every sign-in, carries the session Komga opens afterwards, and
  never logs them.
- **A claim changes nothing Komga already holds** (`ARCH-R328`): only a Komga
  holding no account is claimed, through its own claim API, and one claimed by
  anybody else is answered as theirs. The claim that makes the administrator
  then gives Komga a `Comics` library at `/data/media/comics`, unless one already
  reads that folder.
- **Nothing is asked for a member.** Komga does not speak `identity.source`, so
  every `media.serve` operation naming a member answers `unsupported`, as does
  everything else it cannot do (`ARCH-R334`).
- **A key is found again by Komga's own record of its sign-in.** Komga shows a
  key once and refuses two under one name, so a second key for an app is filed as
  `<app> #2`, and `revoke` reads the key's id from the authentication activity
  Komga records for one request the key signs in, by the user agent that request
  alone carried.
- **The core's crates move with the train.** The `rev` in `Cargo.toml` is the
  core commit `targets.toml` names, the same for every `lemonfiber-*` crate.
- **Moving Komga's digest** means recording every response again against the new
  image in the same change, moving `RELEASE` and `DIGEST` in `src/adapter.rs`
  with it, and committing the `proofs.json` the checks write.

## Checks

```
just ci
```

The gates that need nothing but this tree. The plugin verdict and the
recordings run in CI; `just recordings` takes the recordings again locally, with
Docker.
