#![warn(clippy::pedantic)]
use shrek_deck::parser::parse_file;
use shrek_deck::tts::write_to_tts_dir;
use shrek_deck::tts::CardShape;
use shrek_deck::tts::SaveState;
use shrek_deck::GetCardInfo;
use std::path::PathBuf;

use clap::{command, Parser};

#[derive(Parser)]
#[command(version, about, long_about)]
struct Args {
    /// The path to the deck file
    #[arg(short, long)]
    input: PathBuf,
    /// The output path (will overwrite!)
    #[arg(short, long)]
    output: PathBuf,
    /// Output path is relative to Tabletop Simulator's saved objects directory. Will overwrite existing objects.
    #[arg(short, long)]
    tabletop: bool,
    /// Output should use the blood card back as thumbnail (does nothing if not using the --tabletop flag)
    #[arg(short, long)]
    flask: bool,
}

#[derive(Clone)]
struct BloodlessCard {
    name: String,
}

impl GetCardInfo for BloodlessCard {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_front_image(&self) -> Result<String, shrek_deck::CardError> {
        Ok(get_filegarden_link(self.get_name()))
    }

    fn get_back_image(&self) -> Result<String, shrek_deck::CardError> {
        Ok("https://file.garden/ZJSEzoaUL3bz8vYK/bloodlesscards/00%20back.png".to_string())
    }

    fn get_card_shape(&self) -> Result<CardShape, shrek_deck::CardError> {
        Ok(CardShape::RoundedRectangle)
    }

    fn parse(string: &str) -> Result<Self, shrek_deck::parser::ParseError> {
        Ok(BloodlessCard {
            name: string.to_owned(),
        })
    }
}

fn main() {
    let cli = Args::parse();

    match parse_file::<BloodlessCard>(&cli.input) {
        Ok(cards) => {
            let save = match SaveState::new_with_deck(cards) {
                Ok(x) => x,
                Err(x) => return eprintln!("{x}"),
            };

            let contents = match serde_json::to_string_pretty(&save) {
                Ok(x) => x,
                Err(err) => return eprintln!("{err}"),
            };

            if cli.tabletop {
                let result = if cli.flask {
                    write_to_tts_dir(cli.output, contents, include_bytes!("blood.png"))
                } else {
                    write_to_tts_dir(cli.output, contents, include_bytes!("card.png"))
                };

                match result {
                    Ok(()) => (),
                    Err(error) => eprintln!("{error}"),
                }
            } else {
                match std::fs::write(cli.output, contents) {
                    Ok(()) => (),
                    Err(error) => eprintln!("{error}"),
                }
            }
        }
        Err(errors) => {
            for x in errors {
                eprintln!("{x}");
                eprintln!();
            }
        }
    }
}

fn get_filegarden_link(name: &str) -> String {
    format!(
        "https://file.garden/ZJSEzoaUL3bz8vYK/bloodlesscards/{}.png",
        name.replace(' ', "").replace('ä', "a")
    )
}
