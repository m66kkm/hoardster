import { create } from 'zustand';

interface LocalGamesState {
  activeTab: string;
  setActiveTab: (tab: string) => void;
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
  currentPage: number;
  setCurrentPage: (page: number | ((prev: number) => number)) => void;
  viewMode: "tile" | "detail";
  setViewMode: (mode: "tile" | "detail") => void;
  resetFilters: () => void;
}

export const useLocalGamesStore = create<LocalGamesState>((set) => ({
  activeTab: "all",
  setActiveTab: (tab) => set({ activeTab: tab }),
  searchVal: "",
  setSearchVal: (val) => set({ searchVal: val }),
  driveVal: "",
  setDriveVal: (val) => set({ driveVal: val }),
  typeVal: "",
  setTypeVal: (val) => set({ typeVal: val }),
  ratingVal: "",
  setRatingVal: (val) => set({ ratingVal: val }),
  sortVal: "",
  setSortVal: (val) => set({ sortVal: val }),
  currentPage: 1,
  setCurrentPage: (page) => set((state) => ({ currentPage: typeof page === "function" ? page(state.currentPage) : page })),
  viewMode: "tile",
  setViewMode: (mode) => set({ viewMode: mode }),
  resetFilters: () =>
    set({
      searchVal: "",
      driveVal: "",
      typeVal: "",
      ratingVal: "",
      sortVal: "",
      currentPage: 1,
    }),
}));
