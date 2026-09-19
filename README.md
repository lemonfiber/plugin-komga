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
provides    = ["media.serve", "komga:opds", "komga:kobo-sync"]

[[claim]]
capability = "media.serve"          # and the probes that demonstrate it

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

## What it can do, and what that now wires

`provides` is the capability model's plugin side (`F4-R1`): a service declares
what it can do so that wiring can ask for a capability rather than name a
service.

**`media.serve` is a core name**, out of the vocabulary lemonfiber publishes and
owns (`F4-R2`). It means the contracted thing that vocabulary defines, so
anything asking for a library server — rather than naming one — can be answered
by this plugin. Until that vocabulary existed this was `komga:comics-serve`, and
a namespaced capability is **inert until something asks for it**: the claim was
true, validated, and wired nothing.

A core capability is **demonstrated, not asserted**. The vocabulary says what has
to be shown and this manifest says where to ask it:

| Probe | Asked as | What it establishes |
| --- | --- | --- |
| `guarded` | nobody | An unauthenticated read of the catalogue is refused. A refusal is the pass. |
| `catalogue` | the operator | The same read answers with the series it holds, and holds at least one |

The second is asked with a credential, which a manifest cannot hold until recipes
arrive (`F8`). Against the recordings it runs like any other; against a live
service with nothing to present it is reported **unproven** — never failed,
because a runner that could not ask has established nothing about the service.

`komga:opds` and `komga:kobo-sync` stay this plugin's own, namespaced with its id
(`F4-R4`), and stay inert. Nothing in the stack asks for either, which is exactly
what a namespace is for — and it is why each carries a proof of its own below,
since nothing else would ever notice if one stopped being true.

## What it adds to what lemonfiber says

A plugin may put a row in a register lemonfiber already runs (`F3-R26`). The
doctor already runs checks independently, bounds each one, keeps `unverified`
distinct from `pass`, and carries a remedy on anything that does not pass. This
plugin adds two rows to it, attributed to the plugin wherever they appear.

| Check | What it notices | Remedy |
| --- | --- | --- |
| `komga:claimed` | Komga has no administrator, so the first caller on the household network becomes one | Claim it — and recreate the container if it has been reachable for any length of time |
| `komga:opds-guarded` | The OPDS catalogue answers a reader who presents nothing | Stop it and look at what is in front of it |

Both ask something no credential is needed for, and deliberately: a check that
could only be answered by signing in would report `unrun` on every doctor run
until recipes arrive, and a check that quietly never runs is worse than one that
fails.

`komga:opds-guarded` and the `komga.opds-is-mounted-and-guarded` proof ask almost
the same question, and the difference is the point. A proof is asked once, at
install. A check is asked every time the doctor runs — which is when an upgrade,
a settings change or a reverse proxy in front of the service could have stopped
it refusing what it used to refuse.

No code is contributed and none can be (`F3-R6`). There is nothing for
contributed code to *be*: the row is data and the engine that reads it is
lemonfiber's, unchanged.

## The proofs

Five, and every one asserts a **body**. None asserts only a status, because a
status is not an answer: Docker publishes a port by putting a proxy in front of
it, and that proxy accepts a connection before knowing whether anything inside is
listening. A manifest whose every proof reads only a status is refused
(`ARCH-R105`).

| Proof | What it establishes |
| --- | --- |
| `komga.serves` | The path the health probe asks for is one this image serves, answering `{"status":"UP"}` |
| `komga.answers-as-itself` | It is Komga behind that port: `/api/v1/claim` is Komga's own and reports its own state |
| `komga.library-is-guarded` | An anonymous reader on the household network is refused the library list. A refusal is the pass |
| `komga.opds-is-mounted-and-guarded` | OPDS is served *and* guarded, in one response: Komga answers an anonymous reader with an OPDS authentication document rather than a login page or a 404 |
| `komga.kobo-sync-refuses-an-unknown-key` | The Kobo sync endpoint exists. Asked with a key no account holds it refuses; an image without it answers 404 |

The third is the one worth having. This service is LAN-bound *and* reachable at
a name the whole household knows, and nothing else here would notice if Komga
stopped refusing that read: the health probe would still pass, the API would
still answer, and the shape of somebody's collection would be public.

## What was proved by running, and what only by validating

Proved by running, on `ghcr.io/gotson/komga@sha256:6c2a967b…`:

- the image starts as a non-root user against a config directory that is not
  root-owned, and opens its listener in 20.6s;
- all five proofs and both contributed checks pass against the live container,
  as does the `guarded` probe; the `catalogue` probe reports **unproven** against
  it, because it is asked with a credential this manifest cannot hold;
- everything declared passes against the recordings, which is what lets this run
  with no live Komga anywhere;
- the recordings for the core capability came off an instance with one library
  pointed at `/data/media/comics` and one comic in it, because a catalogue probe
  against an empty catalogue demonstrates the shape and not the serving;
- the digest resolves in the registry, `1.26.3` still names it, and **no
  signature is offered for it**, recorded as unproven rather than verified.

Proved only by validating:

- that the manifest conforms — including the wiring, the capability namespacing
  and the library targeting. It is held to lemonfiber's own generated schema,
  fetched off its default branch on every run, and then to the capability
  vocabulary and the extension points it publishes beside it.

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

No `[[recipe]]`. Komga needs no first-run flow driving from outside — it claims
itself, and the contributed check above is what notices when nobody has. The
block exists in the format and is checked; a manifest declaring one asks for
`recipe.run` by name, so a lemonfiber that cannot run one refuses the manifest
rather than parsing the block and skipping it.

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
