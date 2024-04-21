use chrono::{DateTime, Datelike, NaiveDateTime, TimeZone, Timelike, Utc};
use chrono_tz::{Asia::Tokyo, Chile::Continental, Tz};
use color_eyre::{
    eyre::{bail, WrapErr},
    Result,
};
use crossterm::{event::{self, poll, Event, KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEvent, MouseEventKind}, ExecutableCommand};
use memmap2::Mmap;
use ratatui::{
    prelude::*,
    symbols::border,
    widgets::{
        block::{Position, Title},
        *,
    },
};
use std::{io, path::Path, process::Command, str::FromStr, time::{Duration, Instant}};
use std::{fs, io::{Read, Seek, SeekFrom}, thread, time};
use std::fs::File;
use regex::Regex;

mod errors;
mod tui;

// todo wrap

#[derive(Debug)]
pub struct App<'a> {
    exit: bool,
    file_size: u64,
    verbose: bool,
    vertical: bool,
    auto_scroll: bool,
    panes: [(u16, u16, u16, u16, bool); 7],
    pane_names: [&'a str; 7],
    messages: [Vec<(String, String, String)>; 7],
    scroll: [u16; 7],
    drag: (u16, u16, usize),
    check_boxes: [(u16, u16, u16, u16); 10],
    date: NaiveDateTime,
    file: Option<File>,
}

impl<'a> App<'a> {
    pub fn run() -> Result<()> {
        Command::new("cmd").args(["/c", "title Neos"]).output().unwrap();
        errors::install_hooks()?;
        let mut terminal = tui::init()?;
        App::new().main_loop(&mut terminal)?;
        tui::restore()?;
        Ok(())
    }

    fn new() -> Self {
        App {
            exit: false,
            file_size: 0,
            verbose: false,
            vertical: true,
            auto_scroll: true,
            panes: [
                (0, 0, 0, 0, true),
                (0, 0, 0, 0, false),
                (0, 0, 0, 0, false),
                (0, 0, 0, 0, true),
                (0, 0, 0, 0, true),
                (0, 0, 0, 0, true),
                (0, 0, 0, 0, false),
            ],
            pane_names: ["All", "Public", "Private", "Team", "Club", "System", "Server"],
            messages: [Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new()],
            scroll: [0; 7],
            drag: (0, 0, 0),
            check_boxes: [(0, 0, 0, 0); 10],
            date: Utc::now().naive_utc(),
            file: None,
        }
    }

