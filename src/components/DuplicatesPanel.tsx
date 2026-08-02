import { useTranslation } from "react-i18next";
import { ExternalLink, Layers, AlertTriangle } from "lucide-react";
import { getTypeBadgeClass } from "../utils/helpers";
import type { DuplicateGroup, Game } from "../types";

interface DuplicatesPanelProps {
  exactDuplicates: DuplicateGroup[];
  versionDuplicates: DuplicateGroup[];
  copyPath: (path: string, gameName: string) => void;
  openGameFolder: (path: string) => void;
}

export default function DuplicatesPanel({ exactDuplicates, versionDuplicates, copyPath, openGameFolder }: DuplicatesPanelProps) {
  const { t } = useTranslation();

  
  const renderGroup = (group: DuplicateGroup, type: "exact" | "version") => (
    <div key={group.name} className="conflict-group" style={type === "version" ? { borderLeft: "4px solid var(--warning-color)" } : {}}>
      <div className="conflict-title">
        <span>{group.name}</span>
        <span className={`badge ${type === "exact" ? "badge-dup" : "badge-ver"}`}>
          {type === "exact" ? t("dupExactCount", { count: group.games.length }) : t("dupVersionCount", { count: group.games.length })}
        </span>
      </div>
      <div className="conflict-paths">
        {group.games.map((game: Game) => (
          <div key={game.full_path} className={`conflict-path-item ${game.type.toLowerCase()}`}>
            <div style={{ display: "flex", alignItems: "center", gap: "0.75rem", flex: 1, minWidth: "280px" }}>
              <span className={`badge ${getTypeBadgeClass(game.type)}`}>{game.type}</span>
              <div style={{ display: "flex", flexDirection: "column" }}>
                {type === "version" && <span style={{ fontWeight: 600, color: "var(--text-primary)" }}>{game.original_name}</span>}
                <span className="code-path" onClick={() => copyPath(game.full_path, game.original_name)} title="点击复制路径">{game.full_path}</span>
              </div>
            </div>
            <div style={{ display: "flex", alignItems: "center", gap: "1.5rem", marginLeft: "auto" }}>
              <div style={{ textAlign: "right", fontSize: "0.8rem", color: "var(--text-secondary)" }}>
                <div>{t("dupSize")} <strong style={{ color: "var(--text-primary)" }}>{game.size}</strong></div>
                <div style={{ fontSize: "0.725rem", marginTop: "0.1rem", opacity: 0.8 }}>{t("dupCreated")} {game.created}</div>
              </div>
              <div style={{ display: "flex", gap: "0.5rem" }}>
                <button className="view-btn" onClick={() => openGameFolder(game.full_path)} title={t("openInExplorer")} style={{ padding: "0.4rem" }}>
                  <ExternalLink size={14} />
                </button>
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );

  return (
    <div className="panel" style={{ display: "block" }}>
      <div className="panel-header">
        <h2>{t("dupTitleNew")}</h2>
        <p style={{ color: "var(--text-secondary)", fontSize: "0.9rem", marginTop: "0.25rem" }}>
          {t("dupSubtitleNew")}
        </p>
      </div>

      <div style={{ marginBottom: "3rem" }}>
        <h3 style={{ marginBottom: "1rem", color: "var(--primary-accent)", display: "flex", alignItems: "center", gap: "0.5rem" }}>
          <Layers size={18} /> {t("dupExactSection")} ({exactDuplicates.length})
        </h3>
        <div className="conflicts-list">
          {exactDuplicates.map((group) => renderGroup(group, "exact"))}
          {exactDuplicates.length === 0 && (
            <div style={{ textAlign: "center", padding: "2rem", color: "var(--text-secondary)", border: "1px dashed var(--panel-border)", borderRadius: "12px" }}>
              {t("dupExactNoData")}
            </div>
          )}
        </div>
      </div>

      <div>
        <h3 style={{ marginBottom: "1rem", color: "var(--warning-color)", display: "flex", alignItems: "center", gap: "0.5rem" }}>
          <AlertTriangle size={18} /> {t("dupVersionSection")} ({versionDuplicates.length})
        </h3>
        <div className="conflicts-list">
          {versionDuplicates.map((group) => renderGroup(group, "version"))}
          {versionDuplicates.length === 0 && (
            <div style={{ textAlign: "center", padding: "2rem", color: "var(--text-secondary)", border: "1px dashed var(--panel-border)", borderRadius: "12px" }}>
              {t("dupVersionNoData")}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
