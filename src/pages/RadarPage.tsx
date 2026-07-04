import { Radio, Download, Terminal, RefreshCw } from "lucide-react";
import { useTranslation } from "react-i18next";

import TabNav, { type TabDef } from "../components/TabNav";
import NewsPanel from "../components/NewsPanel";
import Torrents1337Panel from "../components/Torrents1337Panel";
import SRPanel from "../components/SRPanel";
import SearchBox from "../components/shared/SearchBox";
import SortSelect from "../components/shared/SortSelect";

import { useRadarStore } from "../stores/useRadarStore";
import { useAppStore } from "../stores/useAppStore";
import { useScrape } from "../hooks/useScrape";

export default function RadarPage() {
  const { t } = useTranslation();
  const { showToast } = useAppStore();
  
  const { 
    activeTab, setActiveTab,
    searchVal, setSearchVal,
    sortVal, setSortVal
  } = useRadarStore();

  const { 
    isScraping, scrapeProgress, scrapeMessage, 
    startScrape, cancelScrape 
  } = useScrape({ 
    // Usually we might reload data, but TorrentsPanel manages its own query
  });

  const tabs: TabDef[] = [
    { id: "news", icon: Radio, labelKey: "tabNews" },
    { id: "torrents1337", icon: Download, labelKey: "tab1337" },
    { id: "sr", icon: Terminal, labelKey: "tabSR" }
  ];

  const rightAction = (
    <button 
      className="action-btn" 
      onClick={startScrape} 
      disabled={isScraping}
      style={{ 
        marginLeft: "auto", 
        padding: "0.4rem 0.9rem", 
        fontSize: "0.85rem", 
        borderRadius: "8px", 
        height: "32px", 
        display: "inline-flex", 
        alignItems: "center", 
        gap: "0.35rem" 
      }}
    >
      <RefreshCw size={14} className={isScraping ? "animate-spin" : ""} />
      {isScraping ? t("scraping") || "正在更新..." : t("getLatest") || "获取最新"}
    </button>
  );

  const sortOptions = [
    { value: "seeds-desc", label: "种子数从多到少 (Seeds)" },
    { value: "seeds-asc", label: "种子数从少到多 (Seeds)" },
    { value: "leeches-desc", label: "下载数从多到少 (Leechers)" },
    { value: "leeches-asc", label: "下载数从少到多 (Leechers)" },
    { value: "size-desc", label: "文件从大到小 (Size)" },
    { value: "size-asc", label: "文件从小到大 (Size)" }
  ];

  return (
    <>
      <TabNav 
        tabs={tabs} 
        activeTab={activeTab} 
        onTabChange={(id) => setActiveTab(id)} 
        rightAction={rightAction}
      />

      {activeTab === "torrents1337" && (
        <section className="controls-row">
          <SearchBox value={searchVal} onChange={setSearchVal} />
          <SortSelect 
            value={sortVal} 
            onChange={setSortVal} 
            options={sortOptions} 
            defaultLabel="默认排序 (Name)" 
          />
        </section>
      )}

      <div className="tab-content-scrollable">
        {activeTab === "news" && <NewsPanel />}
        
        {activeTab === "torrents1337" && (
          <Torrents1337Panel 
            showToast={showToast} 
            isScraping={isScraping} 
            scrapeProgress={scrapeProgress} 
            scrapeMessage={scrapeMessage} 
            onCancelScrape={cancelScrape} 
            searchVal={searchVal} 
            sortVal={sortVal} 
          />
        )}
        
        {activeTab === "sr" && <SRPanel showToast={showToast} />}
      </div>
    </>
  );
}