    /// runs the application's main loop until the user quits
    fn main_loop(&mut self, terminal: &mut tui::Tui) -> Result<()> {
        std::io::stdout().execute(crossterm::event::EnableMouseCapture)?;
        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events().wrap_err("handle events failed")?;
            let utc = Utc::now().naive_utc();
            let now = Tokyo.from_utc_datetime(&utc);
            let past = Tokyo.from_utc_datetime(&self.date);
            let path = format!("C:\\Nexon\\TalesWeaver\\ChatLog\\TWChatLog_{}_{:>02}_{:>02}.html", now.year(), now.month(), now.day());
            let path = Path::new(&path);
            if !path.is_file() {
                continue;
            }
            if let None = self.file {
                self.file = Some(File::open(path)?);
            } else if past.day() != now.day() {
                let file = self.file.take();
                drop(file);
                self.file = Some(File::open(path)?);
            }
            self.read_log(&path, utc).unwrap();
        }
        Ok(())
    }

    fn render_frame(&mut self, frame: &mut Frame) {
        let roots = Layout::default().direction(Direction::Vertical).constraints(vec![Constraint::Length(1), Constraint::Percentage(100)]).split(frame.size());
        let labels = vec![
            if self.panes[0].4 { "✅ All  " } else { "🔲 All  " },
            if self.panes[1].4 { "✅ Public  " } else { "🔲 Public  " },
            if self.panes[2].4 { "✅ Private  " } else { "🔲 Private  " },
            if self.panes[3].4 { "✅ Team  " } else { "🔲 Team  " },
            if self.panes[4].4 { "✅ Club  " } else { "🔲 Club  " },
            if self.panes[5].4 { "✅ System  " } else { "🔲 System  " },
            if self.panes[6].4 { "✅ Server  " } else { "🔲 Server  " },
            if self.verbose { "✅ Time  " } else { "🔲 Time  " },
            if self.vertical { "✅ Vertical  " } else { "🔲 Vertical  " },
            if self.auto_scroll { "✅ Auto scroll" } else { "🔲 Auto scroll" },
        ];
        let mut constraints = Vec::new();
        for label in &labels {
            constraints.push(Constraint::Length(Text::from(*label).width() as u16));
        }
        let check_boxes = Layout::default().direction(Direction::Horizontal).constraints(constraints).split(roots[0]);
        for i in 0..labels.len() {
            frame.render_widget(Paragraph::new(labels[i]), check_boxes[i]);
            self.check_boxes[i] = (check_boxes[i].x, check_boxes[i].y, check_boxes[i].width, check_boxes[i].height);
        }

        let pane_count = self.panes.iter().filter(|e| e.4).collect::<Vec<_>>().len() as u16;
        let percentage = 100 / pane_count as u16;
        let mut constraints = Vec::new();
        let mut remainder = 100 - percentage * pane_count;
        for _ in 0..pane_count {
            if 0 < remainder {
                constraints.push(Constraint::Percentage(percentage + 1));
                remainder -= 1;
            } else {
                constraints.push(Constraint::Percentage(percentage));
            }
        }
        let panes = if self.vertical {
            Layout::default().direction(Direction::Vertical).constraints(constraints).split(roots[1])
        } else {
            Layout::default().direction(Direction::Horizontal).constraints(constraints).split(roots[1])
        };
        let mut visible_pane_i = 0;
        for pane_i in 0..self.panes.len() {
            if self.panes[pane_i].4 {
                self.panes[pane_i] = (panes[visible_pane_i].x, panes[visible_pane_i].y, panes[visible_pane_i].width, panes[visible_pane_i].height, self.panes[pane_i].4);
                let texts = self.messages[pane_i].iter().map(|e| Line::from(if self.verbose { format!("{}{}", e.2, e.0) } else { format!("{}", e.0) }).fg(Color::from_str(e.1.as_str()).unwrap())).collect::<Vec<_>>();
                let row_count = texts.len();
                if self.auto_scroll && 2 <= panes[visible_pane_i].height && panes[visible_pane_i].height - 2 <= row_count as u16 {
                    self.scroll[pane_i] = row_count as u16 - (panes[visible_pane_i].height - 2);
                }
                if 2 <= self.panes[pane_i].3 && row_count as u16 <= self.panes[pane_i].3 - 2 {
                    self.scroll[pane_i] = 0;
                }
                frame.render_widget(
                    Paragraph::new(texts).scroll((self.scroll[pane_i], 0)).block(Block::new().title(Title::from(self.pane_names[pane_i])).borders(Borders::ALL)),
                    panes[visible_pane_i]);
                if 2 <= panes[visible_pane_i].height && panes[visible_pane_i].height - 2 < row_count as u16 {
                    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight);
                    let mut scrollbar_state = ScrollbarState::new(row_count - (panes[visible_pane_i].height as usize - 2)).position(self.scroll[pane_i] as usize);
                    frame.render_stateful_widget(scrollbar, panes[visible_pane_i].inner(&Margin { vertical: 1, horizontal: 0 }), &mut scrollbar_state);
                }
                visible_pane_i += 1;
            }
        }
    }

    fn read_log(&mut self, path: &Path, date: NaiveDateTime) -> Result<()> {
        let now = Tokyo.from_utc_datetime(&date);
        let past = Tokyo.from_utc_datetime(&self.date);
        self.date = date;

        let file_size = fs::metadata(path)?.len();  
        if self.file_size == 0 {
            self.file_size = file_size;
            return Ok(());
        }
        if self.file_size == file_size && past.day() == now.day() {
            return Ok(());
        }
        let content = unsafe { Mmap::map(self.file.as_ref().unwrap())? };
        let content = if past.day() == now.day() {
            &content[self.file_size as usize..file_size as usize]
        } else {
            &content[..]
        };
        self.file_size = file_size;
        let (cow, _, _) = encoding_rs::SHIFT_JIS.decode(&content);
        let message = cow.into_owned();

        let mut messages: Vec<_> = message.split("\r\n").filter(|e| e.trim() != "").collect();
        if past.day() != now.day() {
            for _ in 0..4 {
                messages.remove(0);
            }
        }
        let regex = Regex::new(r##"^<font.+> (.+)</font> <font.+color="(.+)">(.+)</font></br>$"##).unwrap();
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
                        _ => bail!("invalid captured color.: {} {} {}", color, time, message)
                    };
                    self.messages[0].push(((*message).clone(), color.to_string(), time.to_string()));
                    self.messages[i].push(((*message).clone(), color.to_string(), time.to_string()));
                }
                _ => bail!("regex does not match.: {}", message),
            }
        }
        Ok(())
    }

    /// updates the application's state based on user input
    fn handle_events(&mut self) -> Result<()> {
        if poll(Duration::from_millis(100))? {
            match event::read()? {
                // it's important to check that the event is a key press event as
                // crossterm also emits key release and repeat events on Windows.
                Event::Key(event) if event.kind == KeyEventKind::Press => {
                    self.handle_key_event(event).wrap_err_with(|| {
                        format!("handling key event failed:\n{event:#?}")
                    })
                }
                Event::Mouse(event) => self.handle_mouse_event(event),
                Event::Resize(width, height) => self.handle_resize_event(width, height),
                _ => Ok(()),
            }
        } else {
            Ok(())
        }
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Char('q') => {
                if cfg!(debug_assertions) {
                    self.exit = true;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_mouse_event(&mut self, event: MouseEvent) -> Result<()> {
        const SCROLL: u16 = 5;
        match event.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                for i in 0..self.panes.len() {
                    if !self.panes[i].4 {
                        continue;
                    }
                    if !(self.panes[i].0 <= event.column && event.column <= self.panes[i].0 + self.panes[i].2 && self.panes[i].1 <= event.row && event.row <= self.panes[i].1 + self.panes[i].3) {
                        continue;
                    }
                    self.drag = (event.column, event.row, i);
                    break;
                }
                for i in 0..self.check_boxes.len() {
                    if !(self.check_boxes[i].0 <= event.column && event.column <= self.check_boxes[i].0 + self.check_boxes[i].2 && self.check_boxes[i].1 <= event.row && event.row <= self.check_boxes[i].1 + self.check_boxes[i].3) {
                        continue;
                    }
                    if i == 7 {
                        self.verbose = !self.verbose;
                        return Ok(());
                    }
                    if i == 8 {
                        self.vertical = !self.vertical;
                        return Ok(());
                    }
                    if i == 9 {
                        self.auto_scroll = !self.auto_scroll;
                        return Ok(());
                    }
                    if self.panes.iter().filter(|e| e.4).collect::<Vec<_>>().len() == 1 && self.panes[i].4 {
                        return Ok(());
                    }
                    self.panes[i].4 = !self.panes[i].4;
                    return Ok(());
                }
                if self.auto_scroll {
                    return Ok(());
                }
                // click begin symbol
                for i in 0..self.panes.len() {
                    if !self.panes[i].4 {
                        continue;
                    }
                    if !(self.panes[i].0 + self.panes[i].2 - 1 <= event.column && event.column <= self.panes[i].0 + self.panes[i].2 && self.panes[i].1 <= event.row && event.row <= self.panes[i].1 + 1) {
                        continue;
                    }
                    if self.scroll[i] != 0 {
                        self.scroll[i] -= 1;
                    }
                    return Ok(());
                }
                // click end symbol
                for i in 0..self.panes.len() {
                    if !self.panes[i].4 {
                        continue;
                    }
                    if !(self.panes[i].0 + self.panes[i].2 - 1 <= event.column && event.column <= self.panes[i].0 + self.panes[i].2 && self.panes[i].1 + self.panes[i].3 - 2 <= event.row && event.row <= self.panes[i].1 + self.panes[i].3 - 1) {
                        continue;
                    }
                    if self.panes[i].3 < 2 || self.messages[i].len() < self.panes[i].3 as usize - 2 {
                        continue;
                    }
                    if self.scroll[i] + 1 <= self.messages[i].len() as u16 - (self.panes[i].3 - 2) {
                        self.scroll[i] += 1;
                    } else {
                        self.scroll[i] = self.messages[i].len() as u16 - (self.panes[i].3 - 2);
                    }
                    return Ok(());
                }
            },
            MouseEventKind::Drag(MouseButton::Left) => {
                if self.auto_scroll {
                    return Ok(());
                }
                if self.panes[self.drag.2].3 < 2 || self.messages[self.drag.2].len() <= self.panes[self.drag.2].3 as usize - 2 {
                    return Ok(());
                }
                if event.row <= self.drag.1 {
                    if 4 <= self.panes[self.drag.2].3 && (self.drag.1 - event.row) * (self.messages[self.drag.2].len() as u16 / (self.panes[self.drag.2].3 - 4)) <= self.scroll[self.drag.2] {
                        self.scroll[self.drag.2] -= (self.drag.1 - event.row) * (self.messages[self.drag.2].len() as u16 / (self.panes[self.drag.2].3 - 4));
                        self.drag = (event.column, event.row, self.drag.2);
                        return Ok(());
                    }
                    self.scroll[self.drag.2] = 0;
                    self.drag = (event.column, event.row, self.drag.2);
                    return Ok(());
                }
                if 4 <= self.panes[self.drag.2].3 && self.scroll[self.drag.2] + (event.row - self.drag.1) * (self.messages[self.drag.2].len() as u16 / (self.panes[self.drag.2].3 - 4)) <= self.messages[self.drag.2].len() as u16 - (self.panes[self.drag.2].3 - 2) {
                    self.scroll[self.drag.2] += (event.row - self.drag.1) * (self.messages[self.drag.2].len() as u16 / (self.panes[self.drag.2].3 - 4));
                    self.drag = (event.column, event.row, self.drag.2);
                    return Ok(());
                }
                self.scroll[self.drag.2] = self.messages[self.drag.2].len() as u16 - (self.panes[self.drag.2].3 - 2);
                self.drag = (event.column, event.row, self.drag.2);
            }
            MouseEventKind::ScrollUp => {
                if self.auto_scroll {
                    return Ok(());
                }
                for i in 0..self.panes.len() {
                    if !(self.panes[i].0 <= event.column && event.column <= self.panes[i].0 + self.panes[i].2 && self.panes[i].1 <= event.row && event.row <= self.panes[i].1 + self.panes[i].3) {
                        continue;
                    }
                    if self.panes[i].3 < 2 || self.messages[i].len() < self.panes[i].3 as usize - 2 {
                        continue;
                    }
                    if SCROLL <= self.scroll[i] {
                        self.scroll[i] -= SCROLL;
                    } else {
                        self.scroll[i] = 0;
                    }
                    break;
                }
            },
            MouseEventKind::ScrollDown => {
                if self.auto_scroll {
                    return Ok(());
                }
                for i in 0..self.panes.len() {
                    if !(self.panes[i].0 <= event.column && event.column <= self.panes[i].0 + self.panes[i].2 && self.panes[i].1 <= event.row && event.row <= self.panes[i].1 + self.panes[i].3) {
                        continue;
                    }
                    if self.panes[i].3 < 2 || self.messages[i].len() < self.panes[i].3 as usize - 2 {
                        continue;
                    }
                    if self.scroll[i] + SCROLL <= self.messages[i].len() as u16 - (self.panes[i].3 - 2) {
                        self.scroll[i] += SCROLL;
                    } else {
                        self.scroll[i] = self.messages[i].len() as u16 - (self.panes[i].3 - 2);
                    }
                    break;
                }
            },
            _ => {}
        }
        Ok(())
    }

    fn handle_resize_event(&mut self, _: u16, _: u16) -> Result<()> {
        for i in 0..self.panes.len() {
            if !self.panes[i].4 {
                continue;
            }
            if !self.auto_scroll && 2 <= self.panes[i].3 && self.messages[i].len() as u16 <= self.panes[i].3 - 2 {
                self.scroll[i] = 0;
            }
        }
        Ok(())
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
                    (String::from("◇本日の毎日課題：ステッドを退治"), String::from("#ff64ff"), String::from("[ 0時  0分  0秒] ")),
                    (String::from("◇本日の毎日課題：アビス深層96階以上クリア"), String::from("#ff64ff"), String::from("[ 0時  0分  0秒] ")),
                    (String::from("◇本日の毎日課題：ピリ辛ナテスコ煮"), String::from("#ff64ff"), String::from("[ 0時  0分  0秒] ")),
                    (String::from("?フォレスト?が1分後に「ルーンの庭園」を訪問します。"), String::from("#ff64ff"), String::from("[ 0時  0分  0秒] ")),
                    (String::from("ルーンの庭園へ訪れると、妖精や精霊たちが嬉しそうに迎えてくれます。"), String::from("#ff64ff"), String::from("[ 0時  0分  0秒] ")),
                    (String::from("プシーキーの迷宮が再設定されました。"), String::from("#ff64ff"), String::from("[ 0時  0分  1秒] ")),
                    (String::from("プラバ防衛戦が始まりました。"), String::from("#ff64ff"), String::from("[ 0時  0分  1秒] ")),
                    (String::from("[チーム経験値アップイベント] 始まりました！"), String::from("#ff64ff"), String::from("[ 0時  0分  1秒] ")),
                    (String::from("ランダムレイドバトルに参加できます。[ クラド ]でポータルを利用して入場してくださ"), String::from("#ff64ff"), String::from("[ 0時  0分  2秒] ")),
                    (String::from("い。"), String::from("#ff64ff"), String::from("[ 0時  0分  2秒] ")),
                    (String::from("[チーム経験値アップイベント] 実施中です！"), String::from("#ff64ff"), String::from("[ 0時  0分 59秒] ")),
                ],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![
                    (String::from("◇本日の毎日課題：ステッドを退治"), String::from("#ff64ff"), String::from("[ 0時  0分  0秒] ")),
                    (String::from("◇本日の毎日課題：アビス深層96階以上クリア"), String::from("#ff64ff"), String::from("[ 0時  0分  0秒] ")),
                    (String::from("◇本日の毎日課題：ピリ辛ナテスコ煮"), String::from("#ff64ff"), String::from("[ 0時  0分  0秒] ")),
                    (String::from("?フォレスト?が1分後に「ルーンの庭園」を訪問します。"), String::from("#ff64ff"), String::from("[ 0時  0分  0秒] ")),
                    (String::from("ルーンの庭園へ訪れると、妖精や精霊たちが嬉しそうに迎えてくれます。"), String::from("#ff64ff"), String::from("[ 0時  0分  0秒] ")),
                    (String::from("プシーキーの迷宮が再設定されました。"), String::from("#ff64ff"), String::from("[ 0時  0分  1秒] ")),
                    (String::from("プラバ防衛戦が始まりました。"), String::from("#ff64ff"), String::from("[ 0時  0分  1秒] ")),
                    (String::from("[チーム経験値アップイベント] 始まりました！"), String::from("#ff64ff"), String::from("[ 0時  0分  1秒] ")),
                    (String::from("ランダムレイドバトルに参加できます。[ クラド ]でポータルを利用して入場してくださ"), String::from("#ff64ff"), String::from("[ 0時  0分  2秒] ")),
                    (String::from("い。"), String::from("#ff64ff"), String::from("[ 0時  0分  2秒] ")),
                    (String::from("[チーム経験値アップイベント] 実施中です！"), String::from("#ff64ff"), String::from("[ 0時  0分 59秒] ")),
                ],
                vec![],
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
                    (String::from("◇本日の毎日課題：ステッドを退治"), String::from("#ff64ff"), String::from("[ 0時  0分  0秒] ")),
                    (String::from("◇本日の毎日課題：アビス深層96階以上クリア"), String::from("#ff64ff"), String::from("[ 0時  0分  0秒] ")),
                    (String::from("◇本日の毎日課題：ピリ辛ナテスコ煮"), String::from("#ff64ff"), String::from("[ 0時  0分  0秒] ")),
                    (String::from("?フォレスト?が1分後に「ルーンの庭園」を訪問します。"), String::from("#ff64ff"), String::from("[ 0時  0分  0秒] ")),
                    (String::from("ルーンの庭園へ訪れると、妖精や精霊たちが嬉しそうに迎えてくれます。"), String::from("#ff64ff"), String::from("[ 0時  0分  0秒] ")),
                    (String::from("プシーキーの迷宮が再設定されました。"), String::from("#ff64ff"), String::from("[ 0時  0分  1秒] ")),
                    (String::from("プラバ防衛戦が始まりました。"), String::from("#ff64ff"), String::from("[ 0時  0分  1秒] ")),
                    (String::from("[チーム経験値アップイベント] 始まりました！"), String::from("#ff64ff"), String::from("[ 0時  0分  1秒] ")),
                    (String::from("ランダムレイドバトルに参加できます。[ クラド ]でポータルを利用して入場してくださ"), String::from("#ff64ff"), String::from("[ 0時  0分  2秒] ")),
                    (String::from("い。"), String::from("#ff64ff"), String::from("[ 0時  0分  2秒] ")),
                    (String::from("[チーム経験値アップイベント] 実施中です！"), String::from("#ff64ff"), String::from("[ 0時  0分 59秒] ")),
                ],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![
                    (String::from("◇本日の毎日課題：ステッドを退治"), String::from("#ff64ff"), String::from("[ 0時  0分  0秒] ")),
                    (String::from("◇本日の毎日課題：アビス深層96階以上クリア"), String::from("#ff64ff"), String::from("[ 0時  0分  0秒] ")),
                    (String::from("◇本日の毎日課題：ピリ辛ナテスコ煮"), String::from("#ff64ff"), String::from("[ 0時  0分  0秒] ")),
                    (String::from("?フォレスト?が1分後に「ルーンの庭園」を訪問します。"), String::from("#ff64ff"), String::from("[ 0時  0分  0秒] ")),
                    (String::from("ルーンの庭園へ訪れると、妖精や精霊たちが嬉しそうに迎えてくれます。"), String::from("#ff64ff"), String::from("[ 0時  0分  0秒] ")),
                    (String::from("プシーキーの迷宮が再設定されました。"), String::from("#ff64ff"), String::from("[ 0時  0分  1秒] ")),
                    (String::from("プラバ防衛戦が始まりました。"), String::from("#ff64ff"), String::from("[ 0時  0分  1秒] ")),
                    (String::from("[チーム経験値アップイベント] 始まりました！"), String::from("#ff64ff"), String::from("[ 0時  0分  1秒] ")),
                    (String::from("ランダムレイドバトルに参加できます。[ クラド ]でポータルを利用して入場してくださ"), String::from("#ff64ff"), String::from("[ 0時  0分  2秒] ")),
                    (String::from("い。"), String::from("#ff64ff"), String::from("[ 0時  0分  2秒] ")),
                    (String::from("[チーム経験値アップイベント] 実施中です！"), String::from("#ff64ff"), String::from("[ 0時  0分 59秒] ")),
                ],
                vec![],
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
