// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use app::{App, ReadStatus};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::{CustomMenuItem, Manager, Menu, MenuItem, Submenu};
use tauri_plugin_store::StoreBuilder;
use std::{sync::Mutex, thread, time::Duration};
use chrono_tz::Asia::Tokyo;
use chrono::{Datelike, TimeZone, Utc};
use std::path::Path;

const STORE_NAME: &str = "store.dat";

fn main() {
  let exit = CustomMenuItem::new("exit".to_string(), "終了");
  let file = Submenu::new("ファイル", Menu::new().add_item(exit));
  let all = CustomMenuItem::new("view0".to_string(), "全体");
  let public = CustomMenuItem::new("view1".to_string(), "一般");
  let private = CustomMenuItem::new("view2".to_string(), "耳打ち");
  let team = CustomMenuItem::new("view3".to_string(), "チーム");
  let club = CustomMenuItem::new("view4".to_string(), "クラブ");
  let system = CustomMenuItem::new("view5".to_string(), "システム");
  let server = CustomMenuItem::new("view6".to_string(), "叫び");
  let separator = MenuItem::Separator;
  let verbose = CustomMenuItem::new("verbose".to_string(), "時間表示");
  let wrap = CustomMenuItem::new("wrap".to_string(), "折り返し");
  let vertical = CustomMenuItem::new("vertical".to_string(), "縦分割");
  let view = Submenu::new("表示", Menu::new()
    .add_item(all)
    .add_item(public)
    .add_item(private)
    .add_item(team)
    .add_item(club)
    .add_item(system)
    .add_item(server)
    .add_native_item(separator)
    .add_item(verbose)
    .add_item(wrap)
    .add_item(vertical));
  let menu = Menu::new().add_submenu(file).add_submenu(view);
  tauri::Builder::default()
    .setup(|app| {
      let state = app.state() as tauri::State<Mutex<App>>;
      let mut state = state.lock().unwrap();
      let mut store = StoreBuilder::new(app.handle(), STORE_NAME.parse()?).build();
      match store.load() {
        Ok(_) => {
          for i in 0..state.views.len() {
            state.views[i] = store.get(format!("view{}", i)).unwrap_or(&json!(state.views[i])).as_bool().unwrap();
            app.get_window("main").unwrap().menu_handle().get_item(&format!("view{}", i)).set_selected(state.views[i])?;
            state.auto_scroll[i] = store.get(format!("auto_scroll{}", i)).unwrap_or(&json!(state.auto_scroll[i])).as_bool().unwrap();
          }
          state.verbose = store.get("verbose").unwrap_or(&json!(state.verbose)).as_bool().unwrap();
          app.get_window("main").unwrap().menu_handle().get_item("verbose").set_selected(state.verbose)?;
          state.wrap = store.get("wrap").unwrap_or(&json!(state.wrap)).as_str().unwrap().to_string();
          app.get_window("main").unwrap().menu_handle().get_item("wrap").set_selected(state.wrap == "soft")?;
          state.vertical = store.get("vertical").unwrap_or(&json!(state.vertical)).as_bool().unwrap();
          app.get_window("main").unwrap().menu_handle().get_item("vertical").set_selected(state.vertical)?;
        }
        _ => {
          for i in 0..state.views.len() {
            store.insert(format!("view{}", i), json!(state.views[i]))?;
            app.get_window("main").unwrap().menu_handle().get_item(&format!("view{}", i)).set_selected(state.views[i])?;
            store.insert(format!("auto_scroll{}", i), json!(state.auto_scroll[i]))?;
          }
          store.insert("verbose".to_string(), json!(state.verbose))?;
          app.get_window("main").unwrap().menu_handle().get_item("verbose").set_selected(state.verbose)?;
          store.insert("wrap".to_string(), json!(state.wrap))?;
          app.get_window("main").unwrap().menu_handle().get_item("wrap").set_selected(state.wrap == "soft")?;
          store.insert("vertical".to_string(), json!(state.vertical))?;
          app.get_window("main").unwrap().menu_handle().get_item("vertical").set_selected(state.vertical)?;
          store.save()?;
        }
      }
      let app_handle = app.app_handle();
      thread::spawn(move || {
        loop {
          thread::sleep(Duration::from_millis(500));
          let utc = Utc::now().naive_utc();
          let jst = Tokyo.from_utc_datetime(&utc);
          let path = format!("C:\\Nexon\\TalesWeaver\\ChatLog\\TWChatLog_{}_{:>02}_{:>02}.html", jst.year(), jst.month(), jst.day());
          let path = Path::new(&path);
          let state = app_handle.state() as tauri::State<Mutex<App>>;
          let result = state.lock().unwrap().read_log(path, utc).unwrap();
          if let ReadStatus::Unchanged = result {
            continue;
          }
          let app = state.lock().unwrap();
          app_handle.emit_all("read", app.messages.clone()).unwrap();
        }
      });
      Ok(())
    })
    .menu(menu)
    .on_menu_event(|event| {
      let state = event.window().state() as tauri::State<Mutex<App>>;
      let mut app = state.lock().unwrap();
      let mut store = StoreBuilder::new(event.window().app_handle(), STORE_NAME.parse().unwrap()).build();
      store.load().unwrap();
      match event.menu_item_id() {
        "exit" => event.window().close().unwrap(),
        "verbose" => {
          app.verbose = !app.verbose;
          store.insert("verbose".to_string(), json!(app.verbose)).unwrap();
          store.save().unwrap();
          event.window().menu_handle().get_item(event.menu_item_id()).set_selected(app.verbose).unwrap();
          event.window().emit_all("verbose", app.verbose).unwrap();
        }
        "wrap" => {
          app.wrap = if app.wrap == "soft" { String::from("off") } else { String::from("soft") };
          store.insert("wrap".to_string(), json!(app.wrap)).unwrap();
          store.save().unwrap();
          event.window().menu_handle().get_item(event.menu_item_id()).set_selected(app.wrap == "soft").unwrap();
          event.window().emit_all("wrap", app.wrap.as_str()).unwrap();
        }
        "vertical" => {
          app.vertical = !app.vertical;
          store.insert("vertical".to_string(), json!(app.vertical)).unwrap();
          store.save().unwrap();
          event.window().menu_handle().get_item(event.menu_item_id()).set_selected(app.vertical).unwrap();
          event.window().emit_all("vertical", app.vertical).unwrap();
        }
        _ => ()
      }
      let id = event.menu_item_id();
      if id.starts_with("view") {
        let i = String::from(&id[4..]).parse::<usize>().unwrap();
        app.views[i] = !app.views[i];
        store.insert(format!("view{}", i), json!(app.views[i])).unwrap();
        store.save().unwrap();
        event.window().menu_handle().get_item(event.menu_item_id()).set_selected(app.views[i]).unwrap();
        event.window().emit_all(format!("view{}", i).as_str(), app.views[i]).unwrap();
      }
    })
    .manage(Mutex::new(App::new()))
    .invoke_handler(tauri::generate_handler![get_views, get_state])
    .plugin(tauri_plugin_store::Builder::default().build())
    .run(tauri::generate_context!())
    .expect("error while running application");
}

#[tauri::command]
fn get_views(state: tauri::State<Mutex<App>>) -> Result<Vec<bool>, String> {
  let state = state.lock().unwrap();
  Ok(state.views.clone())
}

#[derive(Serialize, Deserialize)]
struct State {
  verbose: bool,
  wrap: String,
  vertical: bool,
  auto_scroll: Vec<bool>,
}

#[tauri::command]
fn get_state(state: tauri::State<Mutex<App>>) -> Result<State, String> {
    let state = state.lock().unwrap();
    let verbose = state.verbose;
    let wrap = state.wrap.clone();
    let vertical = state.vertical;
    let auto_scroll = state.auto_scroll.clone();
    Ok(State { verbose, wrap, vertical, auto_scroll })
}
