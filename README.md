# 🔧 omadoctor

> Never ask on Discord again. Your system fixes itself.

A fast, terminal-based diagnostic and repair tool for [Omarchy Linux](https://omarchy.org). Detect issues automatically, apply fixes with one keystroke, or search GitHub for community solutions.

![Demo](demo.gif)

## Features

- 🎯 **Automatic Detection** — Run shell commands to check if issues exist
- 🔧 **One-Key Fixes** — Press `f` to apply verified fixes instantly
- 🌐 **GitHub Integration** — Search `basecamp/omarchy` issues for community solutions
- 📦 **TOML-Based** — Easy to extend with new fixes
- 🖥️ **Beautiful TUI** — Ratatui-powered interface with emoji category icons
- 🔍 **Scan Mode** — Non-interactive batch scanning for CI/scripts

## Installation

### From AUR (Arch Linux)

```bash
yay -S omadoctor
# or
paru -S omadoctor
```

### From crates.io

```bash
cargo install omadoctor
```

### Manual Install

```bash
git clone https://github.com/ZeroiJ/omarchy-doctor.git
cd omarchy-doctor
./install.sh
```

This installs:
- Binary to `/usr/bin/omadoctor`
- Fix definitions to `/usr/share/omadoctor/fixes/`

## Usage

### Interactive TUI (default)

```bash
omadoctor
```

**Controls:**
- `↑/↓` — Navigate issues
- `Enter` — View issue details
- `d` — Run detection for current issue
- `f` — Apply fix (only after detection finds issue)
- `g` — Search GitHub for related issues
- `Esc/q` — Go back / quit

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

### CLI Help

```bash
omadoctor --help
omadoctor --version
```

## Contributing New Fixes

Fixes are defined in TOML files. Create a new file in `/usr/share/omadoctor/fixes/` or `~/.local/share/omadoctor/fixes/`:

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
~/.local/share/omadoctor/fixes/  # User-local fixes
/usr/share/omadoctor/fixes/      # System-wide fixes (package manager)
./fixes/                         # Development fallback
```

Searched in that order — user definitions override system ones.

## Building from Source

```bash
cargo build --release
sudo cp target/release/omadoctor /usr/bin/
sudo mkdir -p /usr/share/omadoctor/fixes
sudo cp fixes/*.toml /usr/share/omadoctor/fixes/
```

## License

MIT License — see [LICENSE](LICENSE)

## Related

- [Omarchy Linux](https://omarchy.org) — The operating system
- [Omarchy Discord](https://omarchy.org/discord) — Community support

---

<p align="center">Made with ❤️ for the Omarchy community</p>
