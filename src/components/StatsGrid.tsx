import { useState } from "react";
import type { StatsSummary } from "../types";
import { useTranslation } from "react-i18next";
import { Info } from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";

function StatTooltip({ text }: { text: string }) {
  const [show, setShow] = useState(false);
  return (
    <div 
      style={{ position: "relative", display: "inline-flex", alignItems: "center" }}
      onMouseEnter={() => setShow(true)}
      onMouseLeave={() => setShow(false)}
    >
      <Info size={15} style={{ opacity: 0.5, cursor: "help" }} />
      <AnimatePresence>
        {show && (
          <motion.div
            initial={{ opacity: 0, y: 5, x: "-50%" }}
            animate={{ opacity: 1, y: 0, x: "-50%" }}
            exit={{ opacity: 0, y: 5, x: "-50%" }}
            transition={{ duration: 0.15 }}
            style={{
              position: "absolute",
              bottom: "100%",
              left: "50%",
              marginBottom: "8px",
              background: "rgba(15, 23, 42, 0.95)",
              border: "1px solid var(--primary-accent)",
              padding: "0.5rem 0.85rem",
              borderRadius: "8px",
              color: "var(--text-primary)",
              fontSize: "0.85rem",
              whiteSpace: "nowrap",
              boxShadow: "0 4px 20px rgba(0, 0, 0, 0.6), 0 0 10px rgba(0, 242, 254, 0.2)",
              zIndex: 50,
              pointerEvents: "none",
              fontWeight: "normal",
              textTransform: "none",
              letterSpacing: "normal"
            }}
          >
            {text}
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  );
}

interface StatsGridProps {
  stats: StatsSummary;
  onCardClick: (tab: string) => void;
}

export default function StatsGrid({ stats, onCardClick }: StatsGridProps) {
  const { t } = useTranslation();

  return (
    <section className="stats-grid">
      <div className="stat-card" onClick={() => onCardClick("all")}>
        <div className="stat-label" style={{ display: "flex", alignItems: "center", gap: "0.4rem" }}>
          <span>{t("statTotalScanned")}</span>
          <StatTooltip text={t("statTotalScannedDesc")} />
        </div>
        <div className="stat-value">{stats.total_scan}</div>
      </div>
      <div className="stat-card success" onClick={() => onCardClick("posters")}>
        <div className="stat-label" style={{ display: "flex", alignItems: "center", gap: "0.4rem" }}>
          <span>{t("statUniqueGames")}</span>
          <StatTooltip text={t("statUniqueGamesDesc")} />
        </div>
        <div className="stat-value">{stats.unique_games}</div>
      </div>
      <div className="stat-card warning" onClick={() => onCardClick("franchise")}>
        <div className="stat-label" style={{ display: "flex", alignItems: "center", gap: "0.4rem" }}>
          <span>{t("statFranchise")}</span>
          <StatTooltip text={t("statFranchiseDesc")} />
        </div>
        <div className="stat-value">{stats.franchise_count || 0}</div>
      </div>
      <div className="stat-card danger" onClick={() => onCardClick("duplicates")}>
        <div className="stat-label" style={{ display: "flex", alignItems: "center", gap: "0.4rem" }}>
          <span>{t("statDuplicates")}</span>
          <StatTooltip text={t("statDuplicatesDesc")} />
        </div>
        <div className="stat-value">{(stats.exact_dups || 0) + (stats.version_dups || 0)}</div>
      </div>
    </section>
  );
}
