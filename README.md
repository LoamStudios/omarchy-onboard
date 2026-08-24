# omarchy-onboard

Move from a Mac to [Omarchy](https://omarchy.org). Discovers what's set up on the
source machine, proposes the *semantically equivalent* thing on Omarchy (install
the package, not copy its files), and applies what you accept.

Runs on the Omarchy machine and pulls from the Mac; the Mac only reports.

## Usage

On the Mac:

```sh
omarchy-onboard serve
```

On the Omarchy machine, with the code it prints:

```sh
omarchy-onboard migrate K7QT-3MZP
```

It discovers what is on the Mac, shows proposals grouped by area (packages, apps,
shell, …) with sensible defaults, and applies what you accept. Add `--dry-run` to
only plan. Offline: `scan` → `plan` → `apply`.

## Status

Early. Pairing, discovery, planning and file pulls work. Topics so far: Homebrew,
shell dotfiles, SSH keys and config.

## Layout

See [AGENTS.md](AGENTS.md).

## License

MIT
