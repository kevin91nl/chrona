import { createContext, useContext, useState, useEffect, useCallback, type ReactNode } from "react";
import { getSetting, setSetting } from "@/settings";

const STORAGE_KEY = "filter_enabled_providers";

interface FilterContextValue {
  enabledProviders: Set<string>;
  allProviders: string[];
  toggleProvider: (id: string) => void;
  setAllProviders: (ids: string[]) => void;
  isProviderEnabled: (id: string) => boolean;
  resetFilters: () => void;
  filtersActive: boolean;
}

const FilterContext = createContext<FilterContextValue | null>(null);

// Known provider IDs — used as fallback before discovery runs
const KNOWN_PROVIDERS = ["cron", "launchd", "codex", "systemd", "windows-task"];

export function FilterProvider({ children }: { children: ReactNode }) {
  const [enabledProviders, setEnabledProviders] = useState<Set<string>>(new Set());
  const [loaded, setLoaded] = useState(false);

  // Load persisted filter on mount
  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const stored = await getSetting(STORAGE_KEY);
        if (cancelled) return;
        if (stored) {
          const ids: string[] = JSON.parse(stored);
          setEnabledProviders(new Set(ids));
        } else {
          // Default: all providers enabled
          setEnabledProviders(new Set(KNOWN_PROVIDERS));
        }
      } catch {
        setEnabledProviders(new Set(KNOWN_PROVIDERS));
      } finally {
        if (!cancelled) setLoaded(true);
      }
    })();
    return () => { cancelled = true; };
  }, []);

  // Persist on change (debounced)
  useEffect(() => {
    if (!loaded) return;
    const timer = setTimeout(() => {
      setSetting(STORAGE_KEY, JSON.stringify([...enabledProviders])).catch(() => {});
    }, 300);
    return () => clearTimeout(timer);
  }, [enabledProviders, loaded]);

  const toggleProvider = useCallback((id: string) => {
    setEnabledProviders((prev) => {
      const next = new Set(prev);
      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
      }
      return next;
    });
  }, []);

  const setAllProviders = useCallback((ids: string[]) => {
    setEnabledProviders(new Set(ids));
  }, []);

  const isProviderEnabled = useCallback(
    (id: string) => enabledProviders.has(id),
    [enabledProviders],
  );

  const resetFilters = useCallback(() => {
    setEnabledProviders(new Set(KNOWN_PROVIDERS));
  }, []);

  // filtersActive is true when not all known providers are enabled
  const filtersActive = enabledProviders.size < KNOWN_PROVIDERS.length;

  return (
    <FilterContext.Provider
      value={{
        enabledProviders,
        allProviders: KNOWN_PROVIDERS,
        toggleProvider,
        setAllProviders,
        isProviderEnabled,
        resetFilters,
        filtersActive,
      }}
    >
      {children}
    </FilterContext.Provider>
  );
}

export function useFilters(): FilterContextValue {
  const ctx = useContext(FilterContext);
  if (!ctx) throw new Error("useFilters must be used within FilterProvider");
  return ctx;
}
