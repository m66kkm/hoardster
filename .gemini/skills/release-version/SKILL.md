---
name: release-version
description: 自动发布一个新版本（更新版本号、打标签、推送并触发流水线）
---

# Release New Version Skill

当用户要求“发布一个新版本”（Release a new version）时，按照以下步骤进行自动化操作：

## 1. 检查当前版本号
- 使用 `view_file` 或相关工具查看项目根目录下的 `package.json` 和 `src-tauri/tauri.conf.json`，获取当前版本号（如 `1.0.5`）。

## 2. 确认或计算新版本号
- 默认情况下进行 patch 更新（即最后一位数字加 1，如 `1.0.5` -> `1.0.6`）。
- 也可以通过向用户确认是否需要 minor 或 major 更新。

## 3. 修改版本号
- 修改 `package.json` 中的 `"version"` 字段为新版本号。
- 修改 `src-tauri/tauri.conf.json` 中的 `"version"` 字段为新版本号。

## 4. 提交代码 (Commit)
- 使用 `git add package.json src-tauri/tauri.conf.json` 将改动暂存。
- 使用 `git commit -m "chore: release version <新版本号>"` 提交代码。

## 5. 打标签 (Tag)
- 使用 `git tag v<新版本号>` 为当前提交打上标签（例如 `git tag v1.0.6`）。

## 6. 推送到远程仓库并触发流水线
- 首先推送代码提交：`git push`
- **关键步骤**：由于部分环境代理（如 rtk）的限制，不要依赖 `--follow-tags`。务必使用原生 git 命令明确推送标签：`git push origin v<新版本号>`

## 7. 汇报结果
- 告知用户新版本已经打好 Tag 并推送到远端。
- 提醒用户可以前往 GitHub Actions 页面查看自动发布流水线的构建状态。
