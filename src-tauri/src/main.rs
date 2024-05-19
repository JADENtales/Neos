// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use app::{App, ReadStatus};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::{Builder, CustomMenuItem, Manager, Menu, MenuItem, Submenu};
use tauri_plugin_store::StoreBuilder;
use std::{sync::Mutex, thread, time::Duration};
use chrono_tz::Asia::Tokyo;
use chrono::{Datelike, TimeZone, Utc};
use std::path::Path;

const STORE_NAME: &str = "store.dat";

fn main() {
  let exit = CustomMenuItem::new("exit".to_string(), "終了");
  let file = Submenu::new("ファイル", Menu::new().add_item(exit));
  let all = CustomMenuItem::new("view0".to_string(), "全体").accelerator("1");
  let public = CustomMenuItem::new("view1".to_string(), "一般").accelerator("2");
  let private = CustomMenuItem::new("view2".to_string(), "耳打ち").accelerator("3");
  let team = CustomMenuItem::new("view3".to_string(), "チーム").accelerator("4");
  let club = CustomMenuItem::new("view4".to_string(), "クラブ").accelerator("5");
  let system = CustomMenuItem::new("view5".to_string(), "システム").accelerator("6");
  let server = CustomMenuItem::new("view6".to_string(), "叫び").accelerator("7");
  let all_auto_scroll = CustomMenuItem::new("auto_scroll0".to_string(), "全体").accelerator("Ctrl+1");
  let public_auto_scroll = CustomMenuItem::new("auto_scroll1".to_string(), "一般").accelerator("Ctrl+2");
  let private_auto_scroll = CustomMenuItem::new("auto_scroll2".to_string(), "耳打ち").accelerator("Ctrl+3");
  let team_auto_scroll = CustomMenuItem::new("auto_scroll3".to_string(), "チーム").accelerator("Ctrl+4");
  let club_auto_scroll = CustomMenuItem::new("auto_scroll4".to_string(), "クラブ").accelerator("Ctrl+5");
  let system_auto_scroll = CustomMenuItem::new("auto_scroll5".to_string(), "システム").accelerator("Ctrl+6");
  let server_auto_scroll = CustomMenuItem::new("auto_scroll6".to_string(), "叫び").accelerator("Ctrl+7");
  let auto_scroll = Submenu::new("オートスクロール", Menu::new()
    .add_item(all_auto_scroll)
    .add_item(public_auto_scroll)
    .add_item(private_auto_scroll)
    .add_item(team_auto_scroll)
    .add_item(club_auto_scroll)
    .add_item(system_auto_scroll)
    .add_item(server_auto_scroll));
  let separator = MenuItem::Separator;
  let verbose = CustomMenuItem::new("verbose".to_string(), "時間表示").accelerator("T");
  let vertical = CustomMenuItem::new("vertical".to_string(), "縦分割").accelerator("D");
  let view = Submenu::new("表示", Menu::new()
    .add_item(all)
    .add_item(public)
    .add_item(private)
    .add_item(team)
    .add_item(club)
    .add_item(system)
    .add_item(server)
    .add_native_item(separator)
    .add_submenu(auto_scroll)
    .add_item(verbose)
    .add_item(vertical));
  let about = CustomMenuItem::new("about".to_string(), "Neosについて...");
  let help = Submenu::new("ヘルプ", Menu::new().add_item(about));
  let menu = Menu::new().add_submenu(file).add_submenu(view).add_submenu(help);
  Builder::default()
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
            app.get_window("main").unwrap().menu_handle().get_item(&format!("auto_scroll{}", i)).set_selected(state.auto_scroll[i])?;
          }
          state.verbose = store.get("verbose").unwrap_or(&json!(state.verbose)).as_bool().unwrap();
          app.get_window("main").unwrap().menu_handle().get_item("verbose").set_selected(state.verbose)?;
          state.vertical = store.get("vertical").unwrap_or(&json!(state.vertical)).as_bool().unwrap();
          app.get_window("main").unwrap().menu_handle().get_item("vertical").set_selected(state.vertical)?;
        }
        _ => {
          for i in 0..state.views.len() {
            store.insert(format!("view{}", i), json!(state.views[i]))?;
            app.get_window("main").unwrap().menu_handle().get_item(&format!("view{}", i)).set_selected(state.views[i])?;
            store.insert(format!("auto_scroll{}", i), json!(state.auto_scroll[i]))?;
            app.get_window("main").unwrap().menu_handle().get_item(&format!("auto_scroll{}", i)).set_selected(state.auto_scroll[i])?;
          }
          store.insert("verbose".to_string(), json!(state.verbose))?;
          app.get_window("main").unwrap().menu_handle().get_item("verbose").set_selected(state.verbose)?;
          store.insert("vertical".to_string(), json!(state.vertical))?;
          app.get_window("main").unwrap().menu_handle().get_item("vertical").set_selected(state.vertical)?;
          store.save()?;
        }
      }
      let app_handle = app.handle();
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
      for i in 0..state.views.len() {
        let app_handle = app.handle();
        let f = move |value| app_handle.get_window("main").unwrap().menu_handle().get_item(format!("view{}", i).as_str()).set_selected(value).unwrap();
        app.handle().listen_global(format!("view_back{}", i), move |event| {
          let payload = event.payload().unwrap() == "true";
          f(payload);
        });
        let app_handle = app.handle();
        let f = move |value| app_handle.get_window("main").unwrap().menu_handle().get_item(format!("auto_scroll{}", i).as_str()).set_selected(value).unwrap();
        app.handle().listen_global(format!("auto_scroll_back{}", i), move |event| {
          let payload = event.payload().unwrap() == "true";
          f(payload);
        });
      }
      let app_handle = app.handle();
      let f = move |value| app_handle.get_window("main").unwrap().menu_handle().get_item("verbose").set_selected(value).unwrap();
      app.handle().listen_global("verbose_back", move |event| {
          let payload = event.payload().unwrap() == "true";
          f(payload);
      });
      let app_handle = app.handle();
      let f = move |value| app_handle.get_window("main").unwrap().menu_handle().get_item("vertical").set_selected(value).unwrap();
      app.handle().listen_global("vertical_back", move |event| {
          let payload = event.payload().unwrap() == "true";
          f(payload);
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
        "vertical" => {
          app.vertical = !app.vertical;
          store.insert("vertical".to_string(), json!(app.vertical)).unwrap();
          store.save().unwrap();
          event.window().menu_handle().get_item(event.menu_item_id()).set_selected(app.vertical).unwrap();
          event.window().emit_all("vertical", app.vertical).unwrap();
        }
        "about" => event.window().emit_all("about", "").unwrap(),
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
      if id.starts_with("auto_scroll") {
        let i = String::from(&id[11..]).parse::<usize>().unwrap();
        app.auto_scroll[i] = !app.auto_scroll[i];
        store.insert(format!("auto_scroll{}", i), json!(app.auto_scroll[i])).unwrap();
        store.save().unwrap();
        event.window().menu_handle().get_item(event.menu_item_id()).set_selected(app.auto_scroll[i]).unwrap();
        event.window().emit_all(format!("auto_scroll{}", i).as_str(), app.auto_scroll[i]).unwrap();
      }
    })
    .manage(Mutex::new(App::new()))
    .invoke_handler(tauri::generate_handler![get_state, get_store_name])
    .plugin(tauri_plugin_store::Builder::default().build())
    .run(tauri::generate_context!())
    .expect("error while running application");
}

#[derive(Serialize, Deserialize)]
struct State {
  views: Vec<bool>,
  auto_scroll: Vec<bool>,
  verbose: bool,
  vertical: bool,
}

#[tauri::command]
fn get_state(state: tauri::State<Mutex<App>>) -> Result<State, String> {
    let state = state.lock().unwrap();
    let views = state.views.clone();
    let auto_scroll = state.auto_scroll.clone();
    let verbose = state.verbose;
    let vertical = state.vertical;
    Ok(State { views, auto_scroll, verbose, vertical })
}

#[tauri::command]
fn get_store_name() -> String {
  STORE_NAME.to_string()
}
