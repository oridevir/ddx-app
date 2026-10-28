// תוכנת חלונות לאבחנה מבדלת: חלון שנשאר מעל כל החלונות, צמוד לפינה השמאלית התחתונה,
// ומקש F9 שמעלים ומחזיר אותו. התוכן נטען מהאתר, ונשמר במחשב לעבודה בלי אינטרנט.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::menu::{CheckMenuItem, Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindow, WebviewWindowBuilder, WindowEvent};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

const URL: &str = "https://ddx.drdevir.com/?shell=1";
// הפעלה אוטומטית עם חלונות: פועלת כברירת מחדל. אם המשתמש ביטל אותה, נשמר קובץ סימון ולא מפעילים מחדש
const AUTOSTART_OFF: &str = "autostart-off";

fn autostart_marker(app: &AppHandle) -> Option<std::path::PathBuf> {
    app.path().app_data_dir().ok().map(|d| d.join(AUTOSTART_OFF))
}

fn place_bottom_left(w: &WebviewWindow) {
    let mon = w.current_monitor().ok().flatten().or_else(|| w.primary_monitor().ok().flatten());
    if let (Some(m), Ok(size)) = (mon, w.outer_size()) {
        let wa = m.work_area();
        let y = wa.position.y + wa.size.height as i32 - size.height as i32;
        let _ = w.set_position(PhysicalPosition::new(wa.position.x, y));
    }
}

fn toggle(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let shown = w.is_visible().unwrap_or(false) && !w.is_minimized().unwrap_or(false);
        if shown {
            let _ = w.hide();
        } else {
            let _ = w.unminimize();
            let _ = w.show();
            place_bottom_left(&w);
            let _ = w.set_focus();
        }
    }
}

fn main() {
    tauri::Builder::default()
        // פתיחה שנייה של התוכנה מציגה את החלון הקיים במקום לפתוח עוד אחד
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.unminimize();
                let _ = w.show();
                place_bottom_left(&w);
                let _ = w.set_focus();
            }
        }))
        // בהפעלה עם חלונות התוכנה עולה מוסתרת, ומופיעה ב-F9
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, Some(vec!["--hidden"])))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state() == ShortcutState::Pressed && shortcut.matches(Modifiers::empty(), Code::F9) {
                        toggle(app);
                    }
                })
                .build(),
        )
        .setup(|app| {
            let w = WebviewWindowBuilder::new(app, "main", WebviewUrl::External(URL.parse().unwrap()))
                .title("DDx")
                .always_on_top(true)
                .decorations(false)   // בלי שורת הכותרת האפורה של חלונות; גוררים מהכותרת האדומה
                .shadow(true)
                .resizable(true)
                .visible(false)
                .build()?;

            // שמינית מסך: רבע מהרוחב, חצי מהגובה
            if let Some(m) = w.primary_monitor()? {
                let wa = m.work_area();
                let _ = w.set_size(PhysicalSize::new(wa.size.width / 4, wa.size.height / 2));
            }
            place_bottom_left(&w);
            let hidden = std::env::args().any(|a| a == "--hidden");
            if !hidden {
                w.show()?;
            }

            // הפעלה עם חלונות: מפעילים בכל הרצה (כדי שהנתיב יתעדכן אם הקובץ הועבר), אלא אם בוטלה
            let marker = autostart_marker(app.handle());
            let opted_out = marker.as_ref().map(|m| m.exists()).unwrap_or(false);
            let autolaunch = app.autolaunch();
            if !opted_out {
                let _ = autolaunch.enable();
            }
            let auto_on = autolaunch.is_enabled().unwrap_or(false);

            app.global_shortcut().register(Shortcut::new(None, Code::F9))?;

            // אייקון ליד השעון: לחיצה מעלימה ומחזירה, ובתפריט יש יציאה
            let toggle_item = MenuItem::with_id(app, "toggle", "הצג / הסתר  (F9)", true, None::<&str>)?;
            let auto_item = CheckMenuItem::with_id(app, "autostart", "הפעלה עם חלונות", true, auto_on, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "יציאה", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&toggle_item, &auto_item, &quit_item])?;
            let auto_item_c = auto_item.clone();
            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("DDx · אבחנה מבדלת")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(move |app, e| match e.id.as_ref() {
                    "toggle" => toggle(app),
                    "autostart" => {
                        let al = app.autolaunch();
                        let marker = autostart_marker(app);
                        if al.is_enabled().unwrap_or(false) {
                            let _ = al.disable();
                            if let Some(m) = &marker {
                                if let Some(dir) = m.parent() { let _ = std::fs::create_dir_all(dir); }
                                let _ = std::fs::write(m, "");
                            }
                        } else {
                            let _ = al.enable();
                            if let Some(m) = &marker { let _ = std::fs::remove_file(m); }
                        }
                        let _ = auto_item_c.set_checked(al.is_enabled().unwrap_or(false));
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, e| {
                    if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = e {
                        toggle(tray.app_handle());
                    }
                })
                .build(app)?;
            Ok(())
        })
        // ✕ מעלים את החלון במקום לסגור את התוכנה, כדי ש-F9 ימשיך לעבוד
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running the app");
}
