import { Radio, Download, Terminal, Gift, Gamepad2 } from "lucide-react";

import TabNav, { type TabDef } from "../components/TabNav";
import NewsPanel from "../components/NewsPanel";
import Torrents1337Panel from "../components/Torrents1337Panel";
import SRPanel from "../components/SRPanel";

import { useRadarStore } from "../stores/useRadarStore";
import { useAppStore } from "../stores/useAppStore";
import { useScrape } from "../hooks/useScrape";
import EpicGamesPanel from "../components/EpicGamesPanel";
import SteamGamesPanel from "../components/SteamGamesPanel";

export default function RadarPage() {
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
    onComplete: (msg) => {
      if (msg) showToast(msg);
    }
  });

  const tabs: TabDef[] = [
    { id: "news", icon: Radio, labelKey: "tabNews" },
    { id: "epic", icon: Gift, labelKey: "tabEpic" },
    { id: "steam", icon: Gamepad2, labelKey: "tabSteam" },
    { id: "torrents1337", icon: Download, labelKey: "tab1337" },
    { id: "sr", icon: Terminal, labelKey: "tabSR" }
  ];





  return (
    <>
      <TabNav 
        tabs={tabs} 
        activeTab={activeTab} 
        onTabChange={(id) => setActiveTab(id)} 
      />

      <div className="tab-content-scrollable">
        {activeTab === "news" && <NewsPanel />}
        
        {activeTab === "epic" && <EpicGamesPanel showToast={showToast} />}
        
        {activeTab === "steam" && <SteamGamesPanel showToast={showToast} />}
        
        {activeTab === "torrents1337" && (
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
          />
        )}
        
        {activeTab === "sr" && <SRPanel showToast={showToast} />}
      </div>
    </>
  );
}
