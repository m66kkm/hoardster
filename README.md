[English](README.md) | [简体中文](README_zh.md)

# Game Index (Local Digital Game Asset Library)

**Game Index** is a local game asset management tool tailored for game collectors. It scans your hard drives for massive numbers of indie games, ISO images, and pre-installed repacks, automatically cleans up and matches them with the official Steam API to obtain authentic official names, high-definition posters, and player reviews, presenting them in an extremely modern and highly aesthetic interface.

## ✨ Features

- 🚀 **Ultra-Fast Physical Scanning**: Millisecond-level full-disk scanning with automatic filtering of useless system folders.
- 🔍 **Smart Deduplication & Version Merging**: Automatically identifies exact duplicate files and multiple versions/DLC repacks of the same game, helping you find those hard drive space killers.
- 🎮 **Automated Steam Data Matching**: Accurately requests the Steam API to automatically fetch official localized names, release dates, and user ratings, as well as caching high-definition posters locally.
- 🧩 **Franchise Association**: Intelligently recognizes game series like "Assassin's Creed" or "Dark Souls" and automatically groups them into Franchise cards.
- 🎨 **Premium Dark Visual Aesthetics**: Carefully designed Glassmorphism style paired with cool neon cyber gradient lighting effects, providing you with a top-tier digital showroom experience.
- ⚙️ **High-Performance Architecture**: Built on a lightweight backend using Rust + Tauri, with lightning-fast SQLite storage.

## 📸 Screenshots

### System Dashboard (Storage & Statistical Overview)
![Dashboard](assets/dashboard.png)

### Poster Wall (Immersive Game Navigation)
![Poster Wall](assets/poster_wall.png)

### Franchises & Sequels (Smart Grouping)
![Franchises](assets/franchises.png)

## 🛠️ Tech Stack

- **Frontend UI**: React 18, TypeScript, Vite
- **Styling & Design**: Vanilla CSS (Minimalist custom global variable system)
- **Backend Engine**: Rust, Tauri v1
- **Database**: SQLite (rusqlite)
- **Runtime**: Bun

## 📦 Getting Started

### Prerequisites
- Install [Rust](https://www.rust-lang.org/tools/install) and Cargo
- Install [Bun](https://bun.sh/) 
- (Windows only) Install C++ Build Tools / Visual Studio

### Running the Project

```bash
# 1. Clone the repository
git clone https://github.com/m66kkm/game-index.git
cd game-index

# 2. Install frontend dependencies
bun install

# 3. Start the dev server (with hot-reload for frontend and backend)
bun run tauri:dev
```

## 🏗️ Build & Deploy

The project is packaged via Tauri and supports cross-platform builds. A convenient `deploy` script is built-in:

```bash
# Running this command will perform a Release build and automatically copy the installer to the target directory
bun run deploy
```
*Note: After the build is complete, you can find the Windows NSIS installer (`.exe`) in the `target` folder at the root directory.*

## 📄 License

This project is licensed under the **AGPL-3.0 License** - see the [LICENSE](LICENSE) file for details. This means if you modify it or provide it as a network service to others, you must also open source your code.
