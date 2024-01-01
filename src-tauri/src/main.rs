// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use active_win_pos_rs::{get_active_window, ActiveWindow};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::Manager;

use enigo::{Enigo, Keyboard, Settings};
use rdev::{listen, EventType, Key};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
struct AppSettings {
    trigger: Trigger,
    confirm: Confirm,
    expansions: Vec<Expansion>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
struct Trigger {
    string: String,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
struct Confirm {
    chars: Vec<String>,
    key_enter: bool,
    key_right_arrow: bool,
    append: bool,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
struct Expansion {
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

#[tauri::command]
fn get_default_settings() -> Result<AppSettings, String> {
    Ok(default_settings())
}

#[tauri::command]
fn open_settings_dir(app: tauri::AppHandle) {
    let app_config_dir = get_settings_directory_path(&app);

    println!("Opening settings dir: {:?}", app_config_dir);

    if !app_config_dir.exists() {
        println!("Settings dir does not exist");
    }

    if let Err(error) = open::that(app_config_dir) {
        println!("Error opening settings dir: {:?}", error);
    }
}

const CONFIRM_CHARS: [&str; 7] = [" ", ".", ";", "!", "?", ":", ","];

fn default_settings() -> AppSettings {
    AppSettings {
        trigger: Trigger {
            string: "'".to_string(),
        },
        confirm: Confirm {
            chars: CONFIRM_CHARS.iter().map(|&s| s.to_string()).collect(),
            key_enter: true,
            key_right_arrow: true,
            append: true,
        },
        expansions: vec![Expansion {
            abbr: "typls".to_string(),
            text: "Type less with typls -> https://typls.app".to_string(),
        }],
    }
}

fn main() {
    let default_app_settings = default_settings();

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
        .invoke_handler(tauri::generate_handler![
            get_settings,
            set_settings,
            get_default_settings,
            open_settings_dir
        ])
        .setup(|app| {
            load_settings(app.app_handle());

            // let app_state = app.state::<AppState>();
            // let app_settings: std::sync::MutexGuard<'_, AppSettings> =
            //     app_state.settings.lock().unwrap();
            // let trigger_char = app_settings.trigger_char.clone();
            // let expansions = app_settings.expansions.clone();

            let mut enigo = Enigo::new(&Settings::default()).unwrap();

            // Set minimal delay if on supported platform.
            if cfg!(not(windows)) {
                enigo.set_delay(0);
            }

            fn end_capturing(
                current_sequence: &String,
                expansions: &Vec<Expansion>,
                enigo: &mut Enigo,
                append: &str,
                append_enter: bool,
            ) {
                println!("End capturing, {}", current_sequence);

                let parts = current_sequence.split("|");
                let abbr = parts.clone().next().unwrap();

                // Find matching expansion
                let matching_expansion = expansions.iter().find(|&e| e.abbr == abbr.to_string());

                // Return if no matching expansion found.
                if matching_expansion.is_none() {
                    return;
                }

                let mut text = matching_expansion.unwrap().text.clone();

                let mut variable_map = HashMap::new();

                for part in parts.skip(1) {
                    let pair: Vec<&str> = part.split("=").collect();
                    let (key, value) = if pair.len() == 2 {
                        (pair[0], pair[1])
                    } else {
                        ("", pair[0])
                    };
                    variable_map.insert(key, value);

                    if key.is_empty() {
                        text = text.replacen("{}", value, 1);
                    } else {
                        text = text.replace(&format!("{{{}}}", key), value);
                    }
                }

                println!("text: {}", text);

                let char_count_to_remove =
                    current_sequence.len() + append.len() + 1 + if append_enter { 1 } else { 0 };

                // Undo captured sequence.
                for _ in 0..char_count_to_remove {
                    let r = enigo.key(enigo::Key::Backspace, enigo::Direction::Click);
                    if r.is_err() {
                        println!("Error: {:?}", r);
                    }
                }

                let full_text = format!("{}{}", text, append);

                // Wait for backspace to finish.
                // TODO: Maybe make this depend on the length of the text?
                std::thread::sleep(std::time::Duration::from_millis(
                    (std::cmp::max(char_count_to_remove * 10 / 2, 50))
                        .try_into()
                        .unwrap(),
                ));

                println!("full_text: {}", full_text.as_str());

                enigo.text(full_text.as_str()).unwrap();

                if append_enter {
                    enigo
                        .key(enigo::Key::Return, enigo::Direction::Click)
                        .unwrap();
                }
            }

            let mut current_sequence = String::new();
            let mut is_capturing = false;

            let app_handle_ = app.app_handle().clone();

            if let Err(error) = listen(move |event| {
                let app_state = app_handle_.state::<AppState>();
                let app_settings = app_state.settings.lock().unwrap();

                let mut return_early = false;

                if app_settings.trigger.string.is_empty() {
                    return;
                }

                match event.event_type {
                    // Confirm capture without appending anything.
                    EventType::KeyPress(Key::RightArrow) | EventType::KeyPress(Key::Return) => {
                        if !is_capturing {
                            return;
                        }

                        // Return if confirm via the pressed key is disabled.
                        if (!app_settings.confirm.key_right_arrow
                            && event.event_type == EventType::KeyPress(Key::RightArrow))
                            || (!app_settings.confirm.key_enter
                                && event.event_type == EventType::KeyPress(Key::Return))
                        {
                            return;
                        }

                        end_capturing(
                            &current_sequence,
                            &app_settings.expansions,
                            &mut enigo,
                            "",
                            if app_settings.confirm.append {
                                event.event_type == EventType::KeyPress(Key::Return)
                            } else {
                                false
                            },
                        );
                        is_capturing = false;
                        current_sequence = String::new();
                        return_early = true;
                    }
                    // Cancel capture.
                    EventType::KeyPress(Key::Escape) => {
                        if !is_capturing {
                            return;
                        }
                        is_capturing = false;
                        current_sequence = String::new();
                        return_early = true;
                    }
                    EventType::KeyPress(Key::Backspace) => {
                        if !is_capturing {
                            return;
                        }

                        if current_sequence.is_empty() {
                            is_capturing = false;
                        } else {
                            current_sequence.pop();
                        }

                        return_early = true;
                    }
                    _ => (),
                }

                if return_early {
                    return;
                }

                match event.name {
                    Some(string) => {
                        if string.is_empty() {
                            return;
                        }

                        if is_capturing {
                            // TODO: Make tab work? string == '\t'
                            if app_settings.confirm.chars.contains(&string) {
                                println!("End capturing, {}", current_sequence);
                                end_capturing(
                                    &current_sequence,
                                    &app_settings.expansions,
                                    &mut enigo,
                                    if app_settings.confirm.append {
                                        &string
                                    } else {
                                        ""
                                    },
                                    false,
                                );

                                is_capturing = false;
                                current_sequence = String::new();
                            } else {
                                current_sequence.push_str(&string);
                                println!("current_sequence: {}", current_sequence);
                            }
                        } else {
                            if string == app_settings.trigger.string {
                                println!("Start capturing");
                                current_sequence = String::new();
                                is_capturing = true;
                            }
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

fn get_settings_directory_path(app: &tauri::AppHandle) -> std::path::PathBuf {
    let app_config_dir = app.path().app_config_dir().unwrap();
    app_config_dir
}

const SETTINGS_FILE_NAME: &str = "settings.json";

fn load_settings(app: &tauri::AppHandle) {
    // Load settings
    let app_config_dir = get_settings_directory_path(app);

    if !app_config_dir.exists() {
        std::fs::create_dir_all(&app_config_dir).unwrap();
    }
    let setting_file_path = app_config_dir.join(SETTINGS_FILE_NAME);

    println!("Loading settings from: {:?}", setting_file_path);

    // Ensure settings file exists.
    if setting_file_path.exists() {
        let settings_json = std::fs::read_to_string(setting_file_path).unwrap();

        // parse settings and catch errors
        let new_settings: AppSettings = match serde_json::from_str(&settings_json) {
            Ok(settings) => settings,
            Err(error) => {
                println!("Error parsing settings: {:?}", error);
                AppSettings::default()
            }
        };

        // Write new settings into app state.
        let app_state = app.state::<AppState>();
        let mut app_settings: std::sync::MutexGuard<'_, AppSettings> =
            app_state.settings.lock().unwrap();
        *app_settings = new_settings;
    }
}
