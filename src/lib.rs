use color_eyre::{
    eyre::{bail, WrapErr},
    Result,
};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, poll};
use ratatui::{
    prelude::*,
    symbols::border,
    widgets::{
        block::{Position, Title},
        *,
    },
};
use std::{time::Duration, io};
use std::{fs, io::{Read, Seek, SeekFrom}, thread, time};
use std::fs::File;

mod errors;
mod tui;

#[derive(Debug, Default)]
pub struct App<'a> {
    exit: bool,
    vertical: bool,
    chats: [bool; 6],
    titles: [&'a str; 6],
    messages: [Vec<String>; 6],
    file_size: u64,
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
        self.chats.iter_mut().for_each(|e| *e = true);
        self.titles = ["All(1)", "Public(2)", "Private(3)", "Team(4)", "Club(5)", "System(6)"];
        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.read().unwrap();
            self.handle_events().wrap_err("handle events failed")?;
        }
        Ok(())
    }

    fn render_frame(&self, frame: &mut Frame) {
        let descriptions = vec![
            Line::from("1: All, 2: Public, 3: Private, 4: Team, 5: Club, 6: System"),
            Line::from("space: 分割方向切替, q: 終了"),
        ];
        let height = Into::<Text>::into(descriptions.clone()).height() as u16 * 2;
        let parents = Layout::default().direction(Direction::Vertical).constraints(vec![Constraint::Length(height), Constraint::Percentage(100)]).split(frame.size());
        frame.render_widget(Paragraph::new(descriptions).block(Block::new().title(Title::from("")).borders(Borders::ALL)), parents[0]);

        let chats_count = self.chats.iter().filter(|e| **e).collect::<Vec<&bool>>().len() as u16;
        let percentage = 100 / chats_count as u16;
        let mut constraints = Vec::new();
        let mut remainder = 100 - percentage * chats_count;
        for _ in 0..chats_count {
            if 0 < remainder {
                constraints.push(Constraint::Percentage(percentage + 1));
                remainder -= 1;
            } else {
                constraints.push(Constraint::Percentage(percentage));
            }
        }
        let children = if self.vertical {
            Layout::default().direction(Direction::Vertical).constraints(constraints).split(parents[1])
        } else {
            Layout::default().direction(Direction::Horizontal).constraints(constraints).split(parents[1])
        };
        let mut index = 0;
        for (i, chat) in self.chats.iter().enumerate() {
            if *chat {
                let text: Vec<Line> = self.messages[0].iter().map(|e| Line::from((*e).clone())).collect();
                frame.render_widget(Paragraph::new(text).wrap(Wrap { trim: false }).block(Block::new().title(Title::from(self.titles[i])).borders(Borders::ALL)), children[index]);
                index += 1;
            }
        }
    }

    fn read(&mut self) -> Result<()> {
        let path = "C:\\Nexon\\TalesWeaver\\ChatLog\\TWChatLog_2024_04_18.html";
        let mut file = File::open(path)?;
        let file_size = fs::metadata(path)?.len();  
        let offset = if 1024 < file_size { -1024 } else { -(file_size as i64) };
        file.seek(SeekFrom::End(offset))?;

        let mut content = [0; 1024];
        file.read(&mut content)?;
        let (cow, _, _) = encoding_rs::SHIFT_JIS.decode(&content);
        let content = cow.into_owned();
        let messages: Vec<&str> = content.split("\r\n").filter(|e| e.trim() != "").collect();

        let mut shift_jis_messages = Vec::new();
        for i in 0..messages.len() {
            let (cow, _, _) = encoding_rs::SHIFT_JIS.encode(messages[messages.len() - 1 - i]);
            shift_jis_messages.push(cow.into_owned());
            let shift_jis_message_size = (shift_jis_messages.iter().map(|e| e.len()).sum::<usize>() + (i + 1) * 2) as u64;
            let diff = file_size - self.file_size;
            if self.file_size == 0 || diff == 0 {
                self.file_size = file_size;
                return Ok(());
            }
            if diff == shift_jis_message_size {
                self.file_size = file_size;
                let start = messages.len() - i - 1;
                self.messages[0].push(String::from(messages[start..].join("\n")));
                println!("message size matches. {} {} {}", diff, shift_jis_message_size, self.file_size);
                return Ok(());
            }
            // bail!(format!("message size does not match. {} {} {}", diff, shift_jis_message_size, self.file_size));
            // println!("message size does not match. {} {} {}", diff, shift_jis_message_size, self.file_size);
        }
        bail!("message size does not match.")
    }

    /// updates the application's state based on user input
    fn handle_events(&mut self) -> Result<()> {
        if poll(Duration::from_millis(1))? {
            match event::read()? {
                // it's important to check that the event is a key press event as
                // crossterm also emits key release and repeat events on Windows.
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    self.handle_key_event(key_event).wrap_err_with(|| {
                        format!("handling key event failed:\n{key_event:#?}")
                    })
                }
                _ => Ok(()),
            }
        } else {
            Ok(())
        }
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Char('q') => self.exit = true,
            KeyCode::Char(char) if '1' <= char && char <= '6' => {
                let i = (char.to_digit(10).unwrap() - 1) as usize;
                if self.chats.iter().filter(|e| **e).collect::<Vec<&bool>>().len() == 1 && self.chats[i] {
                    return Ok(())
                }
                self.chats[i] = !self.chats[i];
            }
            KeyCode::Char(' ') => self.vertical = !self.vertical,
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
