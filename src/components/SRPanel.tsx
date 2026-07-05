import { useState } from "react";
import { Terminal, Eye, X, Copy } from "lucide-react";
import { useTranslation } from "react-i18next";

import { RELEASES } from "../mocks/releaseData";

interface SRPanelProps {
  showToast: (msg: string) => void;
}

export default function SRPanel({ showToast }: SRPanelProps) {
  const { t } = useTranslation();
  const [selectedGroup, setSelectedGroup] = useState<string>("ALL");
  const [viewingNfo, setViewingNfo] = useState<string | null>(null);
  const [viewingNfoName, setViewingNfoName] = useState<string>("");

  const filteredReleases = selectedGroup === "ALL" 
    ? RELEASES 
    : RELEASES.filter(r => r.group === selectedGroup);

  const copyNfo = () => {
    if (viewingNfo) {
      navigator.clipboard.writeText(viewingNfo).then(() => {
        showToast(t("srCopySuccess"));
      });
    }
  };

  return (
    <div className="panel" style={{ display: "block" }}>
      <div className="panel-header" style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "1.5rem", flexWrap: "wrap", gap: "1rem" }}>
        <div>
          <h2 style={{ margin: 0, display: "flex", alignItems: "center", gap: "0.5rem" }}>
            <Terminal size={22} style={{ color: "var(--primary-accent)" }} />
            {t("srTitle")}
          </h2>
          <p style={{ color: "var(--text-secondary)", fontSize: "0.9rem", marginTop: "0.25rem", marginBottom: 0 }}>
            {t("srDesc")}
          </p>
        </div>

        {/* Group Selector */}
        <div style={{ display: "flex", gap: "0.35rem", background: "rgba(255,255,255,0.03)", padding: "0.25rem", borderRadius: "8px", border: "1px solid var(--panel-border)" }}>
          {["ALL", "RUNE", "TENOKE", "FLT", "FitGirl", "DODI"].map(g => (
            <button
              key={g}
              onClick={() => setSelectedGroup(g)}
              style={{
                background: selectedGroup === g ? "var(--primary-accent)" : "transparent",
                color: selectedGroup === g ? "var(--bg-color)" : "var(--text-secondary)",
                border: "none",
                padding: "0.35rem 0.75rem",
                borderRadius: "6px",
                fontSize: "0.8rem",
                fontWeight: 600,
                cursor: "pointer",
                transition: "all 0.2s ease"
              }}
            >
              {g}
            </button>
          ))}
        </div>
      </div>

      <div className="table-container">
        <table>
          <thead>
            <tr>
              <th>{t("srColName")}</th>
              <th style={{ width: "120px" }}>{t("srColType")}</th>
              <th style={{ width: "100px" }}>{t("srColSize")}</th>
              <th style={{ width: "120px" }}>{t("srColDate")}</th>
              <th style={{ width: "100px" }}>{t("srColGroup")}</th>
              <th style={{ width: "100px", textAlign: "center" }}>{t("srColNFO")}</th>
            </tr>
          </thead>
          <tbody>
            {filteredReleases.map((r) => (
              <tr key={r.id}>
                <td style={{ fontWeight: 600, color: "var(--text-primary)" }}>{r.releaseName}</td>
                <td>
                  <span className={`badge ${r.type === "Scene Crack" ? "badge-dup" : r.type === "Repack" ? "badge-ver" : "badge-installed"}`}>
                    {r.type}
                  </span>
                </td>
                <td style={{ fontFamily: "'Outfit', sans-serif" }}>{r.size}</td>
                <td>{r.date}</td>
                <td>
                  <span className="badge badge-dir">{r.group}</span>
                </td>
                <td style={{ textAlign: "center" }}>
                  <button 
                    className="view-btn"
                    onClick={() => {
                      setViewingNfo(r.nfoContent);
                      setViewingNfoName(r.releaseName);
                    }}
                    title={t("srViewNFO")}
                    style={{ padding: "0.4rem", display: "inline-flex" }}
                  >
                    <Eye size={12} />
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {/* Modal/Overlay for NFO Viewer */}
      {viewingNfo && (
        <div style={{
          position: "fixed",
          top: 0,
          left: 0,
          right: 0,
          bottom: 0,
          background: "rgba(0, 0, 0, 0.8)",
          display: "flex",
          justifyContent: "center",
          alignItems: "center",
          zIndex: 1000,
          padding: "2rem"
        }}>
          <div style={{
            background: "#0a0f1d",
            border: "1px solid var(--primary-accent)",
            borderRadius: "14px",
            width: "100%",
            maxWidth: "720px",
            maxHeight: "85vh",
            display: "flex",
            flexDirection: "column",
            boxShadow: "0 0 30px rgba(0, 242, 254, 0.15)",
            overflow: "hidden"
          }}>
            {/* Modal Header */}
            <div style={{
              display: "flex",
              justifyContent: "space-between",
              alignItems: "center",
              padding: "1rem 1.25rem",
              background: "rgba(255,255,255,0.02)",
              borderBottom: "1px solid rgba(255,255,255,0.05)"
            }}>
              <div style={{ display: "flex", alignItems: "center", gap: "0.5rem" }}>
                <Terminal size={18} style={{ color: "var(--primary-accent)" }} />
                <span style={{ fontWeight: 600, fontSize: "0.95rem", color: "var(--text-primary)" }}>{viewingNfoName}</span>
              </div>
              <div style={{ display: "flex", gap: "0.5rem" }}>
                <button 
                  onClick={copyNfo}
                  style={{
                    background: "rgba(255,255,255,0.05)",
                    border: "none",
                    color: "var(--text-primary)",
                    padding: "0.35rem 0.65rem",
                    borderRadius: "6px",
                    cursor: "pointer",
                    fontSize: "0.75rem",
                    display: "flex",
                    alignItems: "center",
                    gap: "0.25rem"
                  }}
                >
                  <Copy size={12} />
                  {t("srCopyNFO")}
                </button>
                <button 
                  onClick={() => setViewingNfo(null)}
                  style={{
                    background: "rgba(239, 68, 68, 0.15)",
                    border: "none",
                    color: "#ef4444",
                    padding: "0.35rem",
                    borderRadius: "6px",
                    cursor: "pointer",
                    display: "flex"
                  }}
                >
                  <X size={14} />
                </button>
              </div>
            </div>

            {/* Modal Body */}
            <pre style={{
              padding: "1.25rem",
              margin: 0,
              overflowY: "auto",
              fontFamily: "'Courier New', Courier, monospace",
              fontSize: "0.85rem",
              lineHeight: 1.4,
              color: "#38bdf8",
              background: "#020617",
              textAlign: "left",
              whiteSpace: "pre-wrap"
            }}>
              {viewingNfo}
            </pre>
          </div>
        </div>
      )}
    </div>
  );
}
