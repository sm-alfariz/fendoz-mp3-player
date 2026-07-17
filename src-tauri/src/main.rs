// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Configure ALSA for larger buffers to prevent underruns
    std::env::set_var("ALSA_PERIOD_SIZE", "1024");
    std::env::set_var("ALSA_BUFFER_SIZE", "4096");

    mp3_player_lib::run()
}
