import { create } from 'zustand';

interface RadarState {
  activeTab: string;
  setActiveTab: (tab: string) => void;
  searchVal: string;
  setSearchVal: (val: string) => void;
  sortVal: string;
  setSortVal: (val: string) => void;
  currentPage: number;
  setCurrentPage: (page: number) => void;
}

export const useRadarStore = create<RadarState>((set) => ({
  activeTab: "news",
  setActiveTab: (tab) => set({ activeTab: tab }),
  searchVal: "",
  setSearchVal: (val) => set({ searchVal: val }),
  sortVal: "",
  setSortVal: (val) => set({ sortVal: val }),
  currentPage: 1,
  setCurrentPage: (page) => set({ currentPage: page }),
}));
