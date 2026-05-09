// src/lib/types.ts
export type DisplaySettings = {
  brightness: number;
  contrast: number;
  gamma: number;
  digital_vibrance: number;
};

export type DisplayProfile = {
  name: string;
  description: string;
  settings: DisplaySettings;
};

export type HotkeyConfig = {
    next_profile: string;
    prev_profile: string;
    reset: string;
};

export type AppConfig = {
    hotkeys: HotkeyConfig;
    show_notifications: boolean;
    start_minimized: boolean;
    profiles: DisplayProfile[];
};
