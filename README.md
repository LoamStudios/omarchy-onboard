# omarchy-onboard

Move your computer to [Omarchy](https://omarchy.org).

Point it at the machine you're leaving and it works out what you had set up, proposes the
*equivalent* on Omarchy — install the package, not copy its files — tells you where there
is no equivalent and what people use instead, and applies what you accept.

You drive it from the Omarchy machine. The old computer only answers questions; it never
decides anything, and only the files you accept ever leave it.

## Usage

On the computer you're leaving:

```sh
omarchy-onboard serve
```

On the Omarchy machine, with the code it prints:

```sh
omarchy-onboard migrate K7QT-3MZP
```

Three phases:

1. **Discover** — what's on the old machine, summarised by area.
2. **Propose** — a checklist per area (packages, apps, shell, keys, editors, fonts, …), each
   row with a one-line reason, defaults pre-ticked. Things that need no action — already part
   of Omarchy, not needed on Linux, or with no equivalent — are listed as notes.
3. **Migrate** — runs what you accepted and reports each step.

`--dry-run` stops after Propose. Offline: `scan` on the old machine → copy `discovery.json` →
`plan` → `apply` on Omarchy.

## What it covers

| Area | Reads from | Does on Omarchy |
|---|---|---|
| Packages & apps | Homebrew | `pacman` / AUR installs; notes for the rest |
| Shell | login shell, dotfiles | copies dotfiles |
| SSH | keys, `config`, `known_hosts` | copies keys (0600); rewrites config without platform-only options |
| Keyboard & pointer | key repeat, scroll direction, tap to click, Caps Lock remap | Hyprland `input` block |
| Editors | VS Code settings, keybindings, snippets, extensions | writes config; installs extensions through VS Code |
| Terminal | Ghostty, Alacritty, Kitty, WezTerm config | rewrites config for Linux; checks the font is coming too |
| Fonts | fonts you installed | installs the packaged ones, copies the rest |

Source platforms: macOS today. Windows is planned — the design keeps platform-specific reading
separate from the Omarchy-side proposing, so each area gains a Windows reader without touching
the rest. See [AGENTS.md](AGENTS.md) for how an area (a *topic*) is written.

## Status

Early. Pairing over the local network, discovery, planning, package installs and file pulls
work end to end. Mapping tables are small and grow with use.

## License

MIT
