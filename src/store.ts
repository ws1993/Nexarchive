import { create } from "zustand";
import { api } from "./api";
import { defaultConfig, type AppConfig } from "./types";

interface AppState {
  config: AppConfig;
  loadingConfig: boolean;
  savingConfig: boolean;
  lastRunJobId?: string;
  setConfig: (config: AppConfig) => void;
  refreshConfig: () => Promise<void>;
  saveConfig: () => Promise<boolean>;
  runJobNow: () => Promise<string | undefined>;
}

export const useAppStore = create<AppState>((set, get) => ({
  config: defaultConfig,
  loadingConfig: false,
  savingConfig: false,
  lastRunJobId: undefined,

  setConfig: (config) => set({ config }),

  refreshConfig: async () => {
    set({ loadingConfig: true });
    try {
      const cfg = await api.loadSettings();
      set({ config: cfg });
    } finally {
      set({ loadingConfig: false });
    }
  },

  saveConfig: async () => {
    set({ savingConfig: true });
    try {
      return await api.saveSettings(get().config);
    } finally {
      set({ savingConfig: false });
    }
  },

  runJobNow: async () => {
    const jobId = await api.runJobOnce();
    set({ lastRunJobId: jobId });
    return jobId;
  }
}));
