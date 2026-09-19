import { useTranslation } from "react-i18next";
import { ChevronDown, ExternalLink, Gamepad2 } from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";
import type { FranchiseGroup } from "../types";
import { getRatingColorClass, getCoverUrl, getReviewScoreText, getTypeBadgeClass, getSteamStoreUrl, openExternalUrl } from "../utils/helpers";

interface FranchisesPanelProps {
  franchises: FranchiseGroup[];
  openAccordions: { [key: string]: boolean };
  toggleAccordion: (key: string) => void;
  copyPath: (path: string, gameName: string) => void;
  openGameFolder: (path: string) => void;
}

export default function FranchisesPanel({ franchises, openAccordions, toggleAccordion, copyPath, openGameFolder }: FranchisesPanelProps) {
  const { t } = useTranslation();

  return (
    <div className="panel" style={{ display: "block" }}>
      <div className="panel-header">
        <h2>{t("franTitle")}</h2>
        <p style={{ color: "var(--text-secondary)", fontSize: "0.9rem", marginTop: "0.25rem" }}>
          {t("franSubtitle")}
        </p>
      </div>

      <div className="accordion-container">
        {franchises.map((group) => {
          const isOpen = !!openAccordions[group.prefix];
          return (
            <div key={group.prefix} className={`accordion-item ${isOpen ? "open" : ""}`}>
              <div className="accordion-header" onClick={() => toggleAccordion(group.prefix)}>
                <span>{t("franGroup", { name: group.prefix, count: group.games.length })}</span>
                <span style={{ display: "flex", alignItems: "center", gap: "0.5rem", fontSize: "0.9rem", color: "var(--text-secondary)" }}>
                  <motion.div animate={{ rotate: isOpen ? 180 : 0 }} transition={{ duration: 0.2 }}>
                    <ChevronDown className="accordion-icon" size={16} style={{ transform: "none" }} />
                  </motion.div>
                </span>
              </div>
              <AnimatePresence initial={false}>
                {isOpen && (
                  <motion.div
                    initial={{ height: 0, opacity: 0 }}
                    animate={{ height: "auto", opacity: 1 }}
                    exit={{ height: 0, opacity: 0 }}
                    transition={{ duration: 0.3, ease: "easeInOut" }}
                    style={{ overflow: "hidden", display: "block" }}
                    className="accordion-content"
                  >
                    <div className="table-container" style={{ border: "none", borderRadius: 0 }}>
                  <table>
                    <thead>
                      <tr>
                        <th>{t("colName")}</th>
                        <th style={{ width: "160px" }}>{t("colSteamRating")}</th>
                        <th style={{ width: "100px" }}>{t("colFile")}</th>
                        <th>{t("colPath")}</th>
                        <th style={{ width: "80px", textAlign: "center" }}>{t("colAction")}</th>
                      </tr>
                    </thead>
                    <tbody>
                      {group.games.map((g) => (
                        <tr key={g.full_path}>
                          <td 
                            style={{ fontWeight: 600, color: "var(--text-primary)", display: "flex", alignItems: "center", gap: "0.75rem", cursor: "pointer" }} 
                            onClick={() => openGameFolder(g.full_path)} 
                            title={t("openInExplorer")}
                          >
                            {g.local_cover ? (
                              <img src={getCoverUrl(g.local_cover) || ""} style={{ width: "32px", height: "48px", objectFit: "cover", borderRadius: "4px", border: "1px solid var(--panel-border)" }} alt="" />
                            ) : (
                              <span style={{ display: "inline-block", width: "32px", height: "48px", background: "rgba(255,255,255,0.05)", borderRadius: "4px", textAlign: "center", lineHeight: "48px", fontSize: "1.2rem" }}>🎮</span>
                            )}
                            {g.original_name}
                          </td>
                          <td>
                            {g.review_score_desc !== undefined && g.review_score_desc !== null && g.review_score_desc !== "" && Number(g.review_score_desc) > 0 ? (
                              <span 
                                className={`rating-text ${getRatingColorClass(g.review_score_desc)} clickable-rating`}
                                onClick={(e) => {
                                  e.stopPropagation();
                                  openExternalUrl(getSteamStoreUrl(g.appid, g.base_name, g.original_name));
                                }}
                                title={`${t("steamRatingHover", { percent: g.positive_percent, total: g.total_reviews })}\n${t("tipOpenSteam") || "点击在浏览器中打开 Steam 商店页面"}`}
                              >
                                {g.positive_percent !== undefined && g.positive_percent !== null && Number(g.positive_percent) > 0 ? `👍 ${g.positive_percent}% ` : ""}
                                ({getReviewScoreText(t, g.review_score_desc)})
                                <ExternalLink size={10} style={{ marginLeft: 4, opacity: 0.7 }} />
                              </span>
                            ) : (
                              <span style={{ fontSize: "0.75rem", color: "var(--text-secondary)", opacity: 0.5 }}>{t("noRating") || "暂无评价"}</span>
                            )}
                          </td>
                          <td>
                            <span className={`badge ${getTypeBadgeClass(g.type)}`}>{g.type}</span>
                          </td>
                          <td>
                            <span className="code-path" onClick={() => copyPath(g.full_path, g.original_name)} title={t("copyPathMsg")}>{g.full_path}</span>
                          </td>
                          <td style={{ textAlign: "center" }}>
                            <div style={{ display: "flex", gap: "0.35rem", justifyContent: "center" }}>
                              {(g.appid || (g.review_score_desc && Number(g.review_score_desc) > 0)) && (
                                <button 
                                  className="view-btn" 
                                  onClick={() => openExternalUrl(getSteamStoreUrl(g.appid, g.base_name, g.original_name))} 
                                  title={t("tipOpenSteam") || "点击在浏览器中打开 Steam 商店页面"} 
                                  style={{ padding: "0.4rem", display: "inline-flex", color: "var(--primary-accent)" }}
                                >
                                  <Gamepad2 size={12} />
                                </button>
                              )}
                              <button className="view-btn" onClick={() => openGameFolder(g.full_path)} title={t("openInExplorer")} style={{ padding: "0.4rem", display: "inline-flex" }}>
                                <ExternalLink size={12} />
                              </button>
                            </div>
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
                  </motion.div>
                )}
              </AnimatePresence>
            </div>
          );
        })}

        {franchises.length === 0 && (
          <div style={{ textAlign: "center", padding: "4rem", color: "var(--text-secondary)", border: "1px dashed var(--panel-border)", borderRadius: "12px" }}>
            {t("noResults")}
          </div>
        )}
      </div>
    </div>
  );
}
