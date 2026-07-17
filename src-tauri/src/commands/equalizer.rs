use tauri::State;
use std::sync::{Arc, Mutex};
use crate::audio::dsp::Equalizer;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Band {
    pub frequency: f32,
    pub gain: f32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EqPreset {
    pub name: String,
    pub gains: Vec<f32>,
}

pub struct EqState(pub Arc<Mutex<Equalizer>>);

fn get_builtin_presets() -> Vec<EqPreset> {
    vec![
        EqPreset {
            name: "Flat".to_string(),
            gains: vec![0.0; 10],
        },
        EqPreset {
            name: "Pop".to_string(),
            gains: vec![-2.0, 0.0, 2.0, 4.0, 3.0, 0.0, -1.0, -2.0, -2.0, -3.0],
        },
        EqPreset {
            name: "Rock".to_string(),
            gains: vec![4.0, 3.0, 2.0, 0.0, -2.0, -1.0, 2.0, 3.0, 4.0, 4.0],
        },
        EqPreset {
            name: "Jazz".to_string(),
            gains: vec![3.0, 2.0, 1.0, 2.0, -2.0, -2.0, 0.0, 2.0, 3.0, 4.0],
        },
        EqPreset {
            name: "Bass Booster".to_string(),
            gains: vec![6.0, 5.0, 4.0, 3.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        },
    ]
}

#[tauri::command]
pub fn get_equalizer_bands(eq: State<'_, EqState>) -> Vec<Band> {
    let eq = eq.0.lock().unwrap();
    let gains = eq.get_gains();
    let frequencies = [31.0, 62.0, 125.0, 250.0, 500.0, 1000.0, 2000.0, 4000.0, 8000.0, 16000.0];

    frequencies.iter().zip(gains.iter())
        .map(|(&freq, &gain)| Band { frequency: freq, gain })
        .collect()
}

#[tauri::command]
pub fn set_equalizer_bands(eq: State<'_, EqState>, bands: Vec<Band>) {
    let mut eq = eq.0.lock().unwrap();
    for (i, band) in bands.iter().enumerate() {
        eq.set_gain(i, band.gain);
    }
}

#[tauri::command]
pub fn get_eq_presets() -> Vec<EqPreset> {
    get_builtin_presets()
}

#[tauri::command]
pub fn load_eq_preset(eq: State<'_, EqState>, name: String) -> Result<(), String> {
    let presets = get_builtin_presets();
    let preset = presets.iter()
        .find(|p| p.name == name)
        .ok_or_else(|| format!("Preset not found: {}", name))?;

    let mut eq = eq.0.lock().unwrap();
    for (i, &gain) in preset.gains.iter().enumerate() {
        eq.set_gain(i, gain);
    }
    Ok(())
}

#[tauri::command]
pub fn reset_equalizer(eq: State<'_, EqState>) {
    let mut eq = eq.0.lock().unwrap();
    eq.reset();
}
