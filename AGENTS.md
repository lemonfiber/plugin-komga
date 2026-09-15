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
A dashboard widget, because a widget needs a credential and a credential needs a
recipe (`F8`, `0.18.0`). A minimum lemonfiber version, because `ARCH-R89` forbids
one — `[requires].capabilities` says what this needs, and `targets.toml` carries
the CI pin, which is a different fact.

Every `provides` entry is namespaced and therefore inert, because `F4-R2` has not
published a core vocabulary to claim from. Do not "fix" that by inventing a
core-looking name; `vocabulary_gate.py` fails when the real one lands.

## Checks

```
python3 .github/interim/validate.py --self-test   # the gate refuses what it should
python3 .github/interim/validate.py              # the manifest against the contract
python3 .github/interim/prove.py                  # the manifest's proofs, against the recordings
python3 .github/interim/prove.py --against http://127.0.0.1:25600   # against a live one
python3 .github/interim/image_gate.py             # the digest, its tag, its signature state
python3 .github/interim/schema_gate.py            # fails the day the real schema lands
python3 .github/interim/vocabulary_gate.py        # fails the day the capability vocabulary lands
```

## Before you open a PR

- Cite a spec identifier in a commit `Spec:` trailer and the PR body.
- No AI attribution in commits.
