# omarchy-onboard

Move your computer to [Omarchy](https://omarchy.org). Point it at the machine you're
leaving — a Mac today, Windows planned — and it works out what you had set up, proposes
the *equivalent* on Omarchy (install the package, not copy its files), tells you where
there is no equivalent, and applies what you accept.

You drive it from the Omarchy machine; the old computer only answers questions.

## Usage

On the computer you're leaving:

```sh
omarchy-onboard serve
```

On the Omarchy machine, with the code it prints:

```sh
omarchy-onboard migrate K7QT-3MZP
```

It discovers what is on the old machine, shows proposals grouped by area (packages, apps,
shell, …) with sensible defaults, and applies what you accept. Add `--dry-run` to
only plan. Offline: `scan` → `plan` → `apply`.

## Status

Early. Pairing, discovery, planning and file pulls work. Topics so far: Homebrew,
shell dotfiles, SSH keys and config.

## Layout

See [AGENTS.md](AGENTS.md).

## License

MIT
