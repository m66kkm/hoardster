import type { LucideIcon } from "lucide-react";
import { useTranslation } from "react-i18next";
import { motion } from "framer-motion";
import type { ReactNode } from "react";

export interface TabDef {
  id: string;
  icon: LucideIcon;
  labelKey: string;
}

interface TabNavProps {
  tabs: TabDef[];
  activeTab: string;
  onTabChange: (tab: string) => void;
  rightAction?: ReactNode;
}

export default function TabNav({ tabs, activeTab, onTabChange, rightAction }: TabNavProps) {
  const { t } = useTranslation();

  return (
    <nav className="tabs-nav" style={{ display: "flex", alignItems: "center" }}>
      {tabs.map((tab) => {
        const Icon = tab.icon;
        const isActive = activeTab === tab.id;
        return (
          <button
            key={tab.id}
            className={`tab-btn ${isActive ? "active" : ""}`}
            onClick={() => onTabChange(tab.id)}
          >
            {isActive && (
              <motion.div
                layoutId="activeTabIndicator"
                initial={false}
                transition={{ type: "spring", stiffness: 500, damping: 30 }}
                style={{
                  position: "absolute",
                  inset: 0,
                  background: "var(--primary-accent)",
                  boxShadow: "0 0 12px rgba(0, 242, 254, 0.2)",
                  borderRadius: "10px",
                  zIndex: 0
                }}
              />
            )}
            <span style={{ position: "relative", zIndex: 1, display: "flex", alignItems: "center", gap: "0.5rem" }}>
              <Icon size={15} />
              {t(tab.labelKey)}
            </span>
          </button>
        );
      })}

      {rightAction}
    </nav>
  );
}
