use chrono::{DateTime, Datelike, NaiveDateTime, TimeZone, Utc};
use chrono_tz::{Asia::Tokyo, Tz};
use color_eyre::{
    eyre::{bail, WrapErr},
    Result,
};
use crossterm::{event::{self, poll, Event, KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEvent, MouseEventKind}, ExecutableCommand};
use ratatui::{
    prelude::*,
    symbols::border,
    widgets::{
        block::{Position, Title},
        *,
    },
};
use std::{io, path::Path, str::FromStr, time::Duration};
use std::{fs, io::{Read, Seek, SeekFrom}, thread, time};
use std::fs::File;
use regex::Regex;

mod errors;
mod tui;

#[derive(Debug, Default)]
pub struct App<'a> {
    exit: bool,
    file_size: u64,
    horizontal: bool,
    auto_scroll: bool,
    panes: [bool; 7],
    pane_heights: [u16; 7],
    pane_names: [&'a str; 7],
    messages: [Vec<(String, String)>; 7],
    scroll: [u16; 7],
    check_boxes: [(u16, u16, u16, u16); 9],
    date: NaiveDateTime,
}

impl<'a> App<'a> {
    pub fn run() -> Result<()> {
        errors::install_hooks()?;
        let mut terminal = tui::init()?;
        App::default().main_loop(&mut terminal)?;
        tui::restore()?;
        Ok(())
    }

