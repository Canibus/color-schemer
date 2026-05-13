# Color Schemer

**Color Schemer** is a professional-grade Windows utility designed for NVIDIA GPU users who need precise control over their display color profiles. It allows for quick switching between custom-tuned profiles for gaming, movies, or productivity.

![Color Schemer GUI](public/icon.svg) <!-- Placeholder for actual screenshot -->

## 🌟 Key Features

- **NVIDIA Integration:** Direct control over Brightness, Contrast, Gamma, and NVIDIA's **Digital Vibrance** via NVAPI.
- **Auto-Switching:** Automatically applies specific profiles when your favorite games or applications are in the foreground.
- **Global Hotkeys:** Switch profiles instantly using customizable keyboard shortcuts (e.g., `Ctrl+Shift+F5`).
- **Modern HUD Interface:** A sleek, "NVIDIA Neon" inspired GUI built with Svelte 5 and Tauri.
- **System Tray Support:** Runs quietly in the background; access all features via the tray icon.
- **Multi-Monitor Ready:** Apply settings to specific displays or all monitors at once.
- **Multilingual:** Full support for English and Russian.

## 🛠 Requirements

- **Operating System:** Windows 10/11 (64-bit).
- **Hardware:** NVIDIA GPU (required for Digital Vibrance and optimal performance).
- **Drivers:** NVIDIA Game Ready or Studio Drivers installed.

## 👀 Preview
<img width="260" height="340" alt="color-schemer_UJoIxaqZuX" src="https://github.com/user-attachments/assets/c02ac035-9b98-4d54-b7c6-5d5ac750a876" />
<img width="260" height="340" alt="color-schemer_dVlCEZnq0f" src="https://github.com/user-attachments/assets/3168d9bc-4cd2-4f6c-a2fc-55636f4a7279" />


## 🚀 Getting Started

1. **Download:** Grab the latest `.msi` or `.exe` from the [Releases](https://github.com/yourusername/color-schemer/releases) page.
2. **Install:** Run the installer and launch the app.
3. **Configure:** 
   - Open the GUI from the system tray.
   - Create or edit profiles in the **Profiles** tab.
   - Link applications to profiles for automatic switching.
   - Set your preferred shortcuts in the **Settings** tab.
4. **Enjoy:** Use your hotkeys to switch profiles while gaming or working!

## ⌨️ Default Hotkeys

- **Next Profile:** `Ctrl+Shift+F5`
- **Previous Profile:** `Ctrl+Shift+F6`
- **Reset to Default:** `Ctrl+Shift+F7`

## 🏗 Development

If you want to build Color Schemer from source, you'll need [Rust](https://rustup.rs/) and [Bun](https://bun.sh/) (or Node.js).

```bash
# Clone the repository
git clone https://github.com/yourusername/color-schemer.git
cd color-schemer

# Install frontend dependencies
bun install

# Run in development mode
bun run tauri dev

# Build for release
bun run tauri build
```

## 📜 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Credits

- Built with [Tauri](https://tauri.app/) and [Svelte 5](https://svelte.dev/).
- GPU control powered by NVIDIA NVAPI.
