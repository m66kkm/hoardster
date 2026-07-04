import { create } from 'zustand';

type MenuMode = "home" | "local" | "radar" | "settings";

interface AppState {
  menuMode: MenuMode;
  setMenuMode: (mode: MenuMode) => void;
  toastMessage: string;
  showToast: (msg: string) => void;
}

export const useAppStore = create<AppState>((set) => ({
  menuMode: "home",
  setMenuMode: (mode) => set({ menuMode: mode }),
  toastMessage: "",
  showToast: (msg) => {
    set({ toastMessage: msg });
    setTimeout(() => set({ toastMessage: "" }), 2500);
  },
}));
