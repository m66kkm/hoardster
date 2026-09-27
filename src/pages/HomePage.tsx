import { useState, useEffect } from "react";
import { Monitor, Flame, BarChart2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";

import TabNav, { type TabDef } from "../components/TabNav";
import StatsGrid from "../components/StatsGrid";
import Dashboard from "../components/Dashboard";
import PopularPanel from "../components/PopularPanel";
import PosterWall from "../components/PosterWall";
import SearchBox from "../components/shared/SearchBox";
import ViewToggle from "../components/shared/ViewToggle";

import { useAppStore } from "../stores/useAppStore";
import { useGames } from "../hooks/useGames";
import { useSettings } from "../hooks/useSettings";
import type { StatsSummary } from "../types";

export default function HomePage() {
  const { t } = useTranslation();
  const { setMenuMode, showToast } = useAppStore();
  const { scanPaths, loadScanPaths } = useSettings();
  
  const [activeTab, setActiveTab] = useState<"dashboard" | "popular" | "stats">("dashboard");
  const [stats, setStats] = useState<StatsSummary>({
    total_scan: 0,
    unique_games: 0,
    franchise_count: 0,
    exact_dups: 0,
    version_dups: 0
  });
  const [currentPage, setCurrentPage] = useState(1);
  const [viewMode, setViewMode] = useState<"tile" | "detail">("tile");
  const [searchVal, setSearchVal] = useState("");

  const { gamesList, loadGames } = useGames({
    searchVal,
    driveVal: "",
    typeVal: "",
    ratingVal: "",
    sortVal: ""
  });

  useEffect(() => {
    invoke<StatsSummary>("get_games_stats_command")
      .then(setStats)
      .catch(console.error);
    loadScanPaths();
  }, [loadScanPaths]);

  useEffect(() => {
    if (activeTab === "dashboard") {
      loadGames(true, false);
    } else if (activeTab === "stats") {
      invoke<StatsSummary>("get_games_stats_command")
        .then(setStats)
        .catch(console.error);
    }
  }, [activeTab, searchVal, loadGames]);

  const tabs: TabDef[] = [
    { id: "dashboard", icon: Monitor, labelKey: "tabDashboard" },
    { id: "popular", icon: Flame, labelKey: "tabPopular" },
    { id: "stats", icon: BarChart2, labelKey: "tabStats" }
  ];

  const handleSearchInLocal = (_title: string) => {
    setMenuMode("local");
    // Ideally we'd pass this to LocalGamesStore too, but let's stick to the prompt's request
  };

  const copyPath = (path: string, gameName: string) => {
    navigator.clipboard.writeText(path).then(() => {
      showToast(t("toastPathCopied", { name: gameName }));
    }).catch(err => {
      console.error('Failed to copy text: ', err);
      showToast(t("toastPathCopyFailed"));
    });
  };

  const openGameFolder = async (path: string) => {
    try {
      await invoke("open_game_folder_command", { path });
    } catch (e) {
      console.error(e);
      showToast(t("toastOpenFolderFailed"));
    }
  };

  return (
    <>
      <TabNav 
        tabs={tabs} 
        activeTab={activeTab} 
        onTabChange={(tab) => setActiveTab(tab as "dashboard" | "popular" | "stats")} 
      />

      {activeTab === "dashboard" && (
        <section className="controls-row">
          <SearchBox value={searchVal} onChange={setSearchVal} />
          <ViewToggle value={viewMode} onChange={setViewMode} />
        </section>
      )}

      <div className="tab-content-scrollable">
        {activeTab === "dashboard" && (
          <PosterWall 
            games={gamesList} 
            viewMode={viewMode} 
            currentPage={currentPage} 
            setCurrentPage={setCurrentPage} 
            pageSize={36} 
            title={t("wallTitleHome") || "全量游戏海报墙"} 
            subtitle={t("wallSubtitle")} 
            copyPath={copyPath} 
            openGameFolder={openGameFolder} 
            onRefresh={() => loadGames(true, false)}
          />
        )}

        {activeTab === "popular" && (
          <PopularPanel onSearchInLocal={handleSearchInLocal} />
        )}

        {activeTab === "stats" && (
          <div style={{ padding: "0 0.5rem" }}>
            <StatsGrid stats={stats} onCardClick={() => {}} />
            <Dashboard 
              scanPaths={scanPaths} 
              onGenreClick={() => {}} 
              onRatingClick={() => {}} 
            />
          </div>
        )}
      </div>
    </>
  );
}
