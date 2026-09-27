import { useState } from "react";
import { useTranslation } from "react-i18next";
import { 
  FolderOpen, 
  Trash2, 
  Copy, 
  HardDrive, 
  Calendar, 
  Layers, 
  FolderGit2,
  CheckCircle2
} from "lucide-react";
import { useGameContextMenu } from "../hooks/useGameContextMenu";
import GameCard from "./GameCard";
import type { DuplicateGroup, Game } from "../types";

interface DuplicatesPanelProps {
  exactDuplicates: DuplicateGroup[];
  versionDuplicates: DuplicateGroup[];
  copyPath: (path: string, gameName: string) => void;
  openGameFolder: (path: string) => void;
  onDeleteGame?: (game: Game) => void;
  onRefresh?: () => void;
}

interface DuplicateGroupItemProps {
  group: DuplicateGroup;
  type: "exact" | "version";
  copyPath: (path: string, gameName: string) => void;
  openGameFolder: (path: string) => void;
  onDeleteGame?: (game: Game) => void;
  onContextMenu: (e: React.MouseEvent, game: Game) => void;
}

function DuplicateGroupItem({
  group,
  type,
  copyPath,
  openGameFolder,
  onDeleteGame,
  onContextMenu,
}: DuplicateGroupItemProps) {
  const { t } = useTranslation();
  // 默认使用代表游戏或首个游戏，支持鼠标移到某一行记录时动态预览对应条目
  const repGame = group.games.find((g) => g.is_representative) || group.games[0];
  const [activeGame, setActiveGame] = useState<Game>(repGame);

  return (
    <div
      className={`conflict-group ${type === "version" ? "is-version" : "is-exact"}`}
    >
      <div className="conflict-group-body">
        {/* 左侧：游戏展示卡片 */}
        <div className="conflict-card-col">
          <GameCard
            game={activeGame || repGame}
            onOpenFolder={openGameFolder}
            onContextMenu={onContextMenu}
            hideTypeTag={true}
            hideLocation={true}
          />
        </div>

        {/* 右侧：当前重复的记录清单 */}
        <div className="conflict-paths-col">
          {group.games.map((game: Game) => {
            const isHovered = activeGame?.full_path === game.full_path;
            const isZeroSize = game.size === "0 B" || game.size === "0B" || game.size_bytes === 0;

            return (
              <div
                key={game.full_path}
                className={`conflict-path-item ${isHovered ? "active" : ""}`}
                onMouseEnter={() => setActiveGame(game)}
                onContextMenu={(e) => onContextMenu(e, game)}
                title={`${game.original_name}\n路径: ${game.full_path}\n大小: ${game.size}\n\n(右键弹出操作菜单)`}
              >
                {/* 1. 顶栏：标题与操作 */}
                <div className="conflict-path-header">
                  <div className="conflict-path-title-group">
                    <span className="conflict-path-name" title={game.original_name}>
                      {type === "version" ? game.original_name : (game.name || game.original_name)}
                    </span>
                    {game.is_representative && (
                      <span className="conflict-rep-tag" title="当前主库索引代表版本">
                        代表版本
                      </span>
                    )}
                  </div>

                  <div className="conflict-path-actions">
                    <button
                      className="conflict-action-btn"
                      onClick={() => openGameFolder(game.full_path)}
                      title={t("openInExplorer") || "在资源管理器中打开"}
                    >
                      <FolderOpen size={14} />
                    </button>
                    {onDeleteGame && (
                      <button
                        className="conflict-action-btn danger"
                        onClick={(e) => {
                          e.stopPropagation();
                          onDeleteGame(game);
                        }}
                        title={t("deleteCurrentGame") || "删除当前游戏本体"}
                      >
                        <Trash2 size={14} />
                      </button>
                    )}
                  </div>
                </div>

                {/* 2. 中栏：完整路径（点击复制） */}
                <div
                  className="conflict-path-address"
                  onClick={() => copyPath(game.full_path, game.original_name)}
                  title="点击复制完整路径"
                >
                  <span className="drive-indicator">{game.source_path ? game.source_path.substring(0, 2) : "D:"}</span>
                  <span className="path-text">{game.full_path}</span>
                  <Copy size={11} className="copy-icon" />
                </div>

                {/* 3. 底栏：规格数据（占用空间、创建时间） */}
                <div className="conflict-path-footer">
                  <div className={`meta-spec size ${isZeroSize ? "is-zero" : ""}`}>
                    <HardDrive size={11} />
                    <span>{t("dupSize") || "大小:"}</span>
                    <strong className="spec-val">{game.size}</strong>
                  </div>
                  <div className="meta-spec date">
                    <Calendar size={11} />
                    <span>{game.created ? game.created.split(" ")[0] : "未知"}</span>
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}

export default function DuplicatesPanel({
  exactDuplicates,
  versionDuplicates,
  copyPath,
  openGameFolder,
  onDeleteGame,
  onRefresh,
}: DuplicatesPanelProps) {
  const { t } = useTranslation();

  const { handleContextMenu, contextMenuElement } = useGameContextMenu({
    onDeleteGame,
    onRefresh,
  });

  const hasExact = exactDuplicates.length > 0;
  const hasVersion = versionDuplicates.length > 0;

  return (
    <div className="panel" style={{ display: "block" }}>
      <div className="panel-header" style={{ marginBottom: "2rem" }}>
        <h2 style={{ fontSize: "1.5rem", fontWeight: 700, letterSpacing: "-0.01em" }}>{t("dupTitleNew")}</h2>
        <p style={{ color: "var(--text-secondary)", fontSize: "0.9rem", marginTop: "0.35rem" }}>
          {t("dupSubtitleNew")}
        </p>
      </div>

      {hasExact && (
        <div style={{ marginBottom: hasVersion ? "3rem" : 0 }}>
          <div className="conflict-section-header">
            <div className="conflict-section-title">
              <Layers size={19} style={{ color: "#f43f5e" }} />
              <span>{t("dupExactSection")}</span>
              <span className="conflict-section-count">{exactDuplicates.length}</span>
            </div>
            <span className="conflict-section-tip">文件内容完全相同的重复定本，可选择清理多余副本释放磁盘空间</span>
          </div>

          <div className="conflicts-list">
            {exactDuplicates.map((group) => (
              <DuplicateGroupItem
                key={group.name}
                group={group}
                type="exact"
                copyPath={copyPath}
                openGameFolder={openGameFolder}
                onDeleteGame={onDeleteGame}
                onContextMenu={handleContextMenu}
              />
            ))}
          </div>
        </div>
      )}

      {hasVersion && (
        <div>
          <div className="conflict-section-header">
            <div className="conflict-section-title">
              <FolderGit2 size={19} style={{ color: "#f59e0b" }} />
              <span>{t("dupVersionSection")}</span>
              <span className="conflict-section-count">{versionDuplicates.length}</span>
            </div>
            <span className="conflict-section-tip">同一款游戏的不同版本（如早期发售版、年度版或备份），建议保留最佳版本</span>
          </div>

          <div className="conflicts-list">
            {versionDuplicates.map((group) => (
              <DuplicateGroupItem
                key={group.name}
                group={group}
                type="version"
                copyPath={copyPath}
                openGameFolder={openGameFolder}
                onDeleteGame={onDeleteGame}
                onContextMenu={handleContextMenu}
              />
            ))}
          </div>
        </div>
      )}

      {!hasExact && !hasVersion && (
        <div 
          className="conflict-empty-state" 
          style={{ 
            padding: "4.5rem 2rem", 
            display: "flex", 
            flexDirection: "column", 
            alignItems: "center", 
            justifyContent: "center" 
          }}
        >
          <CheckCircle2 size={46} style={{ color: "#10b981", marginBottom: "1rem", opacity: 0.9 }} />
          <h3 style={{ margin: "0 0 0.5rem 0", color: "#f1f5f9", fontSize: "1.15rem", fontWeight: 600 }}>
            {t("dupNoneFoundTitle")}
          </h3>
          <p style={{ margin: 0, color: "var(--text-secondary)", fontSize: "0.875rem" }}>
            {t("dupNoneFoundDesc")}
          </p>
        </div>
      )}

      {contextMenuElement}
    </div>
  );
}
