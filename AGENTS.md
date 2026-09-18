# AGENTS.md — plugin-komga

Guidance for any AI agent working in this repo.

> **Common rules for every lemonfiber repo are canonical in the spec:**
> [50-governance/ai-contributors.md](https://github.com/lemonfiber/spec/blob/main/50-governance/ai-contributors.md).
> Read them. This file is the `plugin-komga`-specific header only.

## What this repo is

A plugin's source: the manifest lemonfiber installs Komga from — which carries
the service, what it can do, how the stack reaches it and the proofs that must
pass — and the recorded responses those proofs run against. It is not the
reviewed catalogue — that is `lemonfiber-plugins` — and it is not a fork of the
stack. Contract:
[plugin-manifest](https://github.com/lemonfiber/spec/blob/main/20-architecture/contracts/plugin-manifest.md).

## The rules you cannot break

- **Nothing here executes.** A plugin is declarative data (`F3-R1`), and
  contributed code is never run, under any opt-in (`F3-R6`). The Python under
  `.github/interim/` is CI harness, is not part of what an operator installs,
  and is deleted when lemonfiber's own verbs replace it.
- **The plugin is `plugin.toml` and `fixtures/`.** Proofs live in the manifest,
  not beside it: an installer reads one file, and a proof the installer never
  reads cannot be what `F3-R4` refuses an install over.
- **The image is named by digest** (`F3-R8`). A tag is a name its publisher can
  repoint; it is carried beside the digest as a readable label and is never
  resolved. Moving the pin means re-recording every fixture against the new
  image in the same change — a recording that describes a different image is
  worse than no recording.
- **A proof asserts a body, never only a status.** Docker's port proxy accepts
  before anything inside is listening, so a status-only proof passes against a
  container that has been emptied.
- **A proof that could not be run is unproven** (`F3-R5`), never a pass. The
  three verdicts stay three.
- **No field beyond the contract's set.** A manifest carrying one is refused by
  name rather than ignored (`ARCH-R84`), so adding one does not extend the
  format — it breaks this plugin.
- **Nothing official about this plugin is a privilege.** If it ever needs one to
  work, that is a defect in the plugin model and belongs in a spec issue, not in
  a field here.

## What is deliberately absent

`[[secret]]` and `[[override]]`: Komga creates its own administrator and
lemonfiber captures nothing from it, and this plugin changes no bundled setting.
`[[recipe]]`, because nothing here needs one — and a manifest declaring one asks
for `recipe.run` by name, so a lemonfiber that cannot run one refuses it rather
than parsing the block and skipping it. A dashboard widget, because a widget
needs a credential and a credential needs a recipe (`F8`, `0.18.0`). A minimum
lemonfiber version, because `ARCH-R89` forbids one — `[requires].capabilities`
says what this needs, and `targets.toml` carries the CI pin, which is a different
fact.

## What it fills, and what it adds

`media.serve` is a **core** name out of lemonfiber's published vocabulary, and it
is what makes this plugin a candidate for anything that asks for a library server
rather than naming one. It was `komga:comics-serve` until that vocabulary
existed, and an inert claim is what it was. A core name is demonstrated rather
than asserted: the `[[claim]]` binds every probe the vocabulary declares for it,
and a binding weaker than the probe permits is refused naming the probe.

`komga:opds` and `komga:kobo-sync` stay this plugin's own. Nothing asks for
them, so nothing else would notice if they stopped being true — which is why each
carries a proof of its own here.

The two contributed checks ask something no credential is needed for, and
deliberately: a check that could only be answered by signing in would report
`unrun` on every doctor run until recipes arrive, and a check that quietly never
runs is worse than one that fails.

## The harness is not this repository's

`.github/interim/` is written in
[`plugin-template`](https://github.com/lemonfiber/plugin-template) and copied
here byte for byte. The `harness` job fails when the two differ. Change it there
and copy it here; changing it here fails.

## Checks

```
just ci        # every gate CI runs over this repository, in CI's order
just live      # the same proofs against a running instance
```

`just` lists the recipes `ci` is made of. The jobs it does not run are named in
the `justfile` beside the recipe, with what covers each.

`prove.py` is given `--against fixtures --report proofs.json`, which is what CI
runs it with. Without `--report` the assertions are proved and `proofs.json` is
left untouched, so a run that reports everything passing is still refused by
`git diff --exit-code proofs.json` — which says nothing but the diff.

## Before you open a PR

`just ci` turns this clone's git hooks on as its first step, and
`.githooks/commit-msg` then refuses a commit that CI would refuse — a
non-conventional subject, a missing sign-off, a missing `Spec:` citation, or a
trailer crediting an assistant. All four rules are in
[50-governance/contributing.md](https://github.com/lemonfiber/spec/blob/main/50-governance/contributing.md).
