# openconnect-tui

A terminal UI for GlobalProtect VPN using [globalprotect-openconnect](https://github.com/yuezk/GlobalProtect-openconnect) (`gpclient`).

## Features

- Connect/disconnect GlobalProtect VPN from a clean TUI
- Browser-based SAML authentication
- HIP report submission support (`--csd-wrapper`)
- Persistent config (server, browser, HIP script path)
- Connection logs at `~/.local/state/openconnect-tui/gpclient.log`

## Installation

### AUR (Arch Linux)

```bash
yay -S openconnect-tui-git
```

### Build from source

```bash
git clone https://github.com/EasyCanadianGamer/openconnect-tui
cd openconnect-tui
cargo build --release
sudo install -Dm755 target/release/openconnect-tui /usr/bin/openconnect-tui
```

**Requires:** `globalprotect-openconnect` (`gpclient`)

## Setup

### Passwordless sudo (required)

`gpclient` requires root to manage the VPN tunnel. Without a sudoers rule, sudo will
prompt for a password mid-session which breaks the TUI.

```bash
sudo visudo -f /etc/sudoers.d/zzz-gpclient
```

Add this line (replace `your_username`):

```
your_username ALL=(ALL) NOPASSWD: /usr/bin/gpclient
```

> The `zzz-` prefix ensures this rule sorts **last** in `/etc/sudoers.d/`, so it
> overrides any earlier rules (like the wheel group) that would re-require a password.

### HIP Report (optional)

If your VPN requires HIP report submission, set the CSD wrapper path in the
Settings tab (F2):

```
/usr/lib/gpclient/hipreport.sh
```

This is included with the `globalprotect-openconnect` package.

## Usage

```bash
openconnect-tui
```

| Key | Action |
|-----|--------|
| F1 | Connect tab |
| F2 | Settings tab |
| F3 | About tab |
| Enter | Connect / Disconnect |
| q | Quit |

### Logs

```bash
tail -f ~/.local/state/openconnect-tui/gpclient.log
```

## License

MIT
