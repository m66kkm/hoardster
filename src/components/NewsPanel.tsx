import { useState } from "react";
import { Radio, Calendar, Clock, ChevronRight, ExternalLink } from "lucide-react";
import { useTranslation } from "react-i18next";

import { NEWS_DATA } from "../mocks/newsData";

export default function NewsPanel() {
  const { t } = useTranslation();
  const [expandedId, setExpandedId] = useState<number | null>(null);

  const toggleExpand = (id: number) => {
    setExpandedId(prev => (prev === id ? null : id));
  };

  const getCategoryBadgeClass = (category: string) => {
    switch (category) {
      case "情报": return "badge-dir"; // cyan/blueish
      case "特惠": return "badge-ver"; // gold/orange
      case "更新": return "badge-installed"; // green
      case "Repack": return "badge-dup"; // red/magenta
      case "限免": return "badge-active"; // purple
      default: return "";
    }
  };

  return (
    <div className="panel" style={{ display: "block" }}>
      <div className="panel-header" style={{ marginBottom: "1.5rem" }}>
        <h2 style={{ margin: 0, display: "flex", alignItems: "center", gap: "0.5rem" }}>
          <Radio size={22} style={{ color: "var(--primary-accent)", animation: "pulse 2s infinite" }} />
          {t("newsTitle")}
        </h2>
        <p style={{ color: "var(--text-secondary)", fontSize: "0.9rem", marginTop: "0.25rem", marginBottom: 0 }}>
          {t("newsDesc")}
        </p>
      </div>

      <div className="news-list" style={{ display: "flex", flexDirection: "column", gap: "1rem" }}>
        {NEWS_DATA.map((item) => {
          const isExpanded = expandedId === item.id;
          return (
            <div 
              key={item.id} 
              className="news-card"
              style={{
                background: "rgba(15, 23, 42, 0.4)",
                border: "1px solid var(--panel-border)",
                borderRadius: "12px",
                padding: "1.25rem",
                transition: "all 0.2s ease",
                cursor: "pointer",
                position: "relative"
              }}
              onClick={() => toggleExpand(item.id)}
            >
              {/* Top metadata row */}
              <div style={{ display: "flex", alignItems: "center", gap: "0.75rem", marginBottom: "0.75rem" }}>
                <span className={`badge ${getCategoryBadgeClass(item.category)}`}>
                  {item.category}
                </span>
                <span style={{ display: "inline-flex", alignItems: "center", gap: "0.25rem", color: "var(--text-secondary)", fontSize: "0.8rem" }}>
                  <Calendar size={12} />
                  {item.time}
                </span>
                <span style={{ display: "inline-flex", alignItems: "center", gap: "0.25rem", color: "var(--text-secondary)", fontSize: "0.8rem" }}>
                  <Clock size={12} />
                  {item.readTime}
                </span>
                <span style={{ marginLeft: "auto", display: "inline-flex", alignItems: "center", gap: "0.25rem", color: "var(--text-secondary)", fontSize: "0.8rem" }}>
                  {t("newsSource")}: {item.source}
                </span>
              </div>

              {/* Title & Chevron */}
              <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", gap: "1rem" }}>
                <h3 style={{ margin: 0, fontSize: "1.1rem", fontWeight: 650, color: "var(--text-primary)" }}>
                  {item.title}
                </h3>
                <ChevronRight 
                  size={18} 
                  style={{ 
                    transform: isExpanded ? "rotate(90deg)" : "rotate(0deg)", 
                    transition: "transform 0.2s ease",
                    color: "var(--text-secondary)"
                  }} 
                />
              </div>

              {/* Summary */}
              <p style={{
                fontSize: "0.9rem",
                color: "var(--text-secondary)",
                marginTop: "0.75rem",
                marginBottom: 0,
                lineHeight: 1.6
              }}>
                {item.summary}
              </p>

              {/* Expanded details */}
              {isExpanded && (
                <div 
                  style={{ 
                    marginTop: "1rem", 
                    paddingTop: "1rem", 
                    borderTop: "1px dashed rgba(255,255,255,0.08)",
                    fontSize: "0.9rem",
                    color: "var(--text-primary)",
                    lineHeight: 1.6,
                    background: "rgba(255,255,255,0.02)",
                    padding: "0.75rem 1rem",
                    borderRadius: "8px"
                  }}
                  onClick={(e) => e.stopPropagation()} // prevent double-closing when clicking text
                >
                  <p style={{ margin: 0 }}>{item.details}</p>
                  <div style={{ display: "flex", justifyContent: "flex-end", marginTop: "0.75rem" }}>
                    <a 
                      href="#" 
                      className="action-btn"
                      style={{ padding: "0.4rem 1rem", fontSize: "0.85rem", height: "auto", display: "inline-flex", alignItems: "center", gap: "0.35rem" }}
                      onClick={(e) => { e.preventDefault(); alert(t("newsReadOriginal") + ": " + item.source); }}
                    >
                      {t("newsReadOriginal")}
                      <ExternalLink size={14} />
                    </a>
                  </div>
                </div>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}
