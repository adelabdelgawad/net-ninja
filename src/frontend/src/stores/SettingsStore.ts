// Settings store - hardcoded to dark theme
// UI preference settings have been removed from the application
import { createRoot, createSignal } from 'solid-js';

export type ThemeMode = 'system' | 'light' | 'dark';
export type ColorTheme = 'oscura-midnight' | 'default-blue';

export interface ThemeSettings {
  mode: ThemeMode;
  colorTheme: ColorTheme;
}

export type InitState = 'uninitialized' | 'loading' | 'ready' | 'error';

interface SettingsStore {
  // State getters
  initState: () => InitState;
  themeSettings: () => ThemeSettings | null;
  resolvedTheme: () => 'light' | 'dark';

  // Actions
  load: () => Promise<void>;
  updateTheme: (updates: Partial<ThemeSettings>) => void;
  setThemeMode: (mode: ThemeMode) => void;
  setColorTheme: (colorTheme: ColorTheme) => void;
}

// Hardcoded to dark theme
const HARDODED_THEME: ThemeSettings = {
  mode: 'dark',
  colorTheme: 'oscura-midnight',
};

// Factory function that creates store within a root
function createSettingsStore(): SettingsStore {
  const store = createRoot(() => {
    // All reactive primitives created INSIDE root
    const [initState, setInitState] = createSignal<InitState>('uninitialized');

    // Apply dark theme immediately
    const root = document.documentElement;
    root.classList.remove('light');
    root.classList.add('dark');
    root.setAttribute('data-color-theme', 'oscura-midnight');

    const store: SettingsStore = {
      initState,
      themeSettings: () => HARDODED_THEME,
      resolvedTheme: () => 'dark',

      async load() {
        if (initState() !== 'uninitialized') return;
        setInitState('loading');

        try {
          // Theme is hardcoded to dark mode
          setInitState('ready');
        } catch (e) {
          console.error('Failed to load settings:', e);
          setInitState('error');
          throw e;
        }
      },

      // No-op methods for backward compatibility
      // Theme is hardcoded, these methods do nothing
      updateTheme: (_updates: Partial<ThemeSettings>) => {
        // No-op: theme is hardcoded
      },
      setThemeMode: (_mode: ThemeMode) => {
        // No-op: theme is hardcoded
      },
      setColorTheme: (_colorTheme: ColorTheme) => {
        // No-op: theme is hardcoded
      },
    };

    return store;
  });

  return store;
}

// Singleton instance
let storeInstance: SettingsStore | null = null;

export function getSettingsStore(): SettingsStore {
  if (!storeInstance) {
    storeInstance = createSettingsStore();
  }
  return storeInstance;
}

// Helper to get current theme settings without store (for non-reactive code)
export function getThemeSettings() {
  return HARDODED_THEME;
}
