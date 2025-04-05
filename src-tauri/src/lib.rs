use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;

use enigo::{Enigo, Keyboard, Settings};
use rdev::{listen, EventType, Key};

use serde::{Deserialize, Serialize};
use tauri::Manager;

use active_win_pos_rs::{get_active_window, ActiveWindow};
use std::time::Duration;

use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
struct AppSettings {
    trigger: TriggerSettings,
    confirm: ConfirmSettings,
    variables: VariableSettings,
    expansions: Vec<Expansion>,
    groups: Option<Vec<Group>>,
    active_group: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
struct TriggerSettings {
    string: String,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
struct ConfirmSettings {
    chars: Vec<String>,
    key_enter: bool,
    key_right_arrow: bool,
    append: bool,
    auto: bool,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
struct VariableSettings {
    separator: String,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
struct Expansion {
    #[serde(default = "generate_uuid")]
    id: String,
    abbr: String,
    text: String,
    group: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
struct Group {
    id: String,
    name: String,
    apps: Vec<App>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
struct App {
    path: String,
    os: String,
}

struct AppState {
    settings: Arc<std::sync::RwLock<AppSettings>>,
}

#[cfg(dev)]
const SETTINGS_FILE_NAME: &str = "test.json";

#[cfg(not(dev))]
const SETTINGS_FILE_NAME: &str = "settings.json";

fn generate_uuid() -> String {
    Uuid::new_v4().to_string()
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
    let setting_file_path = app_config_dir.join(SETTINGS_FILE_NAME);

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
        trigger: TriggerSettings {
            string: "'".to_string(),
        },
        confirm: ConfirmSettings {
            chars: DEFAULT_CONFIRM_CHARS
                .iter()
                .map(|&s| s.to_string())
                .collect(),
            key_enter: true,
            key_right_arrow: true,
            append: true,
            auto: false,
        },
        variables: VariableSettings {
            separator: "|".to_string(),
        },
        expansions: vec![Expansion {
            id: "typls".to_string(),
            abbr: "typls".to_string(),
            text: "Type less with typls: https://typls.app".to_string(),
            group: None,
        }],
        groups: Some(vec![]),
        active_group: None,
    }
}

struct CaptureSignal {
    sequence: String,
    append: String,
    append_enter: bool,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let default_app_settings = default_settings();

    let initial_active_window = get_active_window().unwrap();

