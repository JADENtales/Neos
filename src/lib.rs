use color_eyre::{
    eyre::{bail, WrapErr},
    Result,
};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    prelude::*,
    symbols::border,
    widgets::{
        block::{Position, Title},
        *,
    },
};

mod errors;
mod tui;

#[derive(Debug, Default)]
pub struct App<'a> {
    counter: i64,
    exit: bool,
    vertical: bool,
    chats: [bool; 6],
    titles: [&'a str; 6],
}

impl<'a> App<'a> {
    pub fn start() -> Result<()> {
        errors::install_hooks()?;
        let mut terminal = tui::init()?;
        App::default().run(&mut terminal)?;
        tui::restore()?;
        Ok(())
    }

    /// runs the application's main loop until the user quits
    fn run(&mut self, terminal: &mut tui::Tui) -> Result<()> {
        self.chats.iter_mut().for_each(|e| *e = true);
        self.titles = ["All(1)", "Public(2)", "Private(3)", "Team(4)", "Club(5)", "System(6)"];
        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events().wrap_err("handle events failed")?;
        }
        Ok(())
    }

    fn render_frame(&self, frame: &mut Frame) {
        let descriptions = vec![
            "1: All, 2: Public, 3: Private, 4: Team, 5: Club, 6: System".into(),
            "q: 終了".into(),
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
                frame.render_widget(Paragraph::new("").block(Block::new().title(Title::from(self.titles[i])).borders(Borders::ALL)), children[index]);
                index += 1;
            }
        }
    }

    /// updates the application's state based on user input
    fn handle_events(&mut self) -> Result<()> {
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
        let mut app = App::default();
        app.handle_key_event(KeyCode::Right.into()).unwrap();
        assert_eq!(app.counter, 1);

        app.handle_key_event(KeyCode::Left.into()).unwrap();
        assert_eq!(app.counter, 0);

        let mut app = App::default();
        app.handle_key_event(KeyCode::Char('q').into()).unwrap();
        assert_eq!(app.exit, true);
    }

    #[test]
    #[should_panic(expected = "attempt to subtract with overflow")]
    fn handle_key_event_panic() {
        let mut app = App::default();
        let _ = app.handle_key_event(KeyCode::Left.into());
    }

    #[test]
    fn handle_key_event_overflow() {
        let mut app = App::default();
        assert!(app.handle_key_event(KeyCode::Right.into()).is_ok());
        assert!(app.handle_key_event(KeyCode::Right.into()).is_ok());
        assert_eq!(
            app.handle_key_event(KeyCode::Right.into())
                .unwrap_err()
                .to_string(),
            "counter overflow"
        );
    }
}
