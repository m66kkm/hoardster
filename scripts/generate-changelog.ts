import { execSync } from "child_process";
import fs from "fs";

function getChangelog() {
  let latestTag = "";
  try {
    latestTag = execSync("git describe --tags --abbrev=0", { encoding: "utf-8" }).trim();
  } catch {
    latestTag = "";
  }

  const repo = "m66kkm/hoardster";
  const logRange = latestTag ? `${latestTag}..HEAD` : "HEAD";
  const rawLog = execSync(`git log ${logRange} --pretty=format:"%h%x09%s"`, { encoding: "utf-8" }).trim();

  if (!rawLog) {
    return "No changes found.";
  }

  const lines = rawLog.split("\n").filter(Boolean);

  const categories: Record<string, { title: string; items: string[] }> = {
    feat: { title: "🚀 **新增功能 (Features)**", items: [] },
    fix: { title: "🐛 **问题修复 (Bug Fixes)**", items: [] },
    perf: { title: "⚡ **性能与体验优化 (Performance & Improvements)**", items: [] },
    refactor: { title: "🔨 **代码重构 (Refactoring)**", items: [] },
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
      } else {
        otherItems.push(`- ${subject} ${commitLink}`);
      }
    } else {
      otherItems.push(`- ${subject} ${commitLink}`);
    }
  }

  let changelog = "";
  for (const key of Object.keys(categories)) {
    const cat = categories[key];
    if (cat.items.length > 0) {
      changelog += `### ${cat.title}\n\n${cat.items.join("\n")}\n\n`;
    }
  }

  if (otherItems.length > 0) {
    changelog += `### 📦 **其他变更 (Other Changes)**\n\n${otherItems.join("\n")}\n\n`;
  }

  let pkgVersion = "latest";
  try {
    const pkg = JSON.parse(fs.readFileSync("package.json", "utf-8"));
    pkgVersion = `v${pkg.version}`;
  } catch {}

  if (latestTag) {
    changelog += `**Full Changelog**: https://github.com/${repo}/compare/${latestTag}...${pkgVersion}\n`;
  }

  return changelog.trim();
}

const changelog = getChangelog();
console.log(changelog);

// If running in GitHub Actions, write output to GITHUB_OUTPUT
if (process.env.GITHUB_OUTPUT) {
  const delimiter = `EOF_${Date.now()}`;
  fs.appendFileSync(process.env.GITHUB_OUTPUT, `body<<${delimiter}\n${changelog}\n${delimiter}\n`);
}