    // Get current active window and poll for changes.
    let active_window: Arc<Mutex<ActiveWindow>> = Arc::new(Mutex::new(initial_active_window));
    let gv_clone = Arc::clone(&active_window);
    thread::spawn(move || loop {
        thread::sleep(Duration::from_millis(500));
        match get_active_window() {
            Ok(win) => {
                let mut global_var = gv_clone.lock().unwrap();

                if global_var.window_id != win.window_id {
                    println!("Active window: {:?}", win);
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
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            settings: Arc::new(std::sync::RwLock::new(default_app_settings)),
        })
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            get_settings,
            set_settings,
            get_default_settings,
            open_settings_dir,
        ])
        .setup(|app| {
            load_settings(&app.app_handle());

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
                        &app_settings.variables.separator,
                        &active_window,
                        &app_settings,
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
                handle_input(app.app_handle(), tx);
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
    let mut current_sequence = String::new();
    let mut is_capturing = false;

    let app_handle_ = app.clone();

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

                let is_return_key = event.event_type == EventType::KeyPress(Key::Return);

                tx.send(CaptureSignal {
                    sequence: current_sequence.clone(),
                    append: "".to_string(),
                    append_enter: is_return_key,
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

                        if app_settings.confirm.auto {
                            let matching_expansion = app_settings
                                .expansions
                                .iter()
                                .find(|&e| e.abbr == current_sequence);

                            let expansions_starting_with_sequence = app_settings
                                .expansions
                                .iter()
                                .filter(|&e| e.abbr.starts_with(&current_sequence))
                                .count();

                            if matching_expansion.is_some()
                                && expansions_starting_with_sequence == 1
                            {
                                let loose_variable_regex =
                                    regex::Regex::new(r"\{[^\s}]*\}").unwrap();
                                let matching_has_no_variables = !loose_variable_regex
                                    .is_match(&matching_expansion.unwrap().text);

                                if matching_has_no_variables {
                                    println!("Auto confirm");
                                    tx.send(CaptureSignal {
                                        sequence: current_sequence.clone(),
                                        append: "".to_string(),
                                        append_enter: false,
                                    })
                                    .unwrap();

                                    is_capturing = false;
                                    current_sequence = String::new();
                                    return;
                                }
                            }
                        }
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
    variable_separator: &str,
    active_window: &Arc<Mutex<ActiveWindow>>,
    app_settings: &AppSettings,
) {
    println!("End capturing, {}", current_sequence);

    let parts = current_sequence.split(variable_separator);

    // Extract abbreviation (first element).
    let abbr = parts.clone().next().unwrap();

    // Find all matching expansions.
    let mut matching_expansions = expansions
        .iter()
        .filter(|&e| e.abbr == abbr.to_string())
        .collect::<Vec<_>>();

    if matching_expansions.is_empty() {
        return;
    }

    // Sort expansions so the ones assigned to a group are placed first.
    // This is done so we always check constrained expansions first.
    matching_expansions.sort_by_key(|e| e.group.is_none());

    let mut chosen_expansion = None;

    println!("active_group: {:?}", app_settings.active_group);
    println!("matching_expansions : {:?}", matching_expansions);

    // Select first expansion when active group is set
    if let Some(active_group) = &app_settings.active_group {
        println!("active_group: {:?}", active_group);

        let active_group_expansions = matching_expansions
            .iter()
            .filter(|&e| e.group.is_none() || e.group == Some(active_group.clone()))
            .collect::<Vec<_>>();

        println!("active_group_expansions: {:?}", active_group_expansions);

        if !active_group_expansions.is_empty() {
            chosen_expansion = Some(active_group_expansions[0]);
        }
    }

    if chosen_expansion.is_none() {
        // Find expansion by group matching the active window/app.
        for exp in matching_expansions.iter() {
            if let Some(group_id) = &exp.group {
                let window_props = active_window.lock().unwrap();
                let process_path = &window_props.process_path;

                println!(
                    "Expansion has group {}, current window path is {}",
                    group_id,
                    process_path.display()
                );

                // find group in app_settings that has matching id
                let group = app_settings
                    .groups
                    .as_ref()
                    .unwrap()
                    .iter()
                    .find(|&g| g.id == *group_id);

                if group.is_none() {
                    continue;
                }

                let platform = tauri_plugin_os::platform();

                if group.unwrap().apps.iter().any(|app| {
                    app.os == platform && app.path == process_path.to_string_lossy().to_string()
                }) {
                    chosen_expansion = Some(exp);
                    break;
                }
            } else {
                chosen_expansion = Some(exp);
            }
        }
    }

    // Parse variables in expansion text.
    let mut variables = parse_variables(chosen_expansion.unwrap());
    variables.unnamed.reverse();

    let mut params_unnamed: Vec<&str> = vec![];
    let mut params_named: HashMap<String, &str> = HashMap::new();

    // Extract parameters into named and unnamed.
    for part in parts.skip(1) {
        let pair: Vec<&str> = part.split("=").collect();
        let (key, value) = if pair.len() == 2 {
            (pair[0], pair[1])
        } else {
            ("", pair[0])
        };

        if key.is_empty() {
            params_unnamed.push(value);
        } else {
            params_named.insert(key.to_string(), value);
        }
    }

    // Reverse unnamed parameters to replace them in the correct order.
    params_unnamed.reverse();

    let mut text = chosen_expansion.unwrap().text.clone();

    // Replace unnamed variables ({}, {=default}) in text with provided or default values.
    for variable in variables.unnamed.iter_mut() {
        // Create replace pattern like {} or {=default} and replace first occurrence.
        let pattern = if variable.default.is_empty() {
            "{}".to_string()
        } else {
            format!("{{={}}}", variable.default)
        };

        match params_unnamed.pop() {
            Some(value) => {
                text = text.replacen(&pattern, value, 1);
            }
            None => {
                // Replace with default value if no value was provided.
                text = text.replacen(&pattern, &variable.default, 1);
            }
        }
    }

    // Replace named variables ({key}, {key=default}) in text with provided or default values.
    for (key, variable) in variables.named.iter_mut() {
        // Create replace pattern like {key} or {key=default} and replace first occurrence.
        let pattern = if variable.default.is_empty() {
            format!("{{{}}}", variable.name)
        } else {
            format!("{{{0}={1}}}", variable.name, variable.default)
        };

        match params_named.remove(key) {
            Some(value) => {
                text = text.replace(&pattern, value);
            }
            None => {
                // Replace with default value if no value was provided.
                text = text.replace(&pattern, &variable.default);
            }
        }
    }

    let mut enigo: Enigo = Enigo::new(&Settings::default()).unwrap();

    // Set minimal delay if not on windows.
    // #[cfg(not(target_os = "windows"))]
    // {
    //     enigo.set_delay(0);
    // }

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

#[derive(Debug)]
struct Variable {
    name: String,
    default: String,
}

#[derive(Debug)]
struct ExpansionVariables {
    named: HashMap<String, Variable>,
    unnamed: Vec<Variable>,
}

fn parse_variables(expansion: &Expansion) -> ExpansionVariables {
    let mut named = HashMap::new();
    let mut unnamed = Vec::new();

    let mut chars = expansion.text.chars();

    while let Some(c) = chars.next() {
        if c == '{' {
            let mut name = String::new();
            let mut default = String::new();

            while let Some(c) = chars.next() {
                if c == '}' || c.is_whitespace() {
                    break;
                }

                if c == '=' {
                    while let Some(c) = chars.next() {
                        if c == '}' {
                            break;
                        }

                        default.push(c);
                    }
                    break;
                }

                name.push(c);
            }

            if name.is_empty() {
                unnamed.push(Variable {
                    name: "".to_string(),
                    default,
                });
            } else {
                named.insert(name.clone(), Variable { name, default });
            }
        }
    }

    ExpansionVariables { named, unnamed }
}
