// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use active_win_pos_rs::{get_active_window, ActiveWindow};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::Manager;

use enigo::{Enigo, Keyboard, Settings};
use rdev::{listen, EventType, Key};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[derive(Clone)]
struct AppSettings {
    trigger_char: String,
    expanders: Vec<Expander>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
#[derive(Clone)]
struct Expander {
    abbr: String,
    text: String,
}

struct AppState {
    settings: Mutex<AppSettings>,
}

#[tauri::command]
fn get_settings(state: tauri::State<'_, AppState>) -> Result<AppSettings, String> {
    let app_settings = state.settings.lock().unwrap();
    print!("get_settings: {:?}", app_settings);
    Ok(app_settings.clone())
}

#[tauri::command]
fn set_settings(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    settings: AppSettings,
) -> Result<(), String> {
    print!("save_settings: {:?}", settings);
    let mut app_settings: std::sync::MutexGuard<'_, AppSettings> = state.settings.lock().unwrap();
    *app_settings = settings;

    let app_config_dir = app.path().app_config_dir().unwrap();
    let setting_file_path = app_config_dir.join("settings.json");

    let default_settings_json: String = serde_json::to_string_pretty(&*app_settings).unwrap();

    println!("Settings file path: {:?}", setting_file_path);
    std::fs::write(setting_file_path, default_settings_json).unwrap();

    Ok(())
}

fn main() {
    let default_app_settings = AppSettings {
        trigger_char: String::from("'"),
        expanders: vec![],
    };

    let initial_active_window = get_active_window().unwrap();

    // Assuming the global variable is an integer
    let active_window: Arc<Mutex<ActiveWindow>> = Arc::new(Mutex::new(initial_active_window));

    let gv_clone = Arc::clone(&active_window);
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(500));
            match get_active_window() {
                Ok(win) => {
                    let mut global_var = gv_clone.lock().unwrap();

                    if global_var.window_id != win.window_id {
                        println!("Window changed to: {}", win.app_name);
                    }

                    *global_var = win; // Update the global variable
                }
                Err(_) => {
                    println!("Error getting active window");
                }
            }
        }
    });

    tauri::Builder::default()
        .manage(AppState {
            settings: Mutex::new(default_app_settings),
        })
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_process::init())
        .invoke_handler(tauri::generate_handler![get_settings, set_settings])
        .setup(|app| {
            load_settings(app.app_handle());

            let app_state = app.state::<AppState>();
            let app_settings: std::sync::MutexGuard<'_, AppSettings> =
                app_state.settings.lock().unwrap();

            let mut enigo = Enigo::new(&Settings::default()).unwrap();
            enigo.set_delay(1);

            let mut current_sequence = String::new();
            let mut is_capturing = false;

            const SEQUENCE_END_CHARS: [&str; 7] = [" ", ".", ";", "!", "?", ":", ","];

            let trigger_char = app_settings.trigger_char.clone();
            let expanders = app_settings.expanders.clone();

            fn end_capturing(
                current_sequence: &String,
                expanders: &Vec<Expander>,
                enigo: &mut Enigo,
                append: &str,
            ) {
                println!("End capturing, {}", current_sequence);

                // Find matching expander
                let matching_expander = expanders
                    .iter()
                    .find(|&e| e.abbr == current_sequence.to_string());

                // Return if no matching expander found.
                if matching_expander.is_none() {
                    return;
                }

                // Undo captured sequence.
                for _ in 0..(current_sequence.len() + 2) {
                    let r = enigo.key(enigo::Key::Backspace, enigo::Direction::Click);
                    if r.is_err() {
                        println!("Error: {:?}", r);
                    }
                }

                let full_text = format!("{}{}", matching_expander.unwrap().text, append);

                enigo.text(full_text.as_str()).unwrap();
            }

            if let Err(error) = listen(move |event| {
                match event.event_type {
                    EventType::KeyPress(Key::RightArrow) => {
                        if !is_capturing {
                            return;
                        }
                        end_capturing(&current_sequence, &expanders, &mut enigo, "");
                        is_capturing = false;
                        current_sequence = String::new();
                    }
                    _ => (),
                }
                match event.name {
                    Some(string) => {
                        if string == trigger_char {
                            println!("Start capturing");
                            current_sequence = String::new();
                            is_capturing = true;
                        }
                        // TODO: Make tab work? string == "\t"
                        else if is_capturing && SEQUENCE_END_CHARS.contains(&string.as_str()) {
                            end_capturing(&current_sequence, &expanders, &mut enigo, &string);
                            is_capturing = false;
                            current_sequence = String::new();
                        } else if is_capturing {
                            current_sequence.push_str(&string);
                            println!("current_sequence: {}", current_sequence);
                        }
                    }
                    None => (),
                }
            }) {
                println!("Error: {:?}", error)
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn load_settings(app: &tauri::AppHandle) {
    // Load settings
    let app_config_dir = app.path().app_config_dir().unwrap();

    if !app_config_dir.exists() {
        std::fs::create_dir_all(&app_config_dir).unwrap();
    }
    let setting_file_path = app_config_dir.join("settings.json");

    println!("Loading settings from: {:?}", setting_file_path);

    // Ensure settings file exists.
    if setting_file_path.exists() {
        let settings_json = std::fs::read_to_string(setting_file_path).unwrap();
        let new_settings: AppSettings = serde_json::from_str(&settings_json).unwrap();

        // Write new settings into app state.
        let app_state = app.state::<AppState>();
        let mut app_settings: std::sync::MutexGuard<'_, AppSettings> =
            app_state.settings.lock().unwrap();
        *app_settings = new_settings;
    }
}
