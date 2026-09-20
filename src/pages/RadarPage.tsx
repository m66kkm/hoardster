import { Download, Terminal, Gift, Gamepad2, RefreshCw } from "lucide-react";

import TabNav, { type TabDef } from "../components/TabNav";
import Torrents1337Panel from "../components/Torrents1337Panel";
import SRPanel from "../components/SRPanel";

import { useRadarStore } from "../stores/useRadarStore";
import { useAppStore } from "../stores/useAppStore";
import { useScrape } from "../hooks/useScrape";
import { useScrapeStore, getTaskKey } from "../stores/useScrapeStore";
import EpicGamesPanel from "../components/EpicGamesPanel";
import SteamGamesPanel from "../components/SteamGamesPanel";

export default function RadarPage() {
  const { showToast } = useAppStore();
  
  const { 
    activeTab, setActiveTab,
    searchVal, setSearchVal,
    sortVal, setSortVal,
    ratingFilter, setRatingFilter
  } = useRadarStore();

  const { 
    isScraping, scrapeProgress, scrapeMessage, 
    startScrape, cancelScrape 
  } = useScrape({ 
    onComplete: (msg) => {
      if (msg) showToast(msg);
    }
  });

  const is1337Scraping = isScraping;
  const isSrScraping = useScrapeStore((s) => !!s.tasks[getTaskKey("sr")]?.isScraping);

  const tabs: TabDef[] = [
    { 
      id: "sr", 
      icon: isSrScraping ? RefreshCw : Terminal, 
      labelKey: "tabSR", 
      isSpinning: isSrScraping 
    },
    { 
      id: "torrents1337", 
      icon: is1337Scraping ? RefreshCw : Download, 
      labelKey: "tab1337", 
      isSpinning: is1337Scraping 
    },
    { id: "epic", icon: Gift, labelKey: "tabEpic" },
    { id: "steam", icon: Gamepad2, labelKey: "tabSteam" },
  ];

  // 如果原本持久化了已隐藏的 news tab，自动切到 sr
  const effectiveTab = activeTab === "news" ? "sr" : activeTab;

  return (
    <>
      <TabNav 
        tabs={tabs} 
        activeTab={effectiveTab} 
        onTabChange={(id) => setActiveTab(id)} 
      />

      <div className="tab-content-scrollable">
        {effectiveTab === "sr" && <SRPanel showToast={showToast} />}
        
        {effectiveTab === "torrents1337" && (
          <Torrents1337Panel 
            showToast={showToast} 
            isScraping={isScraping} 
            scrapeProgress={scrapeProgress} 
            scrapeMessage={scrapeMessage} 
            onStartScrape={startScrape}
            onCancelScrape={cancelScrape} 
            searchVal={searchVal} 
            setSearchVal={setSearchVal}
            sortVal={sortVal} 
            setSortVal={setSortVal}
            ratingFilter={ratingFilter}
            setRatingFilter={setRatingFilter}
          />
        )}
        
        {effectiveTab === "epic" && <EpicGamesPanel showToast={showToast} />}
        
        {effectiveTab === "steam" && <SteamGamesPanel showToast={showToast} />}
      </div>
    </>
  );
}
