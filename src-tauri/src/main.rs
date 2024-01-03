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
    settings: Arc<std::sync::RwLock<AppSettings>>,
}

#[tauri::command]
fn get_settings(state: tauri::State<'_, AppState>) -> Result<AppSettings, String> {
    let app_settings = state.settings.read().unwrap();
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
    let mut app_settings = state.settings.write().unwrap();
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

const DEFAULT_CONFIRM_CHARS: [&str; 7] = [" ", ".", ";", "!", "?", ":", ","];

fn default_settings() -> AppSettings {
    AppSettings {
        trigger: Trigger {
            string: "'".to_string(),
        },
        confirm: Confirm {
            chars: DEFAULT_CONFIRM_CHARS
                .iter()
                .map(|&s| s.to_string())
                .collect(),
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

struct CaptureSignal {
    sequence: String,
    append: String,
    append_enter: bool,
}

// Global static value that can be shared between threads.
// TODO: Maybe use `RwLock` here as well?
// fn signal() -> &'static Mutex<Vec<Signal>> {
//     static SIGNAL: OnceLock<Mutex<Vec<Signal>>> = OnceLock::new();

//     SIGNAL.get_or_init(|| Mutex::new(Vec::new()))
// }

fn main() {
    let default_app_settings = default_settings();

    let initial_active_window = get_active_window().unwrap();

    // Assuming the global variable is an integer
    let active_window: Arc<Mutex<ActiveWindow>> = Arc::new(Mutex::new(initial_active_window));

    let gv_clone = Arc::clone(&active_window);
    thread::spawn(move || loop {
        thread::sleep(Duration::from_millis(500));
        match get_active_window() {
            Ok(win) => {
                let mut global_var = gv_clone.lock().unwrap();

                if global_var.window_id != win.window_id {
                    println!("Window changed to: {}", win.app_name);
                }

                *global_var = win;
            }
            Err(_) => {
                println!("Error getting active window");
            }
        }
    });

    // Channel to communicate the captured sequence and possibly trigger an expansion.
    let (tx, rx) = std::sync::mpsc::channel::<CaptureSignal>();

    tauri::Builder::default()
        .manage(AppState {
            settings: Arc::new(std::sync::RwLock::new(default_app_settings)),
        })
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_process::init())
        .invoke_handler(tauri::generate_handler![
            get_settings,
            set_settings,
            get_default_settings,
            open_settings_dir,
        ])
        .setup(|app| {
            load_settings(app.app_handle());

            let app_handle = app.app_handle().clone();

            thread::spawn(move || {
                for received in rx {
                    println!("Got: {}", received.sequence);

                    let app_state = app_handle.state::<AppState>();
                    let app_settings = app_state.settings.read().unwrap();

                    end_capturing(
                        &received.sequence,
                        &app_settings.expansions,
                        &received.append,
                        received.append_enter,
                    );
                }
            });

            // Listen to input events needs to happen in another thread on windows,
            // otherwise the app crashes on startup with no visible error, but
            // due to a "access violation reading location" memory error.
            #[cfg(target_os = "windows")]
            {
                let app_handle_ = app.app_handle().clone();

                thread::spawn(move || {
                    handle_input(&app_handle_, tx);
                });
            }

            #[cfg(not(target_os = "windows"))]
            {
                handle_input(app.app_handle());
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
        let mut app_settings = app_state.settings.write().unwrap();
        *app_settings = new_settings;
    }
}

fn handle_input(app: &tauri::AppHandle, tx: std::sync::mpsc::Sender<CaptureSignal>) {
    // Set minimal delay if not on windows.
    #[cfg(not(target_os = "windows"))]
    {
        enigo.set_delay(0);
    }

    let mut current_sequence = String::new();
    let mut is_capturing = false;

    let app_handle_ = app.app_handle().clone();

    if let Err(error) = listen(move |event| {
        let app_state = app_handle_.state::<AppState>();
        let app_settings = app_state.settings.read().unwrap();
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

                tx.send(CaptureSignal {
                    sequence: current_sequence.clone(),
                    append: "".to_string(),
                    append_enter: event.event_type == EventType::KeyPress(Key::Return),
                })
                .unwrap();

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

                        // TODO: Maybe just call `end_capturing` directly?
                        // This should be fine since the original reason for the channel was to get rid of the humongus lag.
                        // But this was already fixed by using `RwLock` instead of `Mutex` for the app settings.
                        tx.send(CaptureSignal {
                            sequence: current_sequence.clone(),
                            append: if app_settings.confirm.append {
                                string.clone()
                            } else {
                                "".to_string()
                            },
                            append_enter: false,
                        })
                        .unwrap();

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
}

fn end_capturing(
    current_sequence: &String,
    expansions: &Vec<Expansion>,
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

    let mut enigo: Enigo = Enigo::new(&Settings::default()).unwrap();

    let char_count_to_remove =
        current_sequence.len() + append.len() + 1 + if append_enter { 1 } else { 0 };

    // Undo captured sequence.
    for _ in 0..char_count_to_remove {
        println!("Backspace");
        let r = enigo.key(enigo::Key::Backspace, enigo::Direction::Click);
        if r.is_err() {
            println!("Error: {:?}", r);
        }
    }

    // Wait for backspace to finish. This does not seem to be necessary on windows.
    #[cfg(not(target_os = "windows"))]
    {
        let count = (std::cmp::max(char_count_to_remove * 10 / 2, 50))
            .try_into()
            .unwrap();

        println!("Waiting for {} ms", count);
        std::thread::sleep(std::time::Duration::from_millis(count));
    }

    let full_text = format!("{}{}", text, append);
    enigo.text(full_text.as_str()).unwrap();

    if append_enter {
        enigo
            .key(enigo::Key::Return, enigo::Direction::Click)
            .unwrap();
    }
}
