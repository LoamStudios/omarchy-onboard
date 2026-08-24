# omarchy-migrate

Move from a Mac to [Omarchy](https://omarchy.org). Discovers what's set up on the
source machine, proposes the *semantically equivalent* thing on Omarchy (install
the package, not copy its files), and applies what you accept.

Runs on the Omarchy machine and pulls from the Mac; the Mac only reports.

## Status

Early. Offline flow works on a single machine:

```sh
mise install
mise run plan      # scan this machine → interactive proposals → plan.json
```

Pairing over the network (`serve` / `migrate`) is not wired yet.

## Layout

See [AGENTS.md](AGENTS.md).

## License

MIT
