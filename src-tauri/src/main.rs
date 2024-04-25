// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use app::App;
use serde_json::json;
use tauri::{CustomMenuItem, Manager, Menu, MenuItem, Submenu};
use tauri_plugin_store::StoreBuilder;
use std::sync::Mutex;
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
  let auto_scroll = CustomMenuItem::new("auto_scroll".to_string(), "自動スクロール");
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
    .add_item(vertical)
    .add_item(auto_scroll));
  let menu = Menu::new().add_submenu(file).add_submenu(view);
  tauri::Builder::default()
    .setup(|app| {
      let state = app.state() as tauri::State<Mutex<App>>;
      let mut state = state.lock().unwrap();
      let mut store = StoreBuilder::new(app.handle(), STORE_NAME.parse()?).build();
      match store.load() {
        Ok(_) => {
          println!("load ok");
          for i in 0..state.views.len() {
            state.views[i] = store.get(format!("view{}", i)).unwrap_or(&json!(state.views[i])).as_bool().unwrap();
            app.get_window("main").unwrap().menu_handle().get_item(&format!("view{}", i)).set_selected(state.views[i])?;
          }
          state.verbose = store.get("verbose").unwrap_or(&json!(state.verbose)).as_bool().unwrap();
          app.get_window("main").unwrap().menu_handle().get_item("verbose").set_selected(state.verbose)?;
          state.wrap = store.get("wrap").unwrap_or(&json!(state.wrap)).as_str().unwrap().to_string();
          app.get_window("main").unwrap().menu_handle().get_item("wrap").set_selected(state.wrap == "soft")?;
          state.vertical = store.get("vertical").unwrap_or(&json!(state.vertical)).as_bool().unwrap();
          app.get_window("main").unwrap().menu_handle().get_item("vertical").set_selected(state.vertical)?;
          state.auto_scroll = store.get("auto_scroll").unwrap_or(&json!(state.auto_scroll)).as_bool().unwrap();
          app.get_window("main").unwrap().menu_handle().get_item("auto_scroll").set_selected(state.auto_scroll)?;
        }
        _ => {
          println!("load ng");
          for i in 0..state.views.len() {
            store.insert(format!("view{}", i), json!(state.views[i]))?;
            app.get_window("main").unwrap().menu_handle().get_item(&format!("view{}", i)).set_selected(state.views[i])?;
          }
          store.insert("verbose".to_string(), json!(state.verbose))?;
          app.get_window("main").unwrap().menu_handle().get_item("verbose").set_selected(state.verbose)?;
          store.insert("wrap".to_string(), json!(state.wrap))?;
          app.get_window("main").unwrap().menu_handle().get_item("wrap").set_selected(state.wrap == "soft")?;
          store.insert("vertical".to_string(), json!(state.vertical))?;
          app.get_window("main").unwrap().menu_handle().get_item("vertical").set_selected(state.vertical)?;
          store.insert("auto_scroll".to_string(), json!(state.auto_scroll))?;
          app.get_window("main").unwrap().menu_handle().get_item("auto_scroll").set_selected(state.auto_scroll)?;
          store.save()?;
        }
      }
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
        "auto_scroll" => {
          app.auto_scroll = !app.auto_scroll;
          store.insert("auto_scroll".to_string(), json!(app.auto_scroll)).unwrap();
          store.save().unwrap();
          event.window().menu_handle().get_item(event.menu_item_id()).set_selected(app.auto_scroll).unwrap();
          event.window().emit_all("auto_scroll", app.auto_scroll).unwrap();
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
    .invoke_handler(tauri::generate_handler![read_log, get_views, get_states])
    .plugin(tauri_plugin_store::Builder::default().build())
    .run(tauri::generate_context!())
    .expect("error while running application");
}

#[tauri::command]
fn read_log(state: tauri::State<Mutex<App>>) -> Result<Vec<Vec<(String, String, String)>>, String> {
    let utc = Utc::now().naive_utc();
    let jst = Tokyo.from_utc_datetime(&utc);
    let path = format!("C:\\Nexon\\TalesWeaver\\ChatLog\\TWChatLog_{}_{:>02}_{:>02}.html", jst.year(), jst.month(), jst.day());
    let path = Path::new(&path);
    let result = state.lock().unwrap().read_log(path, utc);
    if let Err(error) = result {
      return Err(error.to_string());
    }
    let app = state.lock().unwrap();
    // test
    let mut msgs = Vec::new();
    for i in 0..7 {
      let mut msg = Vec::new();
      for j in 0..10 {
        msg.push(("これはテストメッセージですテストですので適当ですしあてになりません長さを稼ぐために何かを書いて言いますが関係ないです".to_string(), "".to_string(), "[ time ]".to_string()));
      }
      msgs.push(msg);
    }
    return Ok(msgs);
    Ok(app.messages.clone())
}

#[tauri::command]
fn get_views(state: tauri::State<Mutex<App>>) -> Result<Vec<bool>, String> {
  let state = state.lock().unwrap();
  Ok(state.views.clone())
}

#[tauri::command]
fn get_states(state: tauri::State<Mutex<App>>) -> Result<(bool, String, bool, bool), String> {
    let state = state.lock().unwrap();
    let verbose = state.verbose;
    let wrap = state.wrap.clone();
    let vertical = state.vertical;
    let auto_scroll = state.auto_scroll;
    Ok((verbose, wrap, vertical, auto_scroll))
}
