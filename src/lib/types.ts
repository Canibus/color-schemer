// src/lib/types.ts
export type DisplaySettings = {
  brightness: number;
  contrast: number;
  gamma: number;
  digital_vibrance: number;
};

export type DisplayInfo = {
    id: string;
    name: string;
    is_primary: boolean;
};

export type DisplayProfile = {
  name: string;
  description: string;
  settings: DisplaySettings;
  target_displays: string[];
};

export type HotkeyConfig = {
    next_profile: string;
    prev_profile: string;
    reset: string;
};

export type AppConfig = {
    language: string;
    hotkeys: HotkeyConfig;
    show_notifications: boolean;
    start_minimized: boolean;
    auto_start: boolean;
    profiles: DisplayProfile[];
};
