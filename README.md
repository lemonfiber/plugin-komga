# Komga for lemonfiber

**Read comics, manga and digital magazines in a browser, and on the reading apps
you already use.** This plugin adds [Komga](https://github.com/gotson/komga) to
your lemonfiber stack, where everyone on your home network can reach it.

Without it, comics stay folders of images, opened one file at a time with
nothing remembering where you were.

## What you get

- **Komga 1.26.3**, running as a service in your stack.
- **An address the household can remember:** `comics.<your domain>`, through the
  stack's proxy. Komga also answers on port 25600 of this machine.
- **A link on the stack's dashboard**, in the Library group.
- **An OPDS catalogue and Kobo sync**, for reading apps and Kobo e-readers. Both
  ask for a Komga account.
- **Two more checks in `lemonfiber doctor`**, each with a fix when it fails:

| Check | What it notices |
| --- | --- |
| Komga has an administrator, so nobody else can become one | Nobody has created Komga's administrator account yet, so the first person on your network to open Komga becomes the administrator |
| The OPDS catalogue is not readable by the household network | Anyone on your network can list and download the library through OPDS without signing in |

## What it needs

- **A lemonfiber with the `lemonfiber plugin` commands.** lemonfiber 0.15.0 and
  earlier do not have them. `lemonfiber version` says which version you have.
- **A stack lemonfiber has already set up on this machine.** Run
  `lemonfiber setup` first if you have not. Installing a plugin is refused on a
  machine with no stack.
- **Port 25600 free** on this machine.
- **Disk space** for the Komga image, and for Komga's own data in
  `config/komga` in your stack directory. Your comics stay in your data root.
- **Access to `ghcr.io`** the first time Komga starts, to download the image.
- **No accounts anywhere else.** You create Komga's administrator account in
  Komga itself.

## Install

Get a copy of this repository, then ask lemonfiber what installing it would do.
`--dry-run` settles everything a real install settles and writes nothing:

```sh
git clone https://github.com/lemonfiber/plugin-komga.git
lemonfiber plugin install plugin-komga --dry-run
lemonfiber plugin install plugin-komga
```

The [lemonfiber plugin catalogue](https://github.com/lemonfiber/lemonfiber-plugins/blob/main/plugins/komga.toml)
records the revision of this repository that a person reviewed, as `revision`.
To install exactly that, run `git -C plugin-komga checkout <revision>` before
you install.

When you install, lemonfiber:

1. Reads `plugin.toml` and checks all of it. If anything in it does not
   conform, it names every problem at once and changes nothing.
2. Writes Komga's container, its configuration directory, a `comics` site for
   the proxy and a dashboard entry, and records each change as it makes it.
3. Starts Komga and asks it the questions this plugin declares, to show it is
   Komga, that it answers, and that it refuses to show the library to someone
   who has not signed in.
4. Runs the stack's own checks before and after, and compares the two.
5. Records Komga as installed only when all of that holds. If anything fails, it
   puts back everything it wrote, and your machine is as it was.

`lemonfiber plugin installed` lists what is installed, where each plugin came
from, and how each of its services is reached.

## Set it up

1. **Create the administrator account first.** Open Komga at
   `comics.<your domain>`, or at port 25600 of this machine, and create the
   account before anyone else does. Until somebody does, the first person on
   your network to open Komga becomes its administrator, and `lemonfiber doctor`
   reports it.
2. **Add your comics as a library.** Komga sees your data root at `/data`. Put
   your comics in a folder there, for example `media/comics`, and add
   `/data/media/comics` as a library in Komga.
3. **Connect your reading apps.** Komga's OPDS catalogue is at
   `/opds/v2/catalog` on the same address. A Kobo e-reader syncs with an API
   key that belongs to a Komga account.

## What it changes on your machine

Everything below is in your stack directory, or is a setting of the Komga
container lemonfiber writes:

| What | Where |
| --- | --- |
| Komga's container | `compose/plugins/komga.yml`, in a Compose profile of its own, `plugin-komga` |
| Komga's own data | `config/komga`, which Komga sees as `/config` |
| Your library | Your whole data root, which Komga sees as `/data`, with permission to write |
| Network | Port 25600 on the address your household services use (`LAN_BIND`, every interface unless you narrowed it) |
| Proxy | A `comics` site in `config/caddy/Caddyfile` |
| Dashboard | A Komga link in the Library group of `config/homepage/services.yaml` |

A plugin cannot ask for more than this. lemonfiber writes the container itself,
and the plugin format has no field for another mount, another address or a line
of proxy configuration.

## What it sends anywhere

- **The image.** lemonfiber downloads Komga from `ghcr.io/gotson/komga`, pinned
  to one exact build
  (`sha256:6c2a967bbe9acefd05933b2eb498f34afe96a83c6f7f8ab0acb512a1bb3ab50f`).
  If you have switched off downloading with `LEMONFIBER_REACH_REGISTRY`, nothing
  is downloaded and the image has to be on this machine already.
- **Nothing else from this plugin.** It names no host outside your stack,
  captures no password or key, and changes no setting of the services lemonfiber
  bundles. Its doctor checks ask only your own Komga.
- **Komga itself.** What Komga connects to is Komga's own behaviour. This
  plugin does not configure it.

## Update

Get the newer version of this repository, with `git pull` or by checking out
the newer revision the catalogue lists, then update from it:

```sh
git -C plugin-komga pull
lemonfiber plugin update plugin-komga --dry-run
lemonfiber plugin update plugin-komga
```

The new version is checked and proved the way an install is. Your machine is on
the old version or the new one at every moment. If the new one does not hold,
lemonfiber puts the old one back and says which version you are on. Komga's
own data in `config/komga` stays where it is.

## Remove

```sh
lemonfiber plugin remove komga --dry-run
lemonfiber plugin remove komga
```

Removing stops Komga and takes out everything the install wrote: the container,
the proxy site and the dashboard link. If you have edited either of those by
hand since, lemonfiber refuses rather than overwrite your edit.

Two things stay:

- **Komga's own data** in `config/komga`. lemonfiber does not delete a directory
  holding something it did not put there, and it lists what it left.
- **Your comics.** Installing never wrote to your data root, so removing has
  nothing there to take back.

## Getting help

- **A question:** ask on [Discord](https://discord.nightworks.io).
- **Something wrong with installing, updating, removing or the doctor checks:**
  [open an issue on this repository](https://github.com/lemonfiber/plugin-komga/issues).
  `lemonfiber support` shows what a support bundle would hold, with passwords
  and keys replaced; `lemonfiber support --write` writes it for you to attach.
  It sends nothing anywhere by itself.
- **Something wrong in Komga itself:** [Komga's own project](https://github.com/gotson/komga).

## Licence

The plugin data in this repository is under the Hippocratic License 3.0
(HL3-CORE); see [LICENSE](LICENSE). Komga itself is under the MIT licence and
is not distributed here: this repository names an image, it does not contain
one.

## Working on this plugin

How the manifest is built, what each proof establishes, and how to run the
checks CI runs: [docs/development.md](docs/development.md).
