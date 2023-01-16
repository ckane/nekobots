mod bot;
mod map;
mod renderer;
mod terminal;

use bot::Nekobot;
use clap::Parser;
use crossterm::event::{poll, read, Event, KeyCode};
use map::NystopiaMap;
use renderer::Renderer;
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = stdout();
    let prog_args = ProgArgs::parse();
    let mut render_instance = terminal::Terminal::new();
    let mut nekomap = NystopiaMap::new(
        &prog_args,
        render_instance.get_cols()?,
        render_instance.get_rows()?,
    )?;

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

    render_instance.init()?;

    let inst = Instant::now();
    let mut last_capture = inst.elapsed().as_millis();
    let mut next_stop = last_capture + period;

    loop {
        last_capture = inst.elapsed().as_millis();
        if last_capture >= next_stop {
            // Render the map
            nekomap.tick_map();
            render_instance.render_map(&nekomap)?;

            // Render the bots
            for nekobot in nekobots.iter_mut() {
                render_instance.place_tile(&nekomap, nekobot.get_row(), nekobot.get_col())?;
                nekobot.tick(&mut nekomap);
                render_instance.place_bot(&nekomap, &nekobot)?;
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

    Ok(())
}
