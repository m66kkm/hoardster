import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import { BarChart, Bar, XAxis, YAxis, Tooltip, ResponsiveContainer, Cell, PieChart, Pie, Legend } from "recharts";
import type { GenreStat, RatingStat } from "../types";

interface DashboardProps {
  scanPaths: string[];
  onGenreClick?: (genre: string) => void;
  onRatingClick?: (rating: string) => void;
}

const COLORS = ['#00f2fe', '#10b981', '#f59e0b', '#8b5cf6', '#ef4444', '#ec4899', '#6366f1', '#4facfe'];

export default function Dashboard({ scanPaths, onGenreClick, onRatingClick }: DashboardProps) {
  const { t } = useTranslation();
  const [genreStats, setGenreStats] = useState<GenreStat[]>([]);
  const [ratingStats, setRatingStats] = useState<RatingStat[]>([]);

  useEffect(() => {
    invoke<GenreStat[]>("get_genre_stats_command")
      .then(setGenreStats)
      .catch(console.error);
    
    invoke<RatingStat[]>("get_rating_stats_command")
      .then(setRatingStats)
      .catch(console.error);
  }, []);

  return (
    <div>
      {genreStats.length > 0 && (
        <div className="panel" style={{ display: "block", marginBottom: "1.5rem" }}>
          <h3 style={{ fontSize: "1.4rem", fontWeight: 600, marginBottom: "1rem", display: "flex", alignItems: "center", gap: "0.5rem" }}>
            <span style={{ display: "inline-block", width: "8px", height: "8px", borderRadius: "50%", background: "var(--primary-accent)", boxShadow: "0 0 10px var(--primary-accent)" }}></span>
            {t("dashGenreTitle")}
          </h3>
          <p style={{ color: "var(--text-secondary)", marginBottom: "1.75rem", fontSize: "0.95rem" }}>
            {t("dashGenreDesc")}
          </p>
          <div style={{ width: "100%", height: "300px" }}>
            <ResponsiveContainer width="100%" height="100%">
              <BarChart data={genreStats} margin={{ top: 5, right: 30, left: 0, bottom: 25 }}>
                <XAxis 
                  dataKey="name" 
                  stroke="var(--text-secondary)" 
                  tick={{ fill: "var(--text-secondary)", fontSize: 12 }} 
                  axisLine={{ stroke: "var(--panel-border)" }}
                  tickLine={false}
                  angle={-45}
                  textAnchor="end"
                />
                <YAxis 
                  stroke="var(--text-secondary)" 
                  tick={{ fill: "var(--text-secondary)", fontSize: 12 }}
                  axisLine={false}
                  tickLine={false}
                  allowDecimals={false}
                />
                <Tooltip 
                  cursor={{ fill: 'rgba(0, 242, 254, 0.05)' }}
                  contentStyle={{ 
                    backgroundColor: 'var(--panel-bg)', 
                    borderColor: 'var(--panel-border)', 
                    borderRadius: '8px',
                    color: 'var(--text-primary)',
                    boxShadow: '0 8px 32px rgba(0,0,0,0.5)'
                  }}
                  itemStyle={{ color: 'var(--primary-accent)' }}
                />
                <Bar 
                  dataKey="count" 
                  radius={[4, 4, 0, 0]} 
                  maxBarSize={50}
                  onClick={(data) => {
                    const genreName = data?.name || data?.payload?.name;
                    if (onGenreClick && genreName) {
                      onGenreClick(genreName);
                    }
                  }}
                >
                  {genreStats.map((_entry, index) => (
                    <Cell 
                      key={`cell-${index}`} 
                      fill={index === 0 ? "var(--primary-accent)" : "rgba(0, 242, 254, 0.4)"} 
                      style={{ cursor: "pointer" }}
                    />
                  ))}
                </Bar>
              </BarChart>
            </ResponsiveContainer>
          </div>
        </div>
      )}

      {ratingStats.length > 0 && (
        <div className="panel" style={{ display: "block", marginBottom: "1.5rem" }}>
          <h3 style={{ fontSize: "1.4rem", fontWeight: 600, marginBottom: "1rem", display: "flex", alignItems: "center", gap: "0.5rem" }}>
            <span style={{ display: "inline-block", width: "8px", height: "8px", borderRadius: "50%", background: "var(--primary-accent)", boxShadow: "0 0 10px var(--primary-accent)" }}></span>
            {t("dashRatingTitle")}
          </h3>
          <p style={{ color: "var(--text-secondary)", marginBottom: "1.75rem", fontSize: "0.95rem" }}>
            {t("dashRatingDesc")}
          </p>
          <div style={{ width: "100%", height: "300px" }}>
            <ResponsiveContainer width="100%" height="100%">
              <BarChart data={ratingStats} margin={{ top: 5, right: 30, left: 0, bottom: 25 }}>
                <XAxis 
                  dataKey="name" 
                  stroke="var(--text-secondary)" 
                  tick={{ fill: "var(--text-secondary)", fontSize: 12 }} 
                  axisLine={{ stroke: "var(--panel-border)" }}
                  tickLine={false}
                  angle={-45}
                  textAnchor="end"
                />
                <YAxis 
                  stroke="var(--text-secondary)" 
                  tick={{ fill: "var(--text-secondary)", fontSize: 12 }}
                  axisLine={false}
                  tickLine={false}
                  allowDecimals={false}
                />
                <Tooltip 
                  cursor={{ fill: 'rgba(16, 185, 129, 0.05)' }}
                  contentStyle={{ 
                    backgroundColor: 'var(--panel-bg)', 
                    borderColor: 'var(--panel-border)', 
                    borderRadius: '8px',
                    color: 'var(--text-primary)',
                    boxShadow: '0 8px 32px rgba(0,0,0,0.5)'
                  }}
                  itemStyle={{ color: 'var(--success-color)' }}
                />
                <Bar 
                  dataKey="count" 
                  radius={[4, 4, 0, 0]} 
                  maxBarSize={50}
                  onClick={(data) => {
                    if (onRatingClick && data && data.name) {
                      onRatingClick(data.name);
                    }
                  }}
                  style={{ cursor: onRatingClick ? "pointer" : "default" }}
                >
                  {ratingStats.map((_entry, index) => (
                    <Cell key={`cell-${index}`} fill={index === 0 ? "var(--success-color)" : "rgba(16, 185, 129, 0.4)"} />
                  ))}
                </Bar>
              </BarChart>
            </ResponsiveContainer>
          </div>
        </div>
      )}

      <div className="panel" style={{ borderTopColor: "var(--primary-accent)", display: "block", marginBottom: "1.5rem" }}>
        <h3 style={{ fontSize: "1.4rem", fontWeight: 600, marginBottom: "1rem", display: "flex", alignItems: "center", gap: "0.5rem" }}>
          <span style={{ display: "inline-block", width: "8px", height: "8px", borderRadius: "50%", background: "var(--primary-accent)", boxShadow: "0 0 10px var(--primary-accent)" }}></span>
          {t("dashTitle")}
        </h3>
        <p style={{ color: "var(--text-secondary)", marginBottom: "1.75rem", maxWidth: "800px", fontSize: "0.95rem" }}>
          {t("dashSubtitle")}
        </p>
        <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(240px, 1fr))", gap: "1.5rem" }}>
          {scanPaths.map((path) => (
            <div key={path} style={{ background: "rgba(0,0,0,0.2)", border: "1px solid var(--panel-border)", borderRadius: "12px", padding: "1.25rem" }}>
              <div style={{ fontWeight: 600, color: "var(--primary-accent)", marginBottom: "0.5rem", display: "flex", justifyContent: "space-between" }}>
                <span>{path}</span>
              </div>
              <div style={{ color: "var(--text-secondary)", fontSize: "0.85rem", lineHeight: "1.5" }}>
                {t("dashPathDesc")}
              </div>
            </div>
          ))}
        </div>
      </div>
      
      <div className="panel" style={{ display: "block" }}>
        <h3 style={{ fontSize: "1.4rem", fontWeight: 600, marginBottom: "1.25rem" }}>{t("dashGuideTitle")}</h3>
        <div style={{ color: "var(--text-secondary)", fontSize: "0.95rem", lineHeight: "1.8" }}>
          <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(300px, 1fr))", gap: "1.5rem" }}>
            <div>
              <h4 style={{ color: "var(--primary-accent)", fontWeight: 600, fontSize: "1.05rem", marginBottom: "0.5rem" }}>{t("dashGuide1Title")}</h4>
              <p style={{ fontSize: "0.875rem" }}>{t("dashGuide1Desc")}</p>
            </div>
            <div>
              <h4 style={{ color: "var(--danger-color)", fontWeight: 600, fontSize: "1.05rem", marginBottom: "0.5rem" }}>{t("dashGuide2Title")}</h4>
              <p style={{ fontSize: "0.875rem" }}>{t("dashGuide2Desc")}</p>
            </div>
            <div>
              <h4 style={{ color: "var(--warning-color)", fontWeight: 600, fontSize: "1.05rem", marginBottom: "0.5rem" }}>{t("dashGuide3Title")}</h4>
              <p style={{ fontSize: "0.875rem" }}>{t("dashGuide3Desc")}</p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
