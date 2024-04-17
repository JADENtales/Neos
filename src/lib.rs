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
pub struct App {
    counter: i64,
    exit: bool,
    vertical: bool,
}

impl App {
    pub fn start() -> Result<()> {
        errors::install_hooks()?;
        let mut terminal = tui::init()?;
        App::default().run(&mut terminal)?;
        tui::restore()?;
        Ok(())
    }

    /// runs the application's main loop until the user quits
    fn run(&mut self, terminal: &mut tui::Tui) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events().wrap_err("handle events failed")?;
        }
        Ok(())
    }

    fn render_frame(&self, frame: &mut Frame) {
        let layout = if self.vertical {
            Layout::default().direction(Direction::Vertical).constraints(vec![
                Constraint::Percentage(16),
                Constraint::Percentage(16),
                Constraint::Percentage(16),
                Constraint::Percentage(16),
                Constraint::Percentage(16),
                Constraint::Percentage(16),
            ]).split(frame.size())
        } else {
            Layout::default().direction(Direction::Horizontal).constraints(vec![
                Constraint::Percentage(16),
                Constraint::Percentage(16),
                Constraint::Percentage(16),
                Constraint::Percentage(16),
                Constraint::Percentage(16),
                Constraint::Percentage(16),
            ]).split(frame.size())
        };
        frame.render_widget(Paragraph::new("").block(Block::new().title(Title::from("全体")).borders(Borders::ALL)), layout[0]);
        frame.render_widget(Paragraph::new("").block(Block::new().title(Title::from("一般")).borders(Borders::ALL)), layout[1]);
        frame.render_widget(Paragraph::new("").block(Block::new().title(Title::from("耳打ち")).borders(Borders::ALL)), layout[2]);
        frame.render_widget(Paragraph::new("").block(Block::new().title(Title::from("チーム")).borders(Borders::ALL)), layout[3]);
        frame.render_widget(Paragraph::new("").block(Block::new().title(Title::from("クラブ")).borders(Borders::ALL)), layout[4]);
        frame.render_widget(Paragraph::new("").block(Block::new().title(Title::from("システム")).borders(Borders::ALL)), layout[5]);
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
            KeyCode::Char('q') => self.exit(),
            KeyCode::Left => self.decrement_counter()?,
            KeyCode::Right => self.increment_counter()?,
            _ => {}
        }
        Ok(())
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn decrement_counter(&mut self) -> Result<()> {
        self.counter -= 1;
        Ok(())
    }

    fn increment_counter(&mut self) -> Result<()> {
        self.counter += 1;
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
