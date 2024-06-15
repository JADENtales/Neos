use std::{fs::{self, File}, io::{Read, Seek, SeekFrom}, path::Path};
use regex::Regex;
use chrono_tz::Asia::Tokyo;
use chrono::{DateTime, Datelike, Duration, NaiveDateTime, TimeZone, Timelike, Utc};
use anyhow::{bail, Result};

#[derive(Debug)]
pub struct App {
  pub views: Vec<bool>,
  pub exp: bool,
  pub auto_scroll: Vec<bool>,
  pub verbose: bool,
  pub vertical: bool,
  pub limit: bool,
  pub messages: Vec<(Vec<(String, String, String)>, bool)>,
  file: Option<File>,
  file_size: u64,
  date: NaiveDateTime,
}

pub enum ReadStatus {
  Ok,
  Updated,
  Unchanged,
}

impl App {
  pub fn new() -> Self {
    App {
      views: vec![true; 7],
      exp: false,
      verbose: false,
      vertical: true,
      auto_scroll: vec![true; 7],
      limit: true,
      messages: vec![(Vec::new(), false); 7],
      file: None,
      file_size: 0,
      date: Utc::now().naive_utc(),
    }
  }

  pub fn read_log(&mut self, path: &Path, date: NaiveDateTime) -> Result<ReadStatus> {
    if !path.is_file() {
      return Ok(ReadStatus::Ok);
    }
    let now = Tokyo.from_utc_datetime(&date);
    let past = Tokyo.from_utc_datetime(&self.date);
    self.date = date;
    if let None = self.file {
      self.file = Some(File::open(path)?);
    } else if past.day() != now.day() {
      drop(self.file.take());
      self.file = Some(File::open(path)?);
    }

    let file_size = fs::metadata(path)?.len();  
    if self.file_size == 0 {
      self.file_size = file_size;
      self.file.as_ref().unwrap().seek(SeekFrom::Start(file_size))?;
      return Ok(ReadStatus::Ok);
    }
    if self.file_size == file_size && past.day() == now.day() {
      return Ok(ReadStatus::Unchanged);
    }
    let buf_size = if past.day() == now.day() {
      file_size - self.file_size
    } else {
      file_size
    };
    self.file_size = file_size;
    let mut content = vec![0; buf_size as usize];
    self.file.as_ref().unwrap().read(&mut content)?;
    let (cow, _, _) = encoding_rs::SHIFT_JIS.decode(&content);
    let message = cow.into_owned();

    let mut messages: Vec<_> = message.split("\r\n").filter(|e| e.trim() != "").collect();
    if past.day() != now.day() {
      for _ in 0..4 {
        messages.remove(0);
      }
    }
    for i in 0..self.messages.len() {
      self.messages[i].1 = false;
    }
    let regex = Regex::new(r##"^<font.+> (.+) </font> <font.+color="(.+)">(.+)</font></br>$"##).unwrap();
    for message in messages {
      match regex.captures(&message) {
        Some(captures) => {
          let time = &captures[1];
          let color = &captures[2];
          let message = &captures[3].replace("&nbsp", " ");
          let i = match color {
            "#c8ffc8" | "#ffffff" => 1,
            "#64ff64" => 2,
            "#f7b73c" => 3,
            "#94ddfa" => 4,
            "#ff64ff" | "#ff6464" | "#64ff80" => 5,
            "#c896c8" => 6,
            _ => bail!("invalid captured color.: {} {} {}", color, time, message),
          };
          self.messages[0].0.push(((*message).clone(), color.to_string(), time.to_string()));
          self.messages[0].1 = true;
          self.messages[i].0.push(((*message).clone(), color.to_string(), time.to_string()));
          self.messages[i].1 = true;
        }
        _ => bail!("regex does not match.: {}", message),
      }
    }
    Ok(ReadStatus::Updated)
  }

  pub fn get_messages(&self) -> Vec<(Vec<(String, String, String)>, bool)> {
    if self.limit {
      let mut messages = vec![(Vec::new(), false); 7];
      for i in 0..self.messages.len() {
        if 500 < self.messages[i].0.len() {
          messages[i].0 = (&(self.messages[i].0)[self.messages[i].0.len() - 500..self.messages[i].0.len()]).to_vec();
        } else {
          messages[i].0 = self.messages[i].0.clone();
        }
        messages[i].1 = self.messages[i].1;
      }
      messages
    } else {
      self.messages.clone()
    }
  }

  pub fn calc_exp(&self, now: NaiveDateTime) -> (i64, i64, i64) {
    let mut total_exp = 0 as i64;
    let span = 3;
    for i in (0..self.messages[5].0.len()).rev() {
      let message = &self.messages[5].0[i];
      let regex = Regex::new(r##"経験値が (\d+) 上がりました。"##).unwrap();
      match regex.captures(&message.0) {
        Some(captures) => {
          let regex = Regex::new(r##"\[\s?(\d+)時\s+(\d+)分\s+(\d+)秒\]"##).unwrap();
          match regex.captures(&message.2) {
            Some(caps) => {
              let hour = &caps[1];
              let minute = &caps[2];
              let second = &caps[3];
              let now = Tokyo.from_utc_datetime(&now);
              let now = DateTime::parse_from_str(format!("{}{:0>2}{:0>2}{:0>2}{:0>2}{:0>2} +0900", now.year(), now.month(), now.day(), now.hour(), now.minute(), now.second()).as_str(), "%Y%m%d%H%M%S %z").unwrap();
              let time = DateTime::parse_from_str(format!("{}{:0>2}{:0>2}{:0>2}{:0>2}{:0>2} +0900", now.year(), now.month(), now.day(), hour, minute, second).as_str(), "%Y%m%d%H%M%S %z").unwrap();
              let end = now - Duration::seconds(span);
              let time = if now.day() != end.day() && time.hour() == 23 {
                time - Duration::days(1)
              } else {
                time
              };
              if end <= time {
                let exp = (&captures[1]).parse::<i64>().unwrap();
                total_exp += exp;
              } else {
                break;
              }
            }
            _ => (),
          }
        }
        _ => (),
      }
    }
    let exp_per_second = total_exp / span;
    let exp_per_minute = exp_per_second * 60;
    let exp_per_hour = exp_per_minute * 60;
    (exp_per_second, exp_per_minute, exp_per_hour)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn read_file_larger_past_file() {
    let mut app = App::new();
    let path ="test\\TWChatLog_2024_04_20.html";
    let path = Path::new(path);
    app.file = Some(File::open(path).unwrap());
    app.file_size = std::fs::metadata("test\\TWChatLog_2024_04_19_large.html").unwrap().len();
    app.date = NaiveDateTime::parse_from_str("2024/04/19 00:00:00", "%Y/%m/%d %H:%M:%S").unwrap();
    let date = NaiveDateTime::parse_from_str("2024/04/20 00:00:00", "%Y/%m/%d %H:%M:%S").unwrap();
    app.read_log(path, date).unwrap();
    assert_eq!(app.messages,
      [
        (
          vec![
            (String::from("◇本日の毎日課題：ステッドを退治"), String::from("#ff64ff"), String::from("[ 0時  0分  0秒]")),
            (String::from("◇本日の毎日課題：アビス深層96階以上クリア"), String::from("#ff64ff"), String::from("[ 0時  0分  0秒]")),
            (String::from("◇本日の毎日課題：ピリ辛ナテスコ煮"), String::from("#ff64ff"), String::from("[ 0時  0分  0秒]")),
            (String::from("?フォレスト?が1分後に「ルーンの庭園」を訪問します。"), String::from("#ff64ff"), String::from("[ 0時  0分  0秒]")),
            (String::from("ルーンの庭園へ訪れると、妖精や精霊たちが嬉しそうに迎えてくれます。"), String::from("#ff64ff"), String::from("[ 0時  0分  0秒]")),
            (String::from("プシーキーの迷宮が再設定されました。"), String::from("#ff64ff"), String::from("[ 0時  0分  1秒]")),
            (String::from("プラバ防衛戦が始まりました。"), String::from("#ff64ff"), String::from("[ 0時  0分  1秒]")),
            (String::from("[チーム経験値アップイベント] 始まりました！"), String::from("#ff64ff"), String::from("[ 0時  0分  1秒]")),
            (String::from("ランダムレイドバトルに参加できます。[ クラド ]でポータルを利用して入場してくださ"), String::from("#ff64ff"), String::from("[ 0時  0分  2秒]")),
            (String::from("い。"), String::from("#ff64ff"), String::from("[ 0時  0分  2秒]")),
            (String::from("[チーム経験値アップイベント] 実施中です！"), String::from("#ff64ff"), String::from("[ 0時  0分 59秒]")),
          ],
          true,
        ),
        (vec![], false),
        (vec![], false),
        (vec![], false),
        (vec![], false),
        (
          vec![
            (String::from("◇本日の毎日課題：ステッドを退治"), String::from("#ff64ff"), String::from("[ 0時  0分  0秒]")),
            (String::from("◇本日の毎日課題：アビス深層96階以上クリア"), String::from("#ff64ff"), String::from("[ 0時  0分  0秒]")),
            (String::from("◇本日の毎日課題：ピリ辛ナテスコ煮"), String::from("#ff64ff"), String::from("[ 0時  0分  0秒]")),
            (String::from("?フォレスト?が1分後に「ルーンの庭園」を訪問します。"), String::from("#ff64ff"), String::from("[ 0時  0分  0秒]")),
            (String::from("ルーンの庭園へ訪れると、妖精や精霊たちが嬉しそうに迎えてくれます。"), String::from("#ff64ff"), String::from("[ 0時  0分  0秒]")),
            (String::from("プシーキーの迷宮が再設定されました。"), String::from("#ff64ff"), String::from("[ 0時  0分  1秒]")),
            (String::from("プラバ防衛戦が始まりました。"), String::from("#ff64ff"), String::from("[ 0時  0分  1秒]")),
            (String::from("[チーム経験値アップイベント] 始まりました！"), String::from("#ff64ff"), String::from("[ 0時  0分  1秒]")),
            (String::from("ランダムレイドバトルに参加できます。[ クラド ]でポータルを利用して入場してくださ"), String::from("#ff64ff"), String::from("[ 0時  0分  2秒]")),
            (String::from("い。"), String::from("#ff64ff"), String::from("[ 0時  0分  2秒]")),
            (String::from("[チーム経験値アップイベント] 実施中です！"), String::from("#ff64ff"), String::from("[ 0時  0分 59秒]")),
          ],
          true,
        ),
        (vec![], false),
      ]
    );
  }

  #[test]
  fn read_file_smaller_past_file() {
    let mut app = App::new();
    let path ="test\\TWChatLog_2024_04_20.html";
    let path = Path::new(path);
    app.file = Some(File::open(path).unwrap());
    app.file_size = std::fs::metadata("test\\TWChatLog_2024_04_19_small.html").unwrap().len();
    app.date = NaiveDateTime::parse_from_str("2024/04/19 00:00:00", "%Y/%m/%d %H:%M:%S").unwrap();
    let date = NaiveDateTime::parse_from_str("2024/04/20 00:00:00", "%Y/%m/%d %H:%M:%S").unwrap();
    app.read_log(path, date).unwrap();
    assert_eq!(app.messages,
      [
        (
          vec![
            (String::from("◇本日の毎日課題：ステッドを退治"), String::from("#ff64ff"), String::from("[ 0時  0分  0秒]")),
            (String::from("◇本日の毎日課題：アビス深層96階以上クリア"), String::from("#ff64ff"), String::from("[ 0時  0分  0秒]")),
            (String::from("◇本日の毎日課題：ピリ辛ナテスコ煮"), String::from("#ff64ff"), String::from("[ 0時  0分  0秒]")),
            (String::from("?フォレスト?が1分後に「ルーンの庭園」を訪問します。"), String::from("#ff64ff"), String::from("[ 0時  0分  0秒]")),
            (String::from("ルーンの庭園へ訪れると、妖精や精霊たちが嬉しそうに迎えてくれます。"), String::from("#ff64ff"), String::from("[ 0時  0分  0秒]")),
            (String::from("プシーキーの迷宮が再設定されました。"), String::from("#ff64ff"), String::from("[ 0時  0分  1秒]")),
            (String::from("プラバ防衛戦が始まりました。"), String::from("#ff64ff"), String::from("[ 0時  0分  1秒]")),
            (String::from("[チーム経験値アップイベント] 始まりました！"), String::from("#ff64ff"), String::from("[ 0時  0分  1秒]")),
            (String::from("ランダムレイドバトルに参加できます。[ クラド ]でポータルを利用して入場してくださ"), String::from("#ff64ff"), String::from("[ 0時  0分  2秒]")),
            (String::from("い。"), String::from("#ff64ff"), String::from("[ 0時  0分  2秒]")),
            (String::from("[チーム経験値アップイベント] 実施中です！"), String::from("#ff64ff"), String::from("[ 0時  0分 59秒]")),
          ],
          true,
        ),
        (vec![], false),
        (vec![], false),
        (vec![], false),
        (vec![], false),
        (
          vec![
            (String::from("◇本日の毎日課題：ステッドを退治"), String::from("#ff64ff"), String::from("[ 0時  0分  0秒]")),
            (String::from("◇本日の毎日課題：アビス深層96階以上クリア"), String::from("#ff64ff"), String::from("[ 0時  0分  0秒]")),
            (String::from("◇本日の毎日課題：ピリ辛ナテスコ煮"), String::from("#ff64ff"), String::from("[ 0時  0分  0秒]")),
            (String::from("?フォレスト?が1分後に「ルーンの庭園」を訪問します。"), String::from("#ff64ff"), String::from("[ 0時  0分  0秒]")),
            (String::from("ルーンの庭園へ訪れると、妖精や精霊たちが嬉しそうに迎えてくれます。"), String::from("#ff64ff"), String::from("[ 0時  0分  0秒]")),
            (String::from("プシーキーの迷宮が再設定されました。"), String::from("#ff64ff"), String::from("[ 0時  0分  1秒]")),
            (String::from("プラバ防衛戦が始まりました。"), String::from("#ff64ff"), String::from("[ 0時  0分  1秒]")),
            (String::from("[チーム経験値アップイベント] 始まりました！"), String::from("#ff64ff"), String::from("[ 0時  0分  1秒]")),
            (String::from("ランダムレイドバトルに参加できます。[ クラド ]でポータルを利用して入場してくださ"), String::from("#ff64ff"), String::from("[ 0時  0分  2秒]")),
            (String::from("い。"), String::from("#ff64ff"), String::from("[ 0時  0分  2秒]")),
            (String::from("[チーム経験値アップイベント] 実施中です！"), String::from("#ff64ff"), String::from("[ 0時  0分 59秒]")),
          ],
          true,
        ),
        (vec![], false),
      ]
    );
  }

  #[test]
  fn read_file_no_data() {
    let mut app = App::new();
    let path ="test\\TWChatLog_2024_04_20_no_data.html";
    let path = Path::new(path);
    app.file = Some(File::open(path).unwrap());
    app.file_size = std::fs::metadata("test\\TWChatLog_2024_04_19_large.html").unwrap().len();
    app.date = NaiveDateTime::parse_from_str("2024/04/19 00:00:00", "%Y/%m/%d %H:%M:%S").unwrap();
    let date = NaiveDateTime::parse_from_str("2024/04/20 00:00:00", "%Y/%m/%d %H:%M:%S").unwrap();
    app.read_log(path, date).unwrap();
    assert_eq!(app.messages, [(vec![], false), (vec![], false), (vec![], false), (vec![], false), (vec![], false), (vec![], false), (vec![], false)]);
  }

  #[test]
  fn get_messages() {
    let mut app = App::new();
    for i in 0..app.messages.len() {
      app.messages[i].0.push(("test message".to_string(), "#000000".to_string(), "[ 0時  0分  0秒]".to_string()));
      app.messages[i].1 = true;
    }
    assert_eq!(app.get_messages(), [
      (vec![("test message".to_string(), "#000000".to_string(), "[ 0時  0分  0秒]".to_string())], true),
      (vec![("test message".to_string(), "#000000".to_string(), "[ 0時  0分  0秒]".to_string())], true),
      (vec![("test message".to_string(), "#000000".to_string(), "[ 0時  0分  0秒]".to_string())], true),
      (vec![("test message".to_string(), "#000000".to_string(), "[ 0時  0分  0秒]".to_string())], true),
      (vec![("test message".to_string(), "#000000".to_string(), "[ 0時  0分  0秒]".to_string())], true),
      (vec![("test message".to_string(), "#000000".to_string(), "[ 0時  0分  0秒]".to_string())], true),
      (vec![("test message".to_string(), "#000000".to_string(), "[ 0時  0分  0秒]".to_string())], true),
    ]);

    for i in 0..app.messages.len() {
      app.messages[i].0 = Vec::new();
      for _ in 0..600 {
        app.messages[i].0.push(("test message".to_string(), "#000000".to_string(), "[ 0時  0分  0秒]".to_string()));
        app.messages[i].1 = true;
      }
    }
    let mut expected = Vec::new();
    for _ in 0..500 {
      expected.push(("test message".to_string(), "#000000".to_string(), "[ 0時  0分  0秒]".to_string()));
    }
    assert_eq!(app.get_messages(), [
      (expected.clone(), true),
      (expected.clone(), true),
      (expected.clone(), true),
      (expected.clone(), true),
      (expected.clone(), true),
      (expected.clone(), true),
      (expected.clone(), true),
    ]);

    for i in 0..app.messages.len() {
      app.messages[i].0 = Vec::new();
      app.messages[i].0.push(("test message".to_string(), "#000000".to_string(), "[ 0時  0分  0秒]".to_string()));
      app.messages[i].1 = true;
    }
    for _ in 0..600 {
      app.messages[1].0.push(("test message".to_string(), "#000000".to_string(), "[ 0時  0分  0秒]".to_string()));
      app.messages[1].1 = true;
    }
    let mut expected = Vec::new();
    for _ in 0..500 {
      expected.push(("test message".to_string(), "#000000".to_string(), "[ 0時  0分  0秒]".to_string()));
    }
    assert_eq!(app.get_messages(), [
      (vec![("test message".to_string(), "#000000".to_string(), "[ 0時  0分  0秒]".to_string())], true),
      (expected, true),
      (vec![("test message".to_string(), "#000000".to_string(), "[ 0時  0分  0秒]".to_string())], true),
      (vec![("test message".to_string(), "#000000".to_string(), "[ 0時  0分  0秒]".to_string())], true),
      (vec![("test message".to_string(), "#000000".to_string(), "[ 0時  0分  0秒]".to_string())], true),
      (vec![("test message".to_string(), "#000000".to_string(), "[ 0時  0分  0秒]".to_string())], true),
      (vec![("test message".to_string(), "#000000".to_string(), "[ 0時  0分  0秒]".to_string())], true),
    ]);
  }

  #[test]
  fn calc_exp() {
    let mut app = App::new();
    app.messages[5].0.push(("経験値が 30000 上がりました。".to_string(), "#000000".to_string(), "[ 0時  0分  0秒]".to_string()));
    app.messages[5].0.push(("経験値が 30000 上がりました。".to_string(), "#000000".to_string(), "[ 0時  0分  1秒]".to_string()));
    app.messages[5].0.push(("経験値が 30000 上がりました。".to_string(), "#000000".to_string(), "[ 0時  0分  2秒]".to_string()));
    app.messages[5].0.push(("経験値が 30000 上がりました。".to_string(), "#000000".to_string(), "[ 0時  0分  3秒]".to_string()));
    app.messages[5].1 = true;
    let now = NaiveDateTime::parse_from_str("2000/01/01 15:00:3", "%Y/%m/%d %H:%M:%S").unwrap();
    let exp = app.calc_exp(now);
    assert_eq!(exp, (40000, 40000 * 60, 40000 * 60 * 60));

    let mut app = App::new();
    app.messages[5].0.push(("経験値が 30000 上がりました。".to_string(), "#000000".to_string(), "[ 0時  0分  0秒]".to_string()));
    app.messages[5].0.push(("経験値が 30000 上がりました。".to_string(), "#000000".to_string(), "[ 0時  0分  0秒]".to_string()));
    app.messages[5].0.push(("経験値が 30000 上がりました。".to_string(), "#000000".to_string(), "[ 0時  0分  1秒]".to_string()));
    app.messages[5].0.push(("経験値が 30000 上がりました。".to_string(), "#000000".to_string(), "[ 0時  0分  1秒]".to_string()));
    app.messages[5].0.push(("経験値が 30000 上がりました。".to_string(), "#000000".to_string(), "[ 0時  0分  2秒]".to_string()));
    app.messages[5].0.push(("経験値が 30000 上がりました。".to_string(), "#000000".to_string(), "[ 0時  0分  2秒]".to_string()));
    app.messages[5].0.push(("経験値が 30000 上がりました。".to_string(), "#000000".to_string(), "[ 0時  0分  3秒]".to_string()));
    app.messages[5].0.push(("経験値が 30000 上がりました。".to_string(), "#000000".to_string(), "[ 0時  0分  3秒]".to_string()));
    app.messages[5].1 = true;
    let now = NaiveDateTime::parse_from_str("2000/01/01 15:00:3", "%Y/%m/%d %H:%M:%S").unwrap();
    let exp = app.calc_exp(now);
    assert_eq!(exp, (240000 / 3, 240000 / 3 * 60, 240000 / 3 * 60 * 60));

    let mut app = App::new();
    app.messages[5].0.push(("経験値が 30000 上がりました。".to_string(), "#000000".to_string(), "[ 0時  0分  0秒]".to_string()));
    app.messages[5].0.push(("経験値が 30000 上がりました。".to_string(), "#000000".to_string(), "[ 0時  0分  3秒]".to_string()));
    app.messages[5].1 = true;
    let now = NaiveDateTime::parse_from_str("2000/01/01 15:00:3", "%Y/%m/%d %H:%M:%S").unwrap();
    let exp = app.calc_exp(now);
    assert_eq!(exp, (20000, 20000 * 60, 20000 * 60 * 60));

    app.messages[5].0.clear();
    app.messages[5].0.push(("経験値が 30000 上がりました。".to_string(), "#000000".to_string(), "[ 0時  0分  0秒]".to_string()));
    app.messages[5].0.push(("経験値が 30000 上がりました。".to_string(), "#000000".to_string(), "[ 0時  0分  1秒]".to_string()));
    app.messages[5].0.push(("経験値が 30000 上がりました。".to_string(), "#000000".to_string(), "[ 0時  0分  2秒]".to_string()));
    app.messages[5].0.push(("経験値が 30000 上がりました。".to_string(), "#000000".to_string(), "[ 0時  0分  3秒]".to_string()));
    app.messages[5].0.push(("経験値が 30000 上がりました。".to_string(), "#000000".to_string(), "[ 0時  0分  4秒]".to_string()));
    app.messages[5].1 = true;
    let now = NaiveDateTime::parse_from_str("2000/01/01 15:00:4", "%Y/%m/%d %H:%M:%S").unwrap();
    let exp = app.calc_exp(now);
    assert_eq!(exp, (40000, 40000 * 60, 40000 * 60 * 60));

    app.messages[5].0.clear();
    app.messages[5].0.push(("経験値が 30000 上がりました。".to_string(), "#000000".to_string(), "[23時 59分 57秒]".to_string()));
    app.messages[5].0.push(("経験値が 30000 上がりました。".to_string(), "#000000".to_string(), "[23時 59分 58秒]".to_string()));
    app.messages[5].0.push(("経験値が 30000 上がりました。".to_string(), "#000000".to_string(), "[23時 59分 59秒]".to_string()));
    app.messages[5].0.push(("経験値が 30000 上がりました。".to_string(), "#000000".to_string(), "[ 0時  0分  0秒]".to_string()));
    app.messages[5].0.push(("経験値が 30000 上がりました。".to_string(), "#000000".to_string(), "[ 0時  0分  1秒]".to_string()));
    app.messages[5].1 = true;
    let now = NaiveDateTime::parse_from_str("2000/01/01 15:00:1", "%Y/%m/%d %H:%M:%S").unwrap();
    let exp = app.calc_exp(now);
    assert_eq!(exp, (40000, 40000 * 60, 40000 * 60 * 60));
  }
}
