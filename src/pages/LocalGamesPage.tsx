import { useState, useEffect, useCallback } from "react";
import { Database, HardDrive, Layers, Play } from "lucide-react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

import TabNav, { type TabDef } from "../components/TabNav";
import PosterWall from "../components/PosterWall";
import DuplicatesPanel from "../components/DuplicatesPanel";
import FranchisesPanel from "../components/FranchisesPanel";
import SearchBox from "../components/shared/SearchBox";
import FilterSelect from "../components/shared/FilterSelect";
import SortSelect from "../components/shared/SortSelect";
import ViewToggle from "../components/shared/ViewToggle";

import { useLocalGamesStore } from "../stores/useLocalGamesStore";
import { useAppStore } from "../stores/useAppStore";
import { useGames } from "../hooks/useGames";
import { useScan } from "../hooks/useScan";
import { useSettings } from "../hooks/useSettings";
import type { Game } from "../types";

export default function LocalGamesPage() {
  const { t } = useTranslation();
  const { showToast } = useAppStore();
  
  const { 
    activeTab, setActiveTab,
    searchVal, setSearchVal,
    driveVal, setDriveVal,
    typeVal, setTypeVal,
    ratingVal, setRatingVal,
    sortVal, setSortVal,
    currentPage, setCurrentPage,
    viewMode, setViewMode
  } = useLocalGamesStore();

  const { scanPaths, loadScanPaths } = useSettings();
  
  const [genres, setGenres] = useState<string[]>([]);
  const [openAccordions, setOpenAccordions] = useState<Record<string, boolean>>({});

  const toggleAccordion = (name: string) => {
    setOpenAccordions(prev => ({ ...prev, [name]: !prev[name] }));
  };

  const { 
    gamesList, exactDuplicates, versionDuplicates, franchises,
    loadGames, loadDuplicates, loadFranchises
  } = useGames({ searchVal, driveVal, typeVal, ratingVal, sortVal });

  const reloadData = useCallback(() => {
    if (activeTab === "all") loadGames(false, false, true);
    else if (activeTab === "installed") loadGames(true, true, false);
    else if (activeTab === "duplicates") loadDuplicates();
    else if (activeTab === "franchise") loadFranchises();
  }, [activeTab, loadGames, loadDuplicates, loadFranchises]);

  const { isScanning, startScan } = useScan({ onComplete: reloadData });

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    listen("games-updated", () => {
      reloadData();
    }).then(fn => {
      unlisten = fn;
    });

    return () => {
      if (unlisten) unlisten();
    };
  }, [reloadData]);

  // Reset page when filters or tab change
  useEffect(() => {
    setCurrentPage(1);
  }, [searchVal, driveVal, typeVal, ratingVal, sortVal, activeTab, setCurrentPage]);

  // Load data when tab or filters change
  useEffect(() => {
    reloadData();
  }, [activeTab, searchVal, driveVal, typeVal, ratingVal, sortVal]);

  // Initial load
  useEffect(() => {
    loadScanPaths();
    invoke<string[]>("get_all_genres_command")
      .then(setGenres)
      .catch(console.error);
  }, [loadScanPaths]);

  // Scroll to top when active tab changes
  useEffect(() => {
    const scrollContainer = document.querySelector(".tab-content-scrollable");
    if (scrollContainer) {
      scrollContainer.scrollTo({ top: 0, behavior: "smooth" });
    }
  }, [activeTab]);

  const tabs: TabDef[] = [
    { id: "installed", icon: HardDrive, labelKey: "tabInstalled" },
    { id: "all", icon: Database, labelKey: "tabAll" },
    { id: "franchise", icon: Layers, labelKey: "tabFranchise" },
    { id: "duplicates", icon: Layers, labelKey: "tabDuplicates" }
  ];

  const rightAction = (
    <button 
      className="action-btn" 
      onClick={startScan} 
      disabled={isScanning}
      style={{ 
        marginLeft: "auto", 
        padding: "0.4rem 0.9rem", 
        fontSize: "0.85rem", 
        borderRadius: "8px", 
        height: "32px", 
        display: "inline-flex", 
        alignItems: "center" 
      }}
    >
      <Play size={14} className={isScanning ? "animate-spin" : ""} />
      {isScanning ? t("scanning") : t("incScan")}
    </button>
  );

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

  const handleDeleteGame = async (game: Game) => {
    const gameName = game.name || game.original_name;
    showToast(t("toastDeletingGame", { name: gameName }));
    try {
      await invoke("delete_game_directory_command", { path: game.full_path });
      showToast(t("toastDeleteSuccess", { name: gameName }));
      reloadData();
    } catch (e: any) {
      console.error(e);
      showToast(`${t("toastDeleteFailed")}: ${e?.toString() || e}`);
    }
  };

  const sortOptions = [
    { value: "name-asc", label: t("filterSortNameAsc") },
    { value: "name-desc", label: t("filterSortNameDesc") },
    { value: "steam-desc", label: t("filterSortSteamDesc") },
    { value: "steam-asc", label: t("filterSortSteamAsc") },
    { value: "size-desc", label: t("filterSortSizeDesc") },
    { value: "size-asc", label: t("filterSortSizeAsc") }
  ];

  const ratingFilterOptions = [
    { value: "positive", label: t("srFilterPositive") || "Steam好评 (≥70%)" },
    { value: "very_positive", label: t("srFilterVeryPositive") || "特别好评 (≥80%)" },
    { value: "overwhelmingly_positive", label: t("srFilterOverwhelminglyPositive") || "好评如潮 (≥95%)" },
    { value: "mixed_plus", label: t("srFilterMixedPlus") || "褒贬不一及以上 (≥40%)" },
    { value: "has_rating", label: t("srFilterHasRating") || "仅看有评价" },
  ];

  return (
    <>
      <TabNav 
        tabs={tabs} 
        activeTab={activeTab} 
        onTabChange={(id) => setActiveTab(id)} 
        rightAction={rightAction}
      />

      <section className="controls-row">
        <SearchBox value={searchVal} onChange={setSearchVal} />
        <FilterSelect 
          value={driveVal} 
          onChange={setDriveVal} 
          options={scanPaths} 
          allLabel={t("filterAllDrives")} 
        />
        <FilterSelect 
          value={typeVal} 
          onChange={setTypeVal} 
          options={genres} 
          allLabel="所有游戏类型" 
        />
        <FilterSelect 
          value={ratingVal} 
          onChange={setRatingVal} 
          options={ratingFilterOptions} 
          allLabel={t("srFilterAllRatings") || "所有Steam评价"} 
        />
        <SortSelect 
          value={sortVal} 
          onChange={setSortVal} 
          options={sortOptions} 
          defaultLabel={t("filterSortDefault")} 
        />
        {(activeTab === "installed" || activeTab === "all") && (
          <ViewToggle value={viewMode} onChange={setViewMode} />
        )}
      </section>

      <div className="tab-content-scrollable">
        {(activeTab === "installed" || activeTab === "all") && (
          <PosterWall 
            games={gamesList} 
            viewMode={viewMode} 
            currentPage={currentPage} 
            setCurrentPage={setCurrentPage} 
            pageSize={36} 
            title={activeTab === "installed" ? t("wallTitleInstalled") : t("wallTitleArchived")} 
            subtitle={t("wallSubtitle")} 
            copyPath={copyPath} 
            openGameFolder={openGameFolder} 
            onDeleteGame={handleDeleteGame}
            onRefresh={reloadData}
          />
        )}
        
        {activeTab === "duplicates" && (
          <DuplicatesPanel 
            exactDuplicates={exactDuplicates} 
            versionDuplicates={versionDuplicates} 
            copyPath={copyPath} 
            openGameFolder={openGameFolder} 
            onDeleteGame={handleDeleteGame}
            onRefresh={reloadData}
          />
        )}
        
        {activeTab === "franchise" && (
          <FranchisesPanel 
            franchises={franchises} 
            openAccordions={openAccordions} 
            toggleAccordion={toggleAccordion} 
            copyPath={copyPath} 
            openGameFolder={openGameFolder} 
          />
        )}
      </div>
    </>
  );
}
