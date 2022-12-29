mod bot;
mod map;

use bot::{Nekobot, NekobotState};
use clap::Parser;
use crossterm::{
    cursor,
    event::{poll, read, Event, KeyCode},
    execute, queue,
    style::{self, Color},
    terminal, Result,
};
use map::NystopiaMap;
use std::io::{stdout, Write};
use std::time::{Duration, Instant};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct ProgArgs {
    /// Number of bots to create
    #[arg(short, long, default_value_t = 10, value_name = "BOTS")]
    bots: u8,

    /// Tick delay in msec (inverse of speed)
    #[arg(short, long, default_value_t = 250u128, value_name = "MSEC")]
    tick_delay: u128,

    /// Sight (how many squares ahead a bot can "see")
    #[arg(short, long, default_value_t = 10, value_name = "SQUARES")]
    sight: u16,

    /// Vegetation regrowth time (in ticks)
    #[arg(short, long, default_value_t = 100, value_name = "TICKS")]
    regrow_time: u16,

    /// Map vegetation probability (in percent)
    #[arg(short, long, default_value_t = 5, value_name = "PERCENT")]
    food_prob: u8,
}

fn main() -> Result<()> {
    let mut stdout = stdout();
    let prog_args = ProgArgs::parse();
    let mut nekomap = NystopiaMap::new(&prog_args)?;

    let mut count = prog_args.bots as usize;
    let period = prog_args.tick_delay;

    let labels = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz!@#$%^&*()";

    let mut nekobots: Vec<Nekobot> = vec![];

    while count > 0 {
        let label_index = count % labels.len();
        nekobots.push(Nekobot::new_rand(
            format!("{}", labels.get(label_index..(label_index + 1)).unwrap()).as_str(),
            nekomap.get_rows(),
            nekomap.get_cols(),
            &prog_args,
        ));
        count -= 1;
    }

    terminal::enable_raw_mode()?;

    // Clear terminal
    execute!(
        stdout,
        terminal::Clear(terminal::ClearType::All),
        cursor::Hide
    )?;

    let inst = Instant::now();
    let mut last_capture = inst.elapsed().as_millis();
    let mut next_stop = last_capture + period;

    loop {
        last_capture = inst.elapsed().as_millis();
        if last_capture >= next_stop {
            // Render the map
            nekomap.tick_map();
            nekomap.render_map(&mut stdout)?;

            // Render the bots
            for nekobot in nekobots.iter_mut() {
                let mut tile = nekomap
                    .get_tile(nekobot.get_row(), nekobot.get_col())
                    .expect("Failed to identify tile".into());
                // Remove the old placements
                queue!(
                    stdout,
                    cursor::MoveTo(nekobot.get_col(), nekobot.get_row()),
                    if !tile.eaten() && tile.has_food() {
                        style::SetBackgroundColor(Color::DarkGreen)
                    } else {
                        style::SetBackgroundColor(Color::Black)
                    },
                    style::Print(" ")
                )?;

                nekobot.tick(&mut nekomap);

                tile = nekomap
                    .get_tile(nekobot.get_row(), nekobot.get_col())
                    .expect("Failed to identify tile".into());

                // Draw the new placements
                queue!(
                    stdout,
                    cursor::MoveTo(nekobot.get_col(), nekobot.get_row()),
                    if !tile.eaten() && tile.has_food() {
                        style::SetBackgroundColor(Color::DarkGreen)
                    } else {
                        style::SetBackgroundColor(Color::Black)
                    },
                    match nekobot.get_state() {
                        NekobotState::Wander => style::SetForegroundColor(Color::Grey),
                        NekobotState::Forage => style::SetForegroundColor(Color::Yellow),
                        NekobotState::Dead => style::SetForegroundColor(Color::Red),
                    },
                    style::Print(nekobot.get_label().clone())
                )?;
            }

            next_stop = last_capture + period;
        }

        // Flush the output buffer
        stdout.flush()?;

        match poll(Duration::from_millis((next_stop - last_capture) as u64)) {
            Ok(true) => match read() {
                Ok(Event::Key(ev)) => {
                    if ev.code == KeyCode::Char('q') {
                        break;
                    }
                }
                _ => {}
            },
            Ok(false) => {}
            Err(_) => {}
        }
    }

    terminal::disable_raw_mode()?;

    // Put cursor at bottom-left before exit
    queue!(stdout, cursor::MoveTo(0, nekomap.get_rows() - 1))?;
    queue!(stdout, cursor::Show)?;
    stdout.flush()?;

    Ok(())
}
