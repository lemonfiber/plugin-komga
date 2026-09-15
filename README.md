# plugin-komga

**Komga as a lemonfiber plugin.** Comics, manga and digital magazines: read in a
browser, and served over OPDS and Kobo sync to the reader apps you already use.

The bundled stack serves ebooks, audiobooks, film, television, music and
subtitles. Comics are the medium it acquires nothing for and serves nothing of,
which is what this fills.

## What it is

One manifest and a directory of recordings. Nothing here runs:

| Path | What |
| --- | --- |
| `plugin.toml` | The whole plugin: identity, the service, what it can do, how the stack reaches it, and the proofs that must pass before it installs |
| `fixtures/` | Recorded responses the proofs run against, so nobody needs a live Komga |
| `targets.toml` | The lemonfiber release this is validated and proved against |

## What the manifest declares

```toml
[[service]]
image       = "ghcr.io/gotson/komga"
digest      = "sha256:6c2a967bbe9acefd05933b2eb498f34afe96a83c6f7f8ab0acb512a1bb3ab50f"
tag         = "1.26.3"
port        = 25600
bind        = "lan"
health      = { kind = "http", path = "/actuator/health", timeout_s = 120 }
criticality = "enhancing"
takes_data  = true
media_types = ["comics"]
config_path = "/config"
provides    = ["komga:comics-serve", "komga:opds", "komga:kobo-sync"]

[wiring]
hostname        = "comics"
dashboard_group = "Library"
```

**The library it reads is the one the stack already fills.** `takes_data` mounts
the data root; `media_types = ["comics"]` is what points Komga at the comics part
of it rather than leaving the operator to find `/data/media/comics` in Komga's
own settings on first run. The stack's media-type vocabulary gained `comics` for
exactly this ([lemonfiber-media-stack#79](https://github.com/lemonfiber/lemonfiber-media-stack/pull/79)).

**It is wired, not just installed.** `[wiring]` is how the household reaches it:
`comics.your-domain` through the bundled proxy, and an entry in the dashboard's
Library group beside Calibre-Web and Audiobookshelf. lemonfiber writes both — a
plugin supplies no proxy stanza and no dashboard entry for the same reason it
supplies no container ([`F3-R31`](https://github.com/lemonfiber/spec/blob/main/10-functional/features/f-extensibility/f3-stack-manifests.md)).

**The tier governs the hostname, not the plugin.** `comics.your-domain` exists
because `bind = "lan"`. A loopback service gets no route and has no field to ask
for one.

**The digest is what runs.** The tag is a readable name for it and is never
resolved.

## What it can do, and why that currently wires nothing

`provides` is the capability model's plugin side (`F4-R1`): a service declares
what it can do so that wiring can ask for a capability rather than name a
service. Every claim here is namespaced with the plugin's id, because a plugin
may not invent a core-looking name (`F4-R4`).

**And a namespaced capability is inert until something asks for it.** Nothing
asks. The core vocabulary these would otherwise claim from — `F4-R2`, owned by
lemonfiber — is not published, so a plugin written today cannot make a claim that
wires anything.

That is a gap in the model rather than a choice here, and it is not left as a
sentence in a README: `.github/interim/vocabulary_gate.py` fails the day a
lemonfiber release publishes a vocabulary, so these three claims get read against
it instead of staying quietly inert.

## The proofs

Three, and every one asserts a **body**. None asserts only a status, because a
status is not an answer: Docker publishes a port by putting a proxy in front of
it, and that proxy accepts a connection before knowing whether anything inside is
listening. A manifest whose every proof reads only a status is refused
(`ARCH-R105`).

| Proof | What it establishes |
| --- | --- |
| `komga.serves` | The path the health probe asks for is one this image serves, answering `{"status":"UP"}` |
| `komga.answers-as-itself` | It is Komga behind that port: `/api/v1/claim` is Komga's own and reports its own state |
| `komga.library-is-guarded` | An anonymous reader on the household network is refused the library list. A refusal is the pass |

The third is the one worth having. This service is LAN-bound *and* reachable at
a name the whole household knows, and nothing else here would notice if Komga
stopped refusing that read: the health probe would still pass, the API would
still answer, and the shape of somebody's collection would be public.

## What was proved by running, and what only by validating

Proved by running, on `ghcr.io/gotson/komga@sha256:6c2a967b…`:

- the image starts as a non-root user against a config directory that is not
  root-owned, and opens its listener in 20.6s;
- all three proofs pass against the live container;
- all three report **unproven** — not failed, and not passed — against the same
  published port with the process replaced by `sleep infinity`;
- the digest resolves in the registry, `1.26.3` still names it, and **no
  signature is offered for it**, recorded as unproven rather than verified.

Proved only by validating:

- that the manifest conforms — including the wiring, the capability namespacing
  and the library targeting. There is no published schema to conform *to*, so it
  is checked against the contract document by hand, and
  `.github/interim/schema_gate.py` fails the day a real one is published.

**Not proved at all, and not claimed:** that lemonfiber installs this, that the
proxy stanza is generated, or that the dashboard entry appears. `0.16.0` is the
release that implements plugins and it is planned. What the manifest declares is
validated; what an installer would do with it has not been run, because there is
no installer.

## What is deliberately not here

No `[[secret]]` and no `[[override]]`. Komga creates its own administrator on
first run and lemonfiber captures nothing from it, and this plugin changes no
bundled setting. Both blocks exist in the format (`F3-R17`, `F3-R18`); a plugin
that holds nothing declares nothing.

No dashboard **widget**, only a link. A widget reads a service's API with a
credential, which is an adapter and a captured value — a recipe, arriving with
`F8` in `0.18.0`.

## Where this repository is not the catalogue

`lemonfiber-plugins` is the reviewed catalogue, and this is not it. This is a
plugin's **source** — what `F10-R9` says publishing requires and no more than: a
git repository. `F10-R7` is why its CI runs what it runs: the same commands the
catalogue's CI runs, so the first time a plugin meets them is not in somebody
else's pull request.

## Being official buys this nothing

Same schema validation, same digest pinning, same signature verification, same
proof runs as any plugin written by anybody. There is no trust bit and no
shortcut — and where the model would not do what this plugin needed, the model
was changed for everybody rather than bent for this one.

## Licence

The plugin data in this repository is under the Hippocratic License 3.0
(HL3-CORE) — see [LICENSE](LICENSE). Komga itself is MIT and is not distributed
here; this repository names an image, it does not contain one.
