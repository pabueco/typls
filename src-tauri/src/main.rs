// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use active_win_pos_rs::{get_active_window, ActiveWindow};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use rdev::listen;
use enigo::{Enigo, Settings, Keyboard};

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn main() {

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

    // Send keys.

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![greet])
        .setup(|_app| {
            // tauri::async_runtime::spawn(async move {

                // if let Err(error) = listen(callback) {
                //     println!("Error: {:?}", error)
                // }

                let mut enigo = Enigo::new(&Settings::default()).unwrap();
                enigo.set_delay(1);

                let mut current_sequence = String::new();
                let mut is_capturing = false;

                const SEQUENCE_END_CHARS : [&str; 7] =  [" ", ".", ";", "!", "?", ":", ","];

                if let Err(error) = listen(move |event| match event.name {
                    // EventType::KeyRelease(Key::Alt) => {
                    //     println!("alt pressed");
                    // },
                    // _ => (),
                    Some(string) => {
                        // println!("User wrote {:?}", string);

                        if string == "'" {
                            println!("start");
                            current_sequence = String::new();
                            is_capturing = true;
                        } 
                        // TODO: Make tab work? string == "\t"
                        else if is_capturing && SEQUENCE_END_CHARS.contains(&string.as_str())  {
                            println!("end");
                            is_capturing = false;

                            // Undo captured sequence.
                            for _ in 0..(current_sequence.len() + 2)  {
                                let r = enigo.key(enigo::Key::Backspace, enigo::Direction::Click);
                                if r.is_err() {
                                    println!("Error: {:?}", r);
                                }
                            }

                            let full_text = format!("Hello World ❤️{}", string);

                            enigo.text(full_text.as_str()).unwrap();

                            current_sequence = String::new();
                        } else if is_capturing {
                            current_sequence.push_str(&string);
                                println!("current_sequence: {}", current_sequence);
                        }
                    },
                    None => (),
                }) {
                    println!("Error: {:?}", error)
                }
                // .expect("error");
            // });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
