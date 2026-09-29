import { execSync } from "child_process";
import fs from "fs";

function runGit(cmd: string): string {
  try {
    return execSync(cmd, { encoding: "utf-8", stdio: ["pipe", "pipe", "ignore"] }).trim();
  } catch {
    return "";
  }
}

function ensureFullHistory() {
  const isShallow = runGit("git rev-parse --is-shallow-repository");
  if (isShallow === "true") {
    try {
      execSync("git fetch --unshallow --tags", { stdio: "ignore" });
    } catch {
      try {
        execSync("git fetch --tags", { stdio: "ignore" });
      } catch {}
    }
  }
}

function getRepo(): string {
  if (process.env.GITHUB_REPOSITORY) {
    return process.env.GITHUB_REPOSITORY.trim();
  }
  const remoteUrl = runGit("git remote get-url origin");
  const match = remoteUrl.match(/github\.com[/:]([^/]+)\/([^/.]+)(?:\.git)?/);
  if (match) {
    return `${match[1]}/${match[2]}`;
  }
  return "m66kkm/hoardster";
}

function getPackageVersion(): string {
  try {
    const pkg = JSON.parse(fs.readFileSync("package.json", "utf-8"));
    return pkg.version ? `v${pkg.version}` : "latest";
  } catch {
    return "latest";
  }
}

