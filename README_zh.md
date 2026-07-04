[English](README.md) | [简体中文](README_zh.md)

# Hoardster

<p align="center">
  <img src="assets/mascot.jpg" width="400" alt="Hoardster Mascot" />
</p>
<p align="center">
  <em>Hoardster — Welcome to Hammy's Game Den.</em>
</p>

**Hoardster** 不仅仅是一个本地游戏管理工具，它更是**“仓鼠型玩家”**（热衷于疯狂下载和囤积数字游戏、光盘镜像、免安装版的数字资产收藏家）专属的终极兵器与私人阵地！

还在看着硬盘里杂乱无章的 "fitgirl-repack", "XYZ破解版", "某某游戏免安装中文版" 文件夹发愁吗？Hoardster 能以毫秒级速度扫描你的整个硬盘，自动清洗这些杂乱的命名，智能匹配 Steam 官方接口，并为你自动挂载正版中文译名、发售时间、玩家好评率，甚至在本地离线缓存高清海报。

它成功地将海量游戏数据的存储与管理变成了一种具象化且沉浸式的体验。玩家打开软件，看到的不再是冷冰冰的本地目录和文件管理工具，而是一个充满赛博科幻感、让人充满掌控权（Empowerment）的私人专属展厅。

## ✨ 核心功能

- 🚀 **极速物理盘库**：毫秒级全盘游戏特征扫描，自动过滤无用的系统与缓存文件夹。
- 🔍 **智能去重与版本合并**：自动识别完全相同的重复文件以及同一游戏的不同版本/DLC整合包，帮你揪出硬盘空间杀手。
- 🎮 **Steam 数据全自动匹配**：精准请求 Steam API，自动拉取官方中文译名、发售日、玩家评分，并将高清海报缓存至本地。
- 🧩 **系列游戏自动关联**：智能识别“刺客信条”、“黑暗之魂”等游戏系列，并自动将其聚合为 Franchise 卡片。
- 📡 **1337x 独立情报抓取**：内置情报雷达模块，一键支持后台并发抓取最新发布、最多下载 (Leechers) 和最多做种 (Seeders) 游戏资源数据。
- 🎨 **高级暗黑视觉美学**：精心设计的 Glassmorphism (毛玻璃) 风格搭配炫酷的霓虹赛博渐变光效，为你提供顶级的数字展厅体验。
- ⚙️ **极致性能架构**：底层基于 Rust + Tauri 的轻量级引擎构建，搭配闪电般的 SQLite 本地存储。

## 📸 界面预览

### 系统数据总览 (存储与统计视图)
![Dashboard](assets/dashboard.png)

### 沉浸式海报墙 (游戏导航视图)
![Poster Wall](assets/poster_wall.png)

### 游戏系列聚合 (智能分组)
![Franchises](assets/franchises.png)

## 🛠️ 技术栈

- **前端 UI**: React 18, TypeScript, Vite, Zustand
- **样式与设计**: 原生 CSS (极简自定义全局变量系统)
- **后端引擎**: Rust, Tauri v2
- **数据库**: SQLite (rusqlite)
- **运行环境**: Bun

## 📦 快速开始

### 环境依赖
- 安装 [Rust](https://www.rust-lang.org/tools/install) 与 Cargo
- 安装 [Bun](https://bun.sh/) 
- (仅限 Windows) 安装 C++ Build Tools 或 Visual Studio

### 运行项目

```bash
# 1. 克隆代码仓库
git clone https://github.com/m66kkm/hoardster.git
cd hoardster

# 2. 安装前端依赖
bun install

# 3. 启动开发服务器 (支持前端热更新与后端热重载)
bun run tauri:dev
```

## 🏗️ 编译与打包发布

本项目通过 Tauri 进行打包，支持跨平台构建。系统内置了便捷的 `deploy` 脚本：

```bash
# 执行此命令将会进行 Release 级别编译，并自动把安装包拷贝至目标文件夹
bun run deploy
```
*注：编译完成后，你可以在根目录的 `target` 文件夹中找到 Windows NSIS 安装包 (`.exe`)。*

## 📄 开源协议

本项目采用 **AGPL-3.0 License** - 详情请参阅 [LICENSE](LICENSE) 文件。这意味着如果你修改了该项目，或将其作为网络服务提供给其他人使用，你也必须开源你的代码。