    /// runs the application's main loop until the user quits
    fn main_loop(&mut self, terminal: &mut tui::Tui) -> Result<()> {
        std::io::stdout().execute(crossterm::event::EnableMouseCapture).unwrap();
        self.date = Utc::now().naive_utc();
        self.panes.iter_mut().for_each(|e| *e = true);
        self.horizontal = true;
        self.auto_scroll = true;
        self.pane_names = ["All", "Public", "Private", "Team", "Club", "System", "Server"];
        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.read().unwrap();
            self.handle_events().wrap_err("handle events failed")?;
        }
        Ok(())
    }

    fn render_frame(&mut self, frame: &mut Frame) {
        let roots = Layout::default().direction(Direction::Vertical).constraints(vec![Constraint::Length(1), Constraint::Percentage(100)]).split(frame.size());
        let labels = vec![
            if self.panes[0] { "✅ All  " } else { "🔲 All  " },
            if self.panes[1] { "✅ Public  " } else { "🔲 Public  " },
            if self.panes[2] { "✅ Private  " } else { "🔲 Private  " },
            if self.panes[3] { "✅ Team  " } else { "🔲 Team  " },
            if self.panes[4] { "✅ Club  " } else { "🔲 Club  " },
            if self.panes[5] { "✅ System  " } else { "🔲 System  " },
            if self.panes[6] { "✅ Server  " } else { "🔲 Server  " },
            if self.horizontal { "✅ Horizontal  " } else { "🔲 Horizontal  " },
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

        let pane_count = self.panes.iter().filter(|e| **e).collect::<Vec<&bool>>().len() as u16;
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
        let panes = if self.horizontal {
            Layout::default().direction(Direction::Horizontal).constraints(constraints).split(roots[1])
        } else {
            Layout::default().direction(Direction::Vertical).constraints(constraints).split(roots[1])
        };
        let mut visible_pane_i = 0;
        for (pane_i, pane) in self.panes.iter().enumerate() {
            if *pane {
                self.pane_heights[pane_i] = panes[visible_pane_i].height;
                let texts = self.messages[pane_i].iter().map(|e| Line::from(e.0.as_str()).fg(Color::from_str(e.1.as_str()).unwrap())).collect::<Vec<_>>();
                let row_count = texts.len();
                if self.auto_scroll && 2 <= panes[visible_pane_i].height && panes[visible_pane_i].height - 2 < row_count as u16 {
                    self.scroll[pane_i] = row_count as u16 - (panes[visible_pane_i].height - 2);
                }
                frame.render_widget(
                    Paragraph::new(texts).scroll((self.scroll[pane_i], 0)).block(Block::new().title(Title::from(self.pane_names[pane_i])).borders(Borders::ALL)),
                    panes[visible_pane_i]);
                if 2 <= panes[visible_pane_i].height && panes[visible_pane_i].height - 2 < row_count as u16 {
                    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight).begin_symbol(Some("↑")).end_symbol(Some("↓"));
                    let mut scrollbar_state = ScrollbarState::new(row_count - (panes[visible_pane_i].height as usize - 2)).position(self.scroll[pane_i] as usize);
                    frame.render_stateful_widget(scrollbar, panes[visible_pane_i].inner(&Margin { vertical: 1, horizontal: 0 }), &mut scrollbar_state);
                }
                visible_pane_i += 1;
            }
        }
    }

    fn read(&mut self) -> Result<()> {
        let utc = Utc::now().naive_utc();
        let now = Tokyo.from_utc_datetime(&utc);
        let past = Tokyo.from_utc_datetime(&self.date);
        let path = if past.day() == now.day() {
            format!("C:\\Nexon\\TalesWeaver\\ChatLog\\TWChatLog_{}_{:>02}_{:>02}.html", now.year(), now.month(), now.day())
        } else {
            self.date = utc;
            self.file_size = 0;
            format!("C:\\Nexon\\TalesWeaver\\ChatLog\\TWChatLog_{}_{:>02}_{:>02}.html", past.year(), past.month(), past.day())
        };
        let path = Path::new(path.as_str());
        if !path.is_file() {
            return Ok(());
        }
        let mut file = File::open(path)?;
        let file_size = fs::metadata(path)?.len();  

        let diff = file_size - self.file_size;
        if (self.file_size == 0 || diff == 0) && past.day() == now.day() {
            self.file_size = file_size;
            return Ok(());
        }
        self.file_size = file_size;
        let mut content = if past.day() == now.day() {
            file.seek(SeekFrom::End(-(diff as i64)))?;
            vec![0; diff as usize]
        } else {
            vec![0; file_size as usize]
        };
        file.read(&mut content)?;
        let (cow, _, _) = encoding_rs::SHIFT_JIS.decode(&content);
        let message = cow.into_owned();

        let messages: Vec<_> = message.split("\r\n").filter(|e| e.trim() != "").collect();
        let regex = Regex::new(r##" <font.+color="(.+)">(.+)</font></br>$"##).unwrap();
        for message in messages {
            match regex.captures(&message) {
                Some(captures) => {
                    let color = &captures[1];
                    let message = &captures[2].replace("&nbsp", " ");
                    let i = match color {
                        "#c8ffc8" => 1,
                        "#64ff64" => 2,
                        "#f7b73c" => 3,
                        "#94ddfa" => 4,
                        "#ff64ff" | "#ff6464" => 5,
                        "#c896c8" => 6,
                        _ => bail!("invalid captured color")
                    };
                    self.messages[0].push(((*message).clone(), color.to_string()));
                    self.messages[i].push(((*message).clone(), color.to_string()));
                }
                _ => bail!("regex does not match."),
            }
        }
        Ok(())
    }

    /// updates the application's state based on user input
    fn handle_events(&mut self) -> Result<()> {
        if poll(Duration::from_millis(1))? {
            match event::read()? {
                // it's important to check that the event is a key press event as
                // crossterm also emits key release and repeat events on Windows.
                Event::Key(event) if event.kind == KeyEventKind::Press => {
                    self.handle_key_event(event).wrap_err_with(|| {
                        format!("handling key event failed:\n{event:#?}")
                    })
                }
                Event::Mouse(event) => self.handle_mouse_event(event),
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
                for i in 0..self.check_boxes.len() {
                    if self.check_boxes[i].0 <= event.column && event.column <= self.check_boxes[i].0 + self.check_boxes[i].2 && self.check_boxes[i].1 <= event.row && event.row <= self.check_boxes[i].1 + self.check_boxes[i].3 {
                        if i == 7 {
                            self.horizontal = !self.horizontal;
                            break;
                        }
                        if i == 8 {
                            self.auto_scroll = !self.auto_scroll;
                            break;
                        }
                        if self.panes.iter().filter(|e| **e).collect::<Vec<&bool>>().len() == 1 && self.panes[i] {
                            break;
                        }
                        self.panes[i] = !self.panes[i];
                    }
                }
            },
            MouseEventKind::ScrollUp => {
                if self.auto_scroll {
                    return Ok(());
                }
                if self.pane_heights[0] < 2 || self.messages[0].len() < self.pane_heights[0] as usize - 2 {
                    return Ok(());
                }
                if SCROLL <= self.scroll[0] {
                    self.scroll[0] -= SCROLL;
                } else {
                    self.scroll[0] = 0;
                }
            },
            MouseEventKind::ScrollDown => {
                if self.auto_scroll {
                    return Ok(());
                }
                if self.pane_heights[0] < 2 || self.messages[0].len() < self.pane_heights[0] as usize - 2 {
                    return Ok(());
                }
                if self.scroll[0] + SCROLL <= self.messages[0].len() as u16 - (self.pane_heights[0] - 2) {
                    self.scroll[0] += SCROLL;
                } else {
                    self.scroll[0] = self.messages[0].len() as u16 - (self.pane_heights[0] - 2);
                }
            },
            _ => {}
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handle_key_event() {
    }
}
