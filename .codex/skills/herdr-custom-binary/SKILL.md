---
name: herdr-custom-binary
description: Use to rebuild and reinstall custom herdr fork. Use only when rebuilding or reinstall local hrd/custom herdr binary.
---

# Herdr Custom Binary

Use this skill only inside `/Users/maxktz/dev/herdr`.

Max keeps two Herdr entrypoints:

- `herdr`: official installed binary at `~/.local/bin/herdr`, default session, state in `~/.config/herdr/`
- `hrd`: wrapper at `~/.local/bin/hrd`, custom session, shared config, state in `~/.config/herdr/sessions/custom/`

The real custom binary is `~/.local/bin/hrd-bin`. Do not overwrite the `hrd` wrapper when rebuilding.

## Check Current Setup

```sh
command -v herdr hrd hrd-bin
ls -l ~/.local/bin/herdr ~/.local/bin/hrd ~/.local/bin/hrd-bin
herdr status --json
hrd status --json
hrd session list --json
```

Expected split:

- `herdr` talks to `/Users/maxktz/.config/herdr/herdr.sock`
- `hrd` talks to `/Users/maxktz/.config/herdr/sessions/custom/herdr.sock`
- `hrd` client binary should show `~/.local/bin/hrd-bin`

## Rebuild And Reinstall

The vendored terminal library requires Zig 0.15.2. Plain `cargo build` may pick up Zig 0.16 and fail.

```sh
cd /Users/maxktz/dev/herdr
ZIG=/opt/homebrew/Cellar/zig@0.15/0.15.2/bin/zig cargo build --release --locked
install -m 0755 target/release/herdr ~/.local/bin/hrd-bin
```

After installing, hand off the custom session to the rebuilt binary:

```sh
hrd server live-handoff --import-exe ~/.local/bin/hrd-bin --expected-protocol 14 --expected-version 0.7.0
```

Verify:

```sh
hrd status server --json
lsof -U 2>/dev/null | rg '/Users/maxktz/.config/herdr/sessions/custom/herdr\\.sock'
```

## Update From Upstream

Max's local customizations live on local `master` as commits on top of `upstream/master`.

```sh
cd /Users/maxktz/dev/herdr
git fetch upstream --prune
git rebase upstream/master
```

Then rebuild, install to `hrd-bin`, and live-handoff as above.

## Neovim Integration

If the built-in Neovim integration asset changed, reinstall it from the custom binary:

```sh
hrd-bin integration install neovim
hrd-bin integration status | rg '^neovim|neovim'
```

The plugin file is global at `~/.config/nvim/plugin/herdr-navigator.lua`, but it must be inert outside Herdr panes. Check both cases:

```sh
env -u HERDR_ENV -u HERDR_SOCKET_PATH -u HERDR_PANE_ID -u HERDR_BIN_PATH \
  nvim --headless +'redir => g:out | silent verbose nmap <C-h> | redir END | call writefile(split(g:out, "\n"), "/tmp/herdr-nvim-outside.txt")' +'qa'
cat /tmp/herdr-nvim-outside.txt

HERDR_ENV=1 HERDR_SOCKET_PATH=/tmp/herdr.sock HERDR_PANE_ID=w1:p1 HERDR_BIN_PATH=$HOME/.local/bin/hrd-bin \
  nvim --headless +'redir => g:out | silent verbose nmap <C-h> | redir END | call writefile(split(g:out, "\n"), "/tmp/herdr-nvim-inside.txt")' +'qa'
cat /tmp/herdr-nvim-inside.txt
```

Expected:

- outside Herdr: `No mapping found`
- inside Herdr env: mapping comes from `herdr-navigator.lua`

## Guardrails

- Do not run `hrd update`; it downloads official release binaries and is not the custom fork update path.
- Do not install the custom build to `~/.local/bin/herdr` unless Max explicitly asks to replace the official binary.
- Do not replace `~/.local/bin/hrd`; it is the wrapper that injects `--session custom`.
- Prefer live handoff over `server stop` because stopping a server exits panes.
