# Docker Manager

**The clean desktop cockpit for Docker.**

Docker Manager is a fast, native Linux desktop app for developers who want Docker control without terminal noise.  
Built with Rust + GTK4, it gives you a focused UI to inspect containers, run actions confidently, and keep full command visibility.

## Why Docker Manager

- Run `Start`, `Stop`, `Remove`, and `Refresh` in one click.
- View live container inventory with name, ID, status, and image.
- Stay responsive under load with non-blocking background execution.
- Track every operation in a clear, in-app activity log.
- Prevent mistakes with a destructive-action safety toggle.
- Keep your workflow local, lightweight, and keyboard-friendly.

## Product Highlights

- Native desktop performance.
- Practical operational safeguards.
- Immediate feedback on success and failure.
- Automatic list refresh after state-changing actions.
- Minimal, focused interface designed for daily use.

## Quick Start

### 1. Prerequisites

- Linux desktop environment
- Docker CLI on your `PATH`
- Permission to run Docker commands
- Rust (stable)

### 2. Run

```bash
cargo run
```

### Download Binary Release (Linux x86_64)

Download `docker-manager-linux-x86_64.tar.gz` from Releases and extract:

```bash
tar -xzf docker-manager-linux-x86_64.tar.gz
./docker-manager-linux-x86_64
```

The extracted binary keeps executable permissions.

### GNOME Launcher Icon (Overview/Dock)

To get a proper launcher icon in GNOME overview/dock, install the desktop entry and icon:

```bash
chmod +x scripts/install-local.sh
./scripts/install-local.sh
```

Then launch from the app grid as **Docker Manager**.

### 3. Build Release

```bash
cargo build --release
```

### Build AppImage (Recommended for end users)

```bash
chmod +x scripts/build-appimage.sh
./scripts/build-appimage.sh
```

Output:

- `dist/docker-manager-linux-x86_64.AppImage`
- `dist/docker-manager-linux-x86_64.AppImage.sha256`

### 4. Run Tests

```bash
cargo test
```

## Screenshot

![Docker Manager Main UI](assets/screenshot-main.png)

The current release includes one primary screenshot. More workflow screenshots can be added in future updates.

## Architecture

- `src/main.rs`: app bootstrap
- `src/ui.rs`: interface + action orchestration
- `src/docker.rs`: Docker command client + output parsing
- `src/model.rs`: domain models

## Positioning

Docker Manager is ideal for:

- Developers who use Docker every day
- Teams that want safer container operations
- Engineers who prefer native tools over heavy dashboards

## Roadmap

- Container logs viewer
- Restart action
- Search and filters
- Package distribution (`.deb` / Flatpak)

## License

MIT. See [LICENSE](LICENSE).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).
