# plugin-komga

**Komga as a lemonfiber plugin.** Comics, manga and digital magazines: read in a
browser, and served over OPDS and Kobo sync to the reader apps you already use.

The bundled stack serves ebooks, audiobooks, film, television, music and
subtitles. Comics are the medium it acquires nothing for and serves nothing of,
which is what this fills.

## What it is

Three files and a directory, and none of it runs:

| Path | What |
| --- | --- |
| `plugin.toml` | The manifest. Identity, the one service, and what it needs of lemonfiber |
| `proofs.toml` | The proofs it declares, and why each one is worth asserting |
| `fixtures/` | Recorded responses the proofs run against, so nobody needs a live Komga |
| `targets.toml` | The lemonfiber release this is validated and proved against |

A plugin is declarative data. There is no code here that lemonfiber executes,
and no field in the format by which there could be.

## What the manifest declares

```toml
image  = "ghcr.io/gotson/komga"
digest = "sha256:6c2a967bbe9acefd05933b2eb498f34afe96a83c6f7f8ab0acb512a1bb3ab50f"
tag    = "1.26.3"
port   = 25600
bind   = "lan"
health = { kind = "http", path = "/actuator/health", timeout_s = 120 }
criticality = "enhancing"
takes_data  = true
forms  = ["library", "full"]
```

**The digest is what runs.** The tag is a readable name for it and is never
resolved — moving the tag upstream changes nothing here, which is the whole
point of pinning one.

**`bind = "lan"` is a tier, not an address.** lemonfiber assigns the address.
A plugin that could write its own could put an admin surface on the household
network without touching anything the security rules inspect.

**`takes_data = true`** is what mounts the library. Point Komga at
`/data/media/comics` on first run — the same first-run step Calibre-Web takes in
the bundled stack, and for the same reason: the manifest has no field for a
library root, and a service that discovers its own is one lemonfiber does not
have to be taught about.

## The proofs

Three, and every one asserts a **body**. None asserts only a status, because a
status is not an answer: Docker publishes a port by putting a proxy in front of
it, and that proxy accepts a connection before knowing whether anything inside
is listening. A container replaced by `sleep infinity` answers a bare connect
exactly as the real one does.

| Proof | What it establishes |
| --- | --- |
| `komga.serves` | The path the health probe asks for is one this image serves, and answers `{"status":"UP"}` |
| `komga.answers-as-itself` | It is Komga behind that port, not something else: `/api/v1/claim` is Komga's own and reports its own state |
| `komga.library-is-guarded` | An anonymous reader on the household network is refused the library list. A refusal is the pass |

The third is the one worth having. This service is LAN-bound by declaration, and
nothing else here would notice if Komga stopped refusing that read: the health
probe would still pass, the API would still answer, and the shape of somebody's
collection would be public.

## What was proved by running, and what only by validating

Proved by running, on `ghcr.io/gotson/komga@sha256:6c2a967b…`:

- the image starts as a non-root user against a config directory that is not
  root-owned, and opens its listener in 20.6s;
- all three proofs pass against the live container;
- all three report **unproven** — not failed, and certainly not passed —
  against the same published port with the process replaced by `sleep infinity`;
- the digest resolves in the registry, `1.26.3` still names it, and **no
  signature is offered for it**, which is recorded as unproven rather than as
  verified.

Proved only by validating:

- that the manifest conforms. There is no published schema to conform *to* —
  see below — so it is checked against the contract document by hand.

**Not proved at all, and not claimed:** that lemonfiber installs this. The
release that implements plugins is `0.16.0` and it is planned. Nothing here has
been installed, rehearsed or removed, because there is nothing yet to do it.

## Three things the plugin format could not express

Recorded because a gap nobody wrote down is a gap the next author rediscovers.

**There is no block for proofs.** `F3-R1` lists a plugin's proofs among what its
manifest declares and `F3-R4` refuses to install a plugin whose proofs fail —
but `plugin.toml` at `schema_version = 1` has only `[plugin]`, `[[service]]` and
`[requires]`, and an unrecognised declaration must be *refused* rather than
skipped. A `[[proof]]` block in the manifest today would make this plugin
uninstallable. So they live in `proofs.toml`, in the shape they would take in
the manifest, and move into it unchanged when the block exists.

**There is no published schema.** `F3-R2` requires validation against one and
`ARCH-R92` requires it to be generated from lemonfiber's own types rather than
written. None is published, so `.github/interim/validate.py` checks the contract
by hand — deliberately not published as a schema, because `F10-R2` forbids a
second description of this format standing beside the generated one.
`.github/interim/schema_gate.py` is the register that ends the arrangement: it
fails the day a release publishes the real thing.

**There is no media type for comics.** The vocabulary is `tv`, `movies`, `music`
and `books`, so this plugin declares no `media_types` at all rather than
misfiling comics as books. It costs nothing today and would cost something the
moment anything sorted a library by medium.

## Where this repository is not the catalogue

`lemonfiber-plugins` is the reviewed catalogue, and this is not it. This is a
plugin's **source** — the thing `F10-R9` says publishing requires and no more
than: a git repository. The catalogue holds a reviewed copy; an operator may
install from either, and `F5-R6` requires the technical validation to be
identical for both.

Which is also why this repository runs the checks it does. `F10-R7` asks for a
plugin whose own CI runs the same commands the catalogue's CI runs, so that the
first time a plugin meets them is not in somebody else's pull request.

## Being official buys this nothing

Same schema validation, same digest pinning, same signature verification, same
proof runs as any plugin written by anybody. There is no trust bit, no shortcut
and nothing here asks for one.

## Licence

The plugin data in this repository is under the Hippocratic License 3.0
(HL3-CORE) — see [LICENSE](LICENSE). Komga itself is MIT and is not distributed
here; this repository names an image, it does not contain one.
