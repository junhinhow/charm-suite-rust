//! Demonstrates the `charm-spinner` animations.
//!
//! ```text
//! cargo run -p charm-spinner --example demo            # animates "Dots"
//! cargo run -p charm-spinner --example demo -- Rocket   # animates "Rocket"
//! cargo run -p charm-spinner --example demo -- list     # lists every spinner
//! ```

use std::io::{self, Write as _};
use std::thread::sleep;

use charm_spinner::all;

/// Number of complete cycles played for each animation.
const CYCLES: usize = 12;

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let spinners = all();

    match args.first().map(String::as_str) {
        Some("list") => {
            for (name, spinner) in &spinners {
                println!(
                    "{name:<10} {:>4} ms  {}",
                    spinner.interval.as_millis(),
                    spinner.frames.join(" ")
                );
            }
            Ok(())
        }
        Some("all") => {
            for (name, spinner) in &spinners {
                println!("== {name} ==");
                play(spinner)?;
            }
            Ok(())
        }
        Some(name) => {
            if let Some(spinner) = spinners.get(name) {
                play(spinner)
            } else {
                eprintln!("spinner desconhecido: {name}");
                eprintln!("use `list` para ver os disponiveis");
                Ok(())
            }
        }
        None => {
            let spinner = spinners.get("Dots").expect("o spinner Dots sempre existe");
            play(spinner)
        }
    }
}

/// Plays one animation on the current line.
fn play(spinner: &charm_spinner::Spinner) -> io::Result<()> {
    let mut stdout = io::stdout();
    let total = spinner.frames.len() * CYCLES;
    for index in 0..total {
        write!(stdout, "\r{} carregando...", spinner.render(index))?;
        stdout.flush()?;
        sleep(spinner.interval);
    }
    writeln!(stdout)?;
    Ok(())
}
