import { useTranslation } from "react-i18next";
import { Search, LayoutGrid, List } from "lucide-react";

interface FilterBarProps {
  activeTab: string;
  searchVal: string;
  setSearchVal: (val: string) => void;
  driveVal: string;
  setDriveVal: (val: string) => void;
  typeVal: string;
  setTypeVal: (val: string) => void;
  ratingVal: string;
  setRatingVal: (val: string) => void;
  sortVal: string;
  setSortVal: (val: string) => void;
  viewMode: "tile" | "detail";
  setViewMode: (mode: "tile" | "detail") => void;
  scanPaths: string[];
  genres?: string[];
  ratings?: string[];
}

export default function FilterBar({
  activeTab,
  searchVal,
  setSearchVal,
  driveVal,
  setDriveVal,
  typeVal,
  setTypeVal,
  ratingVal,
  setRatingVal,
  sortVal,
  setSortVal,
  viewMode,
  setViewMode,
  scanPaths,
  genres,
  ratings
}: FilterBarProps) {
  const { t } = useTranslation();

  if (activeTab === "dashboard" || activeTab === "settings") return null;

  return (
    <section className="controls-row">
      <div className="search-box">
        <Search className="search-icon" size={16} />
        <input
          type="text"
          value={searchVal}
          onChange={(e) => setSearchVal(e.target.value)}
          className="search-input"
          placeholder={t("filterSearch")}
        />
      </div>
      <select
        value={driveVal}
        onChange={(e) => setDriveVal(e.target.value)}
        className="filter-select"
      >
        <option value="">{t("filterAllDrives")}</option>
        {scanPaths.map((p) => (
          <option key={p} value={p}>{p}</option>
        ))}
      </select>
      <select
        value={typeVal}
        onChange={(e) => setTypeVal(e.target.value)}
        className="filter-select"
      >
        <option value="">所有游戏类型</option>
        {genres && genres.map((g) => (
          <option key={g} value={g}>{g}</option>
        ))}
      </select>
      <select
        value={ratingVal}
        onChange={(e) => setRatingVal(e.target.value)}
        className="filter-select"
      >
        <option value="">所有评价热度</option>
        {ratings && ratings.map((r) => (
          <option key={r} value={r}>{r}</option>
        ))}
      </select>
      
      {(activeTab === "posters" || activeTab === "installed" || activeTab === "all") && (
        <select
          value={sortVal}
          onChange={(e) => setSortVal(e.target.value)}
          className="filter-select"
        >
          <option value="">{t("filterSortDefault")}</option>
          <option value="name-asc">{t("filterSortNameAsc")}</option>
          <option value="name-desc">{t("filterSortNameDesc")}</option>
          <option value="steam-desc">{t("filterSortSteamDesc")}</option>
          <option value="steam-asc">{t("filterSortSteamAsc")}</option>
          <option value="size-desc">{t("filterSortSizeDesc")}</option>
          <option value="size-asc">{t("filterSortSizeAsc")}</option>
        </select>
      )}

      {(activeTab === "posters" || activeTab === "installed") && (
        <div className="view-toggle-group">
          <button
            className={`view-btn ${viewMode === "tile" ? "active" : ""}`}
            onClick={() => setViewMode("tile")}
          >
            <LayoutGrid size={14} />
            平铺
          </button>
          <button
            className={`view-btn ${viewMode === "detail" ? "active" : ""}`}
            onClick={() => setViewMode("detail")}
          >
            <List size={14} />
            详细
          </button>
        </div>
      )}
    </section>
  );
}
