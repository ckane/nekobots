use super::bot::{Nekobot, NekobotState};
use super::map::NystopiaMap;
use super::renderer::Renderer;
use crossterm::{
    cursor, execute, queue,
    style::{self, Color},
    terminal as term,
};
use std::io::{stdout, Stdout, Write};

pub struct Terminal {
    stdout: Stdout,
}

impl Renderer for Terminal {
    fn new() -> Self {
        Terminal { stdout: stdout() }
    }

    fn init(self: &mut Self) -> Result<(), Box<dyn std::error::Error>> {
        term::enable_raw_mode()?;
        self.blank()?;
        Ok(())
    }

    fn blank(self: &mut Self) -> Result<(), Box<dyn std::error::Error>> {
        // Clear terminal
        execute!(self.stdout, term::Clear(term::ClearType::All), cursor::Hide)?;
        Ok(())
    }

    fn render_map(self: &mut Self, map: &NystopiaMap) -> Result<(), Box<dyn std::error::Error>> {
        queue!(self.stdout, cursor::MoveTo(0, 0))?;

        for r in 0..map.get_rows() {
            for c in 0..map.get_cols() {
                if let Some(this_tile) = map.get_tile(r, c) {
                    if this_tile.has_food() && !this_tile.eaten() {
                        queue!(
                            self.stdout,
                            style::SetBackgroundColor(Color::DarkGreen),
                            style::Print(" ")
                        )?;
                    } else {
                        queue!(
                            self.stdout,
                            style::SetBackgroundColor(Color::Black),
                            style::Print(" ")
                        )?;
                    }
                }
            }
        }

        self.stdout.flush()?;

        Ok(())
    }

    fn get_rows(self: &Self) -> Result<u16, Box<dyn std::error::Error>> {
        let (_, rows) = term::size()?;
        Ok(rows)
    }

    fn get_cols(self: &Self) -> Result<u16, Box<dyn std::error::Error>> {
        let (cols, _) = term::size()?;
        Ok(cols)
    }

    fn place_tile(
        self: &mut Self,
        map: &NystopiaMap,
        row: u16,
        col: u16,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let tile = map
            .get_tile(row, col)
            .expect("Failed to identify tile".into());
        // Remove the old placements
        queue!(
            self.stdout,
            cursor::MoveTo(col, row),
            if !tile.eaten() && tile.has_food() {
                style::SetBackgroundColor(Color::DarkGreen)
            } else {
                style::SetBackgroundColor(Color::Black)
            },
            style::Print(" ")
        )?;
        Ok(())
    }

    fn place_bot(
        self: &mut Self,
        map: &NystopiaMap,
        bot: &Nekobot,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let tile = map
            .get_tile(bot.get_row(), bot.get_col())
            .expect("Failed to identify tile".into());

        // Draw the new placements
        queue!(
            self.stdout,
            cursor::MoveTo(bot.get_col(), bot.get_row()),
            if !tile.eaten() && tile.has_food() {
                style::SetBackgroundColor(Color::DarkGreen)
            } else {
                style::SetBackgroundColor(Color::Black)
            },
            match bot.get_state() {
                NekobotState::Wander => style::SetForegroundColor(Color::Grey),
                NekobotState::Forage => style::SetForegroundColor(Color::Yellow),
                NekobotState::Dead => style::SetForegroundColor(Color::Red),
            },
            style::Print(bot.get_label().clone())
        )?;
        Ok(())
    }
}

impl Drop for Terminal {
    fn drop(self: &mut Self) {
        term::disable_raw_mode().ok();
        // Put cursor at bottom-left before exit
        let bottom = self.get_rows().unwrap();
        queue!(self.stdout, cursor::MoveTo(0, bottom - 1)).ok();
        queue!(self.stdout, cursor::Show).ok();
        self.stdout.flush().ok();
    }
}