export function generateChangelog(argTarget?: string): string {
  ensureFullHistory();

  const repo = getRepo();
  const pkgVersion = getPackageVersion();

  let baseTag = "";
  let targetRef = "";
  let displayTarget = "";

  const arg = argTarget;

  if (arg && arg.includes("..")) {
    const [b, t] = arg.split("..");
    baseTag = b.trim();
    targetRef = t.trim();
    displayTarget = targetRef;
  } else if (arg) {
    targetRef = arg.trim();
    displayTarget = targetRef;
    baseTag = runGit(`git describe --tags --abbrev=0 "${targetRef}^"`);
  } else {
    const ciTag = process.env.GITHUB_REF_NAME;
    if (ciTag && process.env.GITHUB_REF_TYPE === "tag") {
      targetRef = ciTag.trim();
      displayTarget = targetRef;
      baseTag = runGit(`git describe --tags --abbrev=0 "${targetRef}^"`);
    } else {
      const headTags = runGit("git tag --points-at HEAD").split("\n").map(t => t.trim()).filter(Boolean);
      if (headTags.length > 0) {
        targetRef = headTags[0];
        displayTarget = targetRef;
        baseTag = runGit(`git describe --tags --abbrev=0 "${targetRef}^"`) || runGit("git describe --tags --abbrev=0 HEAD^");
      } else {
        targetRef = "HEAD";
        displayTarget = pkgVersion;
        baseTag = runGit("git describe --tags --abbrev=0 HEAD");
      }
    }
  }

  const logRange = baseTag ? `${baseTag}..${targetRef}` : targetRef;
  const rawLog = runGit(`git log ${logRange} --pretty=format:"%h%x09%s"`);

  if (!rawLog) {
    return `No changes found${baseTag ? ` between ${baseTag} and ${displayTarget}` : ""}.`;
  }

  let lines = rawLog.split("\n").map(l => l.trim()).filter(Boolean);

  // Filter out self-release commits (e.g. chore: release version 1.1.2) unless that's all there is
  const filteredLines = lines.filter(line => {
    const [, ...rest] = line.split("\t");
    const subject = rest.join("\t").trim();
    return !/^(chore|build)(\([^)]+\))?:\s*(release|bump)\s+version/i.test(subject);
  });

  if (filteredLines.length > 0) {
    lines = filteredLines;
  }

  const categories: Record<string, { title: string; items: string[] }> = {
    feat: { title: "🚀 **新增功能 (Features)**", items: [] },
    fix: { title: "🐛 **问题修复 (Bug Fixes)**", items: [] },
    perf: { title: "⚡ **性能与体验优化 (Performance & Improvements)**", items: [] },
    refactor: { title: "🔨 **代码重构 (Refactoring)**", items: [] },
    docs: { title: "📝 **文档更新 (Documentation)**", items: [] },
    chore: { title: "🧹 **工程与维护 (Chores & Maintenance)**", items: [] },
  };

  const otherItems: string[] = [];

  for (const line of lines) {
    const [hash, ...rest] = line.split("\t");
    const subject = rest.join("\t").trim();

    const commitLink = `([\`${hash}\`](https://github.com/${repo}/commit/${hash}))`;

    const match = subject.match(/^([a-z]+)(\(([^)]+)\))?:\s*(.+)$/i);
    if (match) {
      const type = match[1].toLowerCase();
      const scope = match[3];
      const desc = match[4];

      const formatted = scope 
        ? `- **${scope}**: ${desc} ${commitLink}`
        : `- ${desc} ${commitLink}`;

      if (categories[type]) {
        categories[type].items.push(formatted);
      } else if (type === "deps" || type === "ci" || type === "build") {
        categories.chore.items.push(formatted);
      } else if (type === "improve") {
        categories.perf.items.push(formatted);
      } else {
        otherItems.push(`- ${subject} ${commitLink}`);
      }
    } else {
      otherItems.push(`- ${subject} ${commitLink}`);
    }
  }

  const featCount = categories.feat.items.length;
  const fixCount = categories.fix.items.length;
  const perfCount = categories.perf.items.length;
  const refactorCount = categories.refactor.items.length;

  let summaryParts: string[] = [];
  if (featCount > 0) summaryParts.push(`${featCount} 项新特性`);
  if (fixCount > 0) summaryParts.push(`${fixCount} 项问题修复`);
  if (perfCount > 0) summaryParts.push(`${perfCount} 项体验优化`);
  if (refactorCount > 0) summaryParts.push(`${refactorCount} 项代码重构`);
  if (summaryParts.length === 0) summaryParts.push("系统维护与优化");

  const summaryText = `本次更新主要包含 ${summaryParts.join("、")}，进一步提升应用稳定性与操作体验。`;

  let changelog = `## 🎮 Hoardster ${displayTarget}\n\n`;
  changelog += `> [!NOTE]\n`;
  changelog += `> **版本概要**：${summaryText}\n\n`;
  changelog += `---\n\n`;

  for (const key of Object.keys(categories)) {
    const cat = categories[key];
    if (cat.items.length > 0) {
      changelog += `### ${cat.title}\n\n`;
      changelog += `${cat.items.join("\n")}\n\n`;
      changelog += `---\n\n`;
    }
  }

  if (otherItems.length > 0) {
    changelog += `### 📦 **其他变更 (Other Changes)**\n\n`;
    changelog += `${otherItems.join("\n")}\n\n`;
    changelog += `---\n\n`;
  }

  const cleanVer = displayTarget.replace(/^v/, "");
  changelog += `### 📦 **安装包与产物说明 (Downloads)**\n\n`;
  changelog += `| 产物文件名 | 适配平台 | 说明 |\n`;
  changelog += `| :--- | :--- | :--- |\n`;
  changelog += `| \`Hoardster_${cleanVer}_x64-setup.exe\` | Windows 64-bit | Windows 桌面版标准安装包 (推荐直接下载安装) |\n\n`;
  changelog += `---\n\n`;

  if (baseTag) {
    changelog += `🔍 **完整更新比对 (Full Changelog)**: [\`${baseTag}...${displayTarget}\`](https://github.com/${repo}/compare/${baseTag}...${displayTarget})\n`;
  }

  return changelog.trim();
}

// Parse CLI args
let outFile = "";
const args = process.argv.slice(2);
let targetArg = "";

for (let i = 0; i < args.length; i++) {
  if (args[i] === "--out" && i + 1 < args.length) {
    outFile = args[i + 1];
    i++;
  } else if (!targetArg && !args[i].startsWith("-")) {
    targetArg = args[i];
  }
}

const changelog = generateChangelog(targetArg);

if (outFile) {
  fs.writeFileSync(outFile, changelog, "utf-8");
  console.log(`Changelog written to ${outFile}`);
} else {
  console.log(changelog);
}

// If running in GitHub Actions, write output to GITHUB_OUTPUT
if (process.env.GITHUB_OUTPUT) {
  const delimiter = `EOF_${Date.now()}`;
  fs.appendFileSync(process.env.GITHUB_OUTPUT, `body<<${delimiter}\n${changelog}\n${delimiter}\n`);
}
