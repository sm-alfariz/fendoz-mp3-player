import React from 'react';
import { useEq } from '../contexts/EqContext';

export function Equalizer() {
  const { bands, presets, activePreset, setBands, loadPreset, reset } = useEq();

  const handleBandChange = (index: number, value: number) => {
    const newBands = [...bands];
    newBands[index] = { ...newBands[index], gain: value };
    setBands(newBands);
  };

  const formatFrequency = (freq: number) => {
    if (freq >= 1000) {
      return `${freq / 1000}k`;
    }
    return freq.toString();
  };

  return (
    <div className="equalizer">
      <div className="eq-header">
        <h3>Equalizer</h3>
        <div className="eq-controls">
          <select
            value={activePreset}
            onChange={(e) => loadPreset(e.target.value)}
            className="preset-select"
          >
            {presets.map((preset) => (
              <option key={preset.name} value={preset.name}>
                {preset.name}
              </option>
            ))}
          </select>
          <button className="reset-btn" onClick={reset}>
            Reset
          </button>
        </div>
      </div>

      <div className="eq-bands">
        {bands.map((band, index) => (
          <div key={band.frequency} className="eq-band">
            <div className="band-value">{band.gain.toFixed(1)}</div>
            <input
              type="range"
              min="-12"
              max="12"
              step="0.5"
              value={band.gain}
              onChange={(e) => handleBandChange(index, parseFloat(e.target.value))}
              className="band-slider"
              orient="vertical"
            />
            <div className="band-label">{formatFrequency(band.frequency)}</div>
          </div>
        ))}
      </div>
    </div>
  );
}
