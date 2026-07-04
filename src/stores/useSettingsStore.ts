import { create } from 'zustand';

interface SettingsState {
  activeTab: string;
  setActiveTab: (tab: string) => void;
}

export const useSettingsStore = create<SettingsState>((set) => ({
  activeTab: "general",
  setActiveTab: (tab) => set({ activeTab: tab }),
}));
