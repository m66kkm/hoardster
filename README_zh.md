[English](README.md) | [简体中文](README_zh.md)

# Game Index (独立游戏本地数字资产管理库)

**Game Index** 是一款专为游戏收藏家打造的本地游戏资产管理工具。它能够扫描你硬盘中海量的独立游戏、ISO 镜像以及免安装整合包，自动清洗并匹配 Steam 官方接口，获取最原汁原味的官方译名、高清海报以及玩家评测，最终以一种极其现代化、高颜值的界面为你呈现。

## ✨ 核心特性 (Features)

- 🚀 **极速物理扫描**：毫秒级全盘扫描，自动过滤系统无用文件夹。
- 🔍 **智能去重与版本合并**：自动识别完全相同的重复文件以及同一游戏的多个版本/DLC 整合包，帮你揪出硬盘空间杀手。
- 🎮 **Steam 数据自动匹配**：精准请求 Steam API，自动获取正版中文译名、发售时间以及好评率，并且自动下载高清海报缓存到本地。
- 🧩 **系列游戏关联**：智能识别“刺客信条”、“黑暗之魂”等游戏系列，自动归类为一个 Franchise 卡片。
- 🎨 **顶级暗黑视觉美学**：精心设计的 Glassmorphism (毛玻璃) 风格，配以炫酷的霓虹赛博渐变光效，给你顶尖的数字展厅体验。
- ⚙️ **高性能架构**：基于 Rust + Tauri 的轻量级后端系统，SQLite 闪电入库。

## 📸 界面预览 (Screenshots)

### 系统首页 (数据中心与统计看板)
![Dashboard](assets/dashboard.png)

### 海报墙 (沉浸式游戏导览)
![Poster Wall](assets/poster_wall.png)

### 游戏系列与续作 (智能聚合归类)
![Franchises](assets/franchises.png)

## 🛠️ 技术栈 (Tech Stack)

- **前端 UI**: React 18, TypeScript, Vite
- **样式与设计**: Vanilla CSS (极简自定义的全局变量系统)
- **后端引擎**: Rust, Tauri v1
- **数据库**: SQLite (rusqlite)
- **运行环境**: Bun

## 📦 快速开始 (Getting Started)

### 环境依赖
- 安装 [Rust](https://www.rust-lang.org/tools/install) 和 Cargo
- 安装 [Bun](https://bun.sh/) 
- (仅 Windows 需要) 安装 C++ 生成工具 / Visual Studio

### 启动项目

```bash
# 1. 克隆代码
git clone https://github.com/m66kkm/game-index.git
cd game-index

# 2. 安装前端依赖
bun install

# 3. 启动开发服务器 (前后端热更新)
bun run tauri:dev
```

## 🏗️ 编译与打包 (Build & Deploy)

本项目打包通过 Tauri 完成，支持跨平台。内置了便捷的 `deploy` 脚本：

```bash
# 执行此命令将会进行 Release 构建，并将打包好的安装程序自动拷贝到 target 目录下
bun run deploy
```
*注：编译完成后，你可以在根目录的 `target` 文件夹内找到 Windows NSIS 安装包 (`.exe`)*

## 📄 许可协议 (License)

本项目采用 **AGPL-3.0 License** 开源协议 - 请参阅 [LICENSE](LICENSE) 文件了解详细信息。这意味着如果你修改或将其作为网络服务提供给其他人使用，你必须同样开源你的代码。
