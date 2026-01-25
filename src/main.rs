#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

use shrek_deck::parser::parse_file;
use shrek_deck::tts;
use shrek_deck::tts::write_to_tts_dir;
use shrek_deck::tts::CardShape;
use shrek_deck::tts::SaveState;
use shrek_deck::GetCardInfo;
use std::path::Path;
use std::path::PathBuf;
use walkdir::WalkDir;

use clap::{command, Parser};

#[derive(Parser)]
#[command(version, about, long_about)]
struct Args {
    /// The path to the deck file or a directory of deck files.
    ///
    /// If it's a directory, it will traverse the entire tree looking for deck files. Deck files have the .marrow or .mflask extension.
    #[arg(short, long)]
    input: PathBuf,
    /// The output path (will overwrite!)
    ///
    /// Output files always have their extension set to .json.
    ///
    /// If `input` is a directory, `output` must be a directory as well.
    #[arg(short, long)]
    output: PathBuf,
    /// Treat output path as relative to Tabletop Simulator's saved objects directory. Will overwrite existing objects.
    #[arg(short, long)]
    tabletop: bool,
    /// Output should use the blood card back as thumbnail (does nothing if not using the --tabletop flag). If compiling a directory, use the .mflask extension as a replacement for this flag.
    #[arg(short, long)]
    flask: bool,
}

#[derive(Clone)]
struct BloodlessCard {
    name: String,
    back: String,
}

impl GetCardInfo for BloodlessCard {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_front_image(&self) -> Result<String, shrek_deck::CardError> {
        Ok(get_filegarden_link(self.get_name()))
    }

    fn get_back_image(&self) -> Result<String, shrek_deck::CardError> {
        Ok(self.back.clone())
    }

    fn get_card_shape(&self) -> Result<CardShape, shrek_deck::CardError> {
        Ok(CardShape::RoundedRectangle)
    }

    fn parse(string: &str) -> Result<Self, shrek_deck::parser::ParseError> {
        Ok(Self {
            name: string.to_owned(),
            back: "https://file.garden/ZJSEzoaUL3bz8vYK/bloodlesscards/00%20back.png".to_string(),
        })
    }
}

fn main() {
    let cli = Args::parse();

    if cli.input.is_file() {
        file_input(cli);
    } else if cli.input.is_dir() {
        dir_input(&cli);
    }
}

fn get_filegarden_link(name: &str) -> String {
    format!(
        "https://hemolymph.net/cardimgs/{}.png",
        name.replace(' ', "").replace('ä', "a")
    )
}

fn dir_input(cli: &Args) {
    for entry in WalkDir::new(&cli.input).follow_links(true) {
        let entry = match entry {
            Ok(entry) => entry,
            Err(err) => {
                eprintln!("Error: {err}");
                continue;
            }
        };

        let ext = entry.path().extension();

        if ext.is_none_or(|x| x != "marrow" && x != "mflask") {
            continue;
        }

        let flask = ext.is_some_and(|x| x == "mflask");

        let mut components = entry.path().components();

        // Remove root dir of this path
        for _ in cli.input.components() {
            components.next();
        }

        let rest = components.as_path();

        let mut output = cli.output.clone();

        output.push(rest);

        println!("{}", output.display());

        file_input(Args {
            input: entry.path().to_path_buf(),
            output,
            tabletop: cli.tabletop,
            flask,
        });
    }
}

fn file_input(cli: Args) {
    let mut cards = match parse_file::<BloodlessCard>(&cli.input) {
        Ok(cards) => cards,
        Err(errors) => {
            for x in errors {
                eprintln!("{x}");
                eprintln!();
            }
            return;
        }
    };

    if cli.flask {
        for card in &mut cards {
            card.card.back =
                "https://file.garden/ZJSEzoaUL3bz8vYK/bloodlesscards/flaskvack.png".to_string();
        }
    }
    let save = match SaveState::new_with_deck(cards) {
        Ok(x) => x,
        Err(x) => return eprintln!("{x}"),
    };

    let contents = match serde_json::to_string_pretty(&save) {
        Ok(x) => x,
        Err(err) => return eprintln!("{err}"),
    };

    if cli.tabletop {
        create_all_needed_folders(&cli.output);
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
        let output = cli.output.with_extension("json");
        match std::fs::write(output, contents) {
            Ok(()) => (),
            Err(error) => eprintln!("{error}"),
        }
    }
}

fn create_all_needed_folders(path: &Path) {
    let path = if path.is_dir() {
        path
    } else {
        path.parent().unwrap()
    };
    let Some(mut saved_object) = tts::get_saved_objects_dir() else {
        panic!("Failed to find saved objects dir")
    };
    saved_object.push(path);
    std::fs::create_dir_all(&saved_object)
        .expect("Failed to create folders needed for output path");
}
