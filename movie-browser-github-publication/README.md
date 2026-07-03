# Movie Browser

Movie Browser is a lightweight desktop app built with Tauri, Vite, and TypeScript for browsing and launching movies from a local or removable drive.

## Features

- Scans for movie files in a `Movies` folder on mounted removable drives
- Groups movies by folder for easy browsing
- Supports search by title, folder, and file path
- Launches movies in the system default player
- Works well on Fedora Linux

## Supported movie paths

The app looks for a `Movies` folder in these common mount locations:

- `/run/media/<user>/<drive>/Movies`
- `/media/<user>/<drive>/Movies`
- `/mnt/<drive>/Movies`

If you want to force a specific folder during development, use:

```bash
MOVIES_DIR="/run/media/$USER/YourDrive/Movies" ./movie-browser
```

## Fedora install

After building the RPM bundle, install or reinstall the packaged app with:

```bash
sudo dnf install ./src-tauri/target/release/bundle/rpm/Movie\ Browser-1.0.0-1.x86_64.rpm
```

If you already installed it once, use:

```bash
sudo dnf reinstall ./src-tauri/target/release/bundle/rpm/Movie\ Browser-1.0.0-1.x86_64.rpm
```

## Development setup

1. Clone the repository:

```bash
git clone https://github.com/<your-user>/movie-browser.git
cd movie-browser
```

2. Install dependencies:

```bash
npm install
```

3. Run the app in development mode:

```bash
npm run tauri dev
```

## Build for release

To build the frontend and create a Tauri release bundle:

```bash
npm run build
npm run tauri build
```

The RPM package will be generated here:

```bash
src-tauri/target/release/bundle/rpm/
```

## Repository structure

- `src/` — frontend source files
- `src-tauri/` — Tauri Rust backend and packaging config
- `index.html` — app shell
- `src/styles.css` — app styles
- `src/main.ts` — frontend logic
- `README.md` — project documentation
- `.gitignore` — files excluded from Git

## Publishing on GitHub

This repo belongs in the `Node`/`Node.js` GitHub language category because the app uses `npm`, `vite`, and frontend JavaScript/TypeScript tooling. The Rust backend is part of the Tauri bundle, but the primary repo setup and package management are Node-based.

## Notes

- Do not commit `node_modules/`, `dist/`, or generated build artifacts.
- Keep `.gitignore` in the project root to prevent machine-specific and temporary files from being tracked.
- The app is intended for desktop use and is especially tuned for Fedora Linux installations.
