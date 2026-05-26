# Mouse Cage Manager

A desktop mouse cage management app built with [Tauri](https://tauri.app/) and Vite.

The app is designed for local cage records, mouse group tracking, search/filtering, native file storage, automatic backups, and printable/exportable cage summaries.

## Features

- Manage cages, cage locations, mouse groups, sex, strain, count, age/week data, and individual IDs.
- Search and filter cage records.
- Drag to reorder cages and location groups.
- Export selected cages as CSV.
- Export selected cages as PNG images:
  - single long image
  - grouped images by location
  - A4-style paginated images
- Export and import JSON data when needed.
- Native Tauri file storage instead of browser-only local storage.
- Automatic daily backups and manual backups.
- Backup history with restore and delete actions.
- About dialog with version, data path, backup path, and packaging notes.
- Light/dark appearance follows the system theme.

## Tech Stack

- Tauri 2
- Rust
- Vite
- Plain HTML/CSS/JavaScript frontend

## Project Structure

```text
.
├── index.html                  # Main frontend
├── mouse manager.html          # Original HTML source kept for reference
├── package.json                # npm scripts and frontend tooling
├── src-tauri/
│   ├── src/lib.rs              # Tauri commands for storage, export, and backups
│   ├── tauri.conf.json         # Tauri app and bundle configuration
│   └── icons/                  # App icons
└── .github/workflows/
    └── windows-portable.yml    # GitHub Actions build for Windows portable EXE
```

## Requirements

- Node.js 20 or newer
- npm
- Rust stable toolchain
- Tauri system dependencies for your platform

For platform setup details, see the official Tauri prerequisites:

https://tauri.app/start/prerequisites/

## Development

Install dependencies:

```bash
npm install
```

Run the Tauri development app:

```bash
npm run tauri:dev
```

Run the frontend only:

```bash
npm run dev
```

Build the frontend:

```bash
npm run build
```

## Packaging

Build the default Tauri bundles:

```bash
npm run tauri:build
```

Build a portable Windows executable without an installer:

```bash
npm run tauri:build:portable
```

The Windows portable executable is generated at:

```text
src-tauri/target/release/mouse-cage-manager.exe
```

On macOS, the local `.dmg` bundle is generated under:

```text
src-tauri/target/release/bundle/dmg/
```

The current app is unsigned. On macOS, unsigned builds may show a Gatekeeper warning when opened on another machine.

## GitHub Actions

The repository includes a manual GitHub Actions workflow for building a Windows portable executable:

```text
.github/workflows/windows-portable.yml
```

Open the repository on GitHub, go to **Actions**, select **Build Windows Portable EXE**, then click **Run workflow**. After the workflow finishes, download the artifact named:

```text
mouse-cage-manager-windows-portable
```

## Data And Backups

The app stores data in Tauri's native application data directory as:

```text
data.json
```

Backups are stored in:

```text
backups/
```

The exact paths depend on the operating system and user account. Use the app's **About** dialog or **Settings > Backup History** to open the data and backup locations.

Backup behavior:

- The app automatically creates a daily backup before saving over existing data.
- Manual backups can be created from the backup history window.
- Restoring a backup first creates a `before-restore-...json` snapshot of the current data.
- Backup files are JSON files and can be inspected manually if needed.

## Files Not To Upload Manually

When uploading the project to GitHub manually, do not upload generated or dependency folders:

```text
node_modules/
dist/
src-tauri/target/
.DS_Store
```

These files are ignored by `.gitignore` and can be regenerated.

## License

MIT
