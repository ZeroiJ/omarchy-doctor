# 🔧 omadoctor

> Never ask on Discord again. Your system fixes itself.

A fast, terminal-based diagnostic and repair tool for [Omarchy Linux](https://omarchy.org). Detect issues automatically, apply fixes with one keystroke, or search GitHub for community solutions.

![Demo](demo.gif)

## Features

- 🎯 **Automatic Detection** — Run shell commands to check if issues exist
- 🔧 **One-Key Fixes** — Press `f` to apply verified fixes with confirmation
- 🌐 **GitHub Integration** — Search `basecamp/omarchy` issues for community solutions
- 📦 **Auto-Updates** — Fix database automatically updates from GitHub Releases
- 📊 **Progress Bar** — Animated progress while applying fixes
- 🛡️ **Safe by Default** — Confirmation dialog before running any fix command
- 📝 **TOML-Based** — Easy to extend with new fixes
- 🖥️ **Beautiful TUI** — Ratatui-powered interface with emoji category icons
- 🔍 **Scan Mode** — Non-interactive batch scanning for CI/scripts

## Installation

### From AUR (Arch Linux) - Recommended

```bash
yay -S omadoctor-bin
# or
paru -S omadoctor-bin
```

This installs the pre-built binary with all fix definitions.

### Manual Install from GitHub Releases

Download the latest release:

```bash
curl -LO https://github.com/ZeroiJ/omarchy-doctor/releases/download/v0.1.0/omadoctor-v0.1.0-x86_64.tar.gz
tar -xzf omadoctor-v0.1.0-x86_64.tar.gz
cd omadoctor-v0.1.0-x86_64
sudo cp omadoctor /usr/bin/
sudo mkdir -p /usr/share/omadoctor/fixes
sudo cp fixes/*.toml /usr/share/omadoctor/fixes/
sudo cp VERSION /usr/share/omarchy-doctor/
```

### Build from Source

```bash
git clone https://github.com/ZeroiJ/omarchy-doctor.git
cd omarchy-doctor
cargo build --release
sudo cp target/release/omadoctor /usr/bin/omadoctor
sudo mkdir -p /usr/share/omadoctor/fixes
sudo cp fixes/*.toml /usr/share/omadoctor/fixes/
```

## Usage

### Interactive TUI (default)

```bash
omadoctor
```

**Controls:**
- `↑/↓` — Navigate issues
- `Enter` — View issue details
- `d` — Run detection for current issue
- `f` — Apply fix (shows confirmation dialog)
- `g` — Search GitHub for related issues
- `Esc/q` — Go back / quit

**Fix Flow:**
1. Select an issue and press `Enter` to view details
2. Press `d` to run detection
3. If issue detected (red box), press `f`
4. Confirm with `y` in the dialog, or `n` to see manual commands
5. Watch the progress bar as the fix applies
6. See success (green) or failure (red) result

### Scan Mode (non-interactive)

```bash
omadoctor --scan
```

Output:
```
🔍 Scanning your system...

❌ 🎮 Steam: missing lib32-mesa
✅ 📹 Zoom: working fine
❌ 🔊 Audio: PipeWire service stopped  
✅ 🖥️ GPU: AMD driver OK

2 issues found. Run `omadoctor` for interactive fixes.
```

Exit codes:
- `0` — All systems operational
- `1` — One or more issues detected

### CLI Options

```bash
omadoctor --help       # Show help
omadoctor --version    # Show version
omadoctor --scan       # Non-interactive scan
omadoctor --skip-update # Skip update check on startup
```

## Auto-Update

On startup, omadoctor silently checks for fix database updates from GitHub Releases. If a newer version is available, it automatically downloads and installs new fix definitions to `~/.local/share/omadoctor/fixes/`.

- ✅ Updates are **silent** — no prompts or interruptions
- ✅ User fixes **override** system fixes — your customizations are preserved
- ✅ Works **offline** — skips update if no internet connection

## Contributing New Fixes

Fixes are defined in TOML files. Create a new file in `~/.local/share/omadoctor/fixes/`:

```toml
[[issue]]
id = "steam_lib32_missing"
category = "gaming"
name = "Steam won't launch - missing 32-bit libraries"
symptoms = ["steam won't launch", "libGL error", "missing 32-bit"]
detection = "command -v steam && ! pacman -Qi lib32-mesa"
fix = "sudo pacman -S lib32-mesa lib32-vulkan-radeon"
safe = true
```

**Fields:**
- `id` — Unique identifier (snake_case)
- `category` — Determines icon: `gaming`🎮, `video-conferencing`📹, `graphics`🖥️, `audio`🔊, `display`🖥️
- `name` — Display name
- `symptoms` — Keywords for GitHub search
- `detection` — Shell command (exit 0 = issue exists)
- `fix` — Shell command to apply fix
- `safe` — `true` if fix is safe to auto-run

### Detection Logic

The detection command:
- **Exit 0** → Issue is present (needs fixing)
- **Non-zero** → No issue (system is fine)

Example patterns:
```bash
# Check if package is missing
detection = "! pacman -Qi lib32-mesa"

# Check if command exists and something else
detection = "command -v steam && ! pacman -Qi lib32-mesa"

# Check systemd service
detection = "! systemctl is-active --quiet pipewire"
```

## Directory Structure

```
~/.local/share/omadoctor/fixes/  # User-local fixes (highest priority)
/usr/share/omadoctor/fixes/      # System-wide fixes (package manager)
./fixes/                         # Development fallback
```

Searched in that order — user definitions override system ones.

## Building from Source

Requirements:
- Rust 1.70+
- `cargo` build tool

```bash
cargo build --release
./target/release/omadoctor --version
```

Create release tarball:
```bash
./create-release.sh 0.1.0
```

## License

MIT License — see [LICENSE](LICENSE)

Copyright (c) 2026 ZeroiJ

## Related

- [Omarchy Linux](https://omarchy.org) — The operating system
- [Omarchy Discord](https://omarchy.org/discord) — Community support
- [AUR Package](https://aur.archlinux.org/packages/omadoctor-bin) — Arch Linux package

---

<p align="center">Made with ❤️ for the Omarchy community</p>
