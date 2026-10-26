mod battery;
mod device;

use std::time::Duration;
use tauri::Emitter;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let mut last: Vec<String> = Vec::new();
                // সব ডিভাইসের তথ্য পড়া গেছে কিনা (Trust না দিলে অসম্পূর্ণ থাকে)
                let mut complete = true;
                loop {
                    if let Ok(ids) = device::list_udids().await {
                        if ids != last || !complete {
                            if let Ok(list) = device::list_all().await {
                                complete = list.len() == ids.len();
                                let _ = handle.emit("devices-changed", &list);
                            }
                            last = ids;
                        }
                    }
                    tokio::time::sleep(Duration::from_secs(2)).await;
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            device::list_devices,
            battery::get_battery
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}