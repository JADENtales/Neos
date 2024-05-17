use std::{fs, path::Path, fs::File, io::{Read, Seek, SeekFrom}};
use regex::Regex;
use chrono_tz::Asia::Tokyo;
use chrono::{Datelike, NaiveDateTime, TimeZone, Utc};
use anyhow::{bail, Result};

#[derive(Debug)]
pub struct App {
    pub views: Vec<bool>,
    pub verbose: bool,
    pub vertical: bool,
    pub auto_scroll: Vec<bool>,
    pub messages: Vec<Vec<(String, String, String)>>,
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
            verbose: false,
            vertical: true,
            auto_scroll: vec![true; 7],
            messages: vec![Vec::new(); 7],
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
                    self.messages[0].push(((*message).clone(), color.to_string(), time.to_string()));
                    self.messages[i].push(((*message).clone(), color.to_string(), time.to_string()));
                }
                _ => bail!("regex does not match.: {}", message),
            }
        }
        Ok(ReadStatus::Updated)
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
                vec![],
                vec![],
                vec![],
                vec![],
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
                vec![]
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
                vec![],
                vec![],
                vec![],
                vec![],
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
                vec![]
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
        assert_eq!(app.messages, [vec![], vec![], vec![], vec![], vec![], vec![], vec![]]);
    }
}
