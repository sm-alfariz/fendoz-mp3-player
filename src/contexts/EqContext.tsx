import { createContext, useContext, useState, useEffect, useCallback, ReactNode } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Band, EqPreset } from '../types';

interface EqContextType {
  bands: Band[];
  presets: EqPreset[];
  activePreset: string;
  setBands: (bands: Band[]) => void;
  loadPreset: (name: string) => Promise<void>;
  reset: () => Promise<void>;
}

const EqContext = createContext<EqContextType | undefined>(undefined);

const DEFAULT_BANDS: Band[] = [
  { frequency: 31, gain: 0 },
  { frequency: 62, gain: 0 },
  { frequency: 125, gain: 0 },
  { frequency: 250, gain: 0 },
  { frequency: 500, gain: 0 },
  { frequency: 1000, gain: 0 },
  { frequency: 2000, gain: 0 },
  { frequency: 4000, gain: 0 },
  { frequency: 8000, gain: 0 },
  { frequency: 16000, gain: 0 },
];

export function EqProvider({ children }: { children: ReactNode }) {
  const [bands, setBandsState] = useState<Band[]>(DEFAULT_BANDS);
  const [presets, setPresets] = useState<EqPreset[]>([]);
  const [activePreset, setActivePreset] = useState('Flat');

  useEffect(() => {
    const loadInitial = async () => {
      const initialBands = await invoke<Band[]>('get_equalizer_bands');
      if (initialBands.length > 0) {
        setBandsState(initialBands);
      }

      const presetList = await invoke<EqPreset[]>('get_eq_presets');
      setPresets(presetList);
    };
    loadInitial();
  }, []);

  const setBands = useCallback(async (newBands: Band[]) => {
    setBandsState(newBands);
    await invoke('set_equalizer_bands', { bands: newBands });
    setActivePreset('Custom');
  }, []);

  const loadPreset = useCallback(async (name: string) => {
    await invoke('load_eq_preset', { name });
    const newBands = await invoke<Band[]>('get_equalizer_bands');
    setBandsState(newBands);
    setActivePreset(name);
  }, []);

  const reset = useCallback(async () => {
    await invoke('reset_equalizer');
    setBandsState(DEFAULT_BANDS);
    setActivePreset('Flat');
  }, []);

  return (
    <EqContext.Provider
      value={{
        bands,
        presets,
        activePreset,
        setBands,
        loadPreset,
        reset,
      }}
    >
      {children}
    </EqContext.Provider>
  );
}

export function useEq() {
  const context = useContext(EqContext);
  if (!context) {
    throw new Error('useEq must be used within an EqProvider');
  }
  return context;
}
