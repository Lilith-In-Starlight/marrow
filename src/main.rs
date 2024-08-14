#![warn(clippy::pedantic)]
use std::{
    collections::HashMap,
    fmt::Display,
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

use clap::{command, Parser};
use serde::Serialize;
use uuid::Uuid;

#[derive(Parser)]
#[command(version, about, long_about)]
struct Args {
    /// The path to the deck file
    #[arg(short, long)]
    input: PathBuf,
    /// The output path (will overwrite!)
    #[arg(short, long)]
    output: PathBuf,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct SaveState {
    save_name: String,
    date: String,
    version_number: String,
    game_mode: String,
    game_type: String,
    game_complexity: String,
    tags: Vec<String>,
    gravity: f64,
    play_area: f64,
    table: String,
    sky: String,
    note: String,
    tab_states: HashMap<String, String>,
    lua_script: String,
    lua_script_state: String,
    #[serde(rename = "XmlUI")]
    xml_ui: String,
    object_states: Vec<ObjectState>,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
#[allow(clippy::struct_excessive_bools)]
struct ObjectState {
    guid: String,
    name: String,
    transform: TransformState,
    nickname: String,
    description: String,
    #[serde(rename = "GMNotes")]
    gm_notes: String,
    alt_look_angle: Vector3,
    color_difuse: ColourState,
    layout_group_sort_index: i64,
    value: i64,
    locked: bool,
    grid: bool,
    snap: bool,
    #[serde(rename = "IgnoreFoW")]
    ignore_fow: bool,
    measure_movement: bool,
    drag_selectable: bool,
    autoraise: bool,
    sticky: bool,
    tooltip: bool,
    grid_projection: bool,
    hide_when_face_down: bool,
    hands: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    card_id: Option<i64>,
    sideways_card: bool,
    #[serde(rename = "DeckIDs")]
    #[serde(skip_serializing_if = "Option::is_none")]
    deck_ids: Option<Vec<i64>>,
    custom_deck: HashMap<i64, CustomDeckState>,
    lua_script: String,
    lua_script_state: String,
    #[serde(rename = "XmlUI")]
    xml_ui: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    contained_objects: Option<Vec<ObjectState>>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "PascalCase")]
struct CustomDeckState {
    face_url: String,
    back_url: String,
    num_width: i64,
    num_height: i64,
    back_is_hidden: bool,
    unique_back: bool,
    r#type: i64,
}

#[derive(Serialize, Default)]
struct Vector3 {
    x: f64,
    y: f64,
    z: f64,
}

#[derive(Serialize, Default)]
struct ColourState {
    r: f64,
    g: f64,
    b: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TransformState {
    pos_x: f64,
    pos_y: f64,
    pos_z: f64,
    rot_x: f64,
    rot_y: f64,
    rot_z: f64,
    scale_x: f64,
    scale_y: f64,
    scale_z: f64,
}

impl Default for TransformState {
    fn default() -> Self {
        Self {
            pos_x: 0.0,
            pos_y: 0.0,
            pos_z: 0.0,
            rot_x: 0.0,
            rot_y: 0.0,
            rot_z: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            scale_z: 1.0,
        }
    }
}

fn main() {
    let cli = Args::parse();

    let mut cards = vec![];
    let file = File::open(cli.input).unwrap();
    let mut reader = BufReader::new(file);
    let mut line_idx = 0;
    let mut errors = vec![];
    loop {
        line_idx += 1;
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => match parse_line(&line) {
                Ok(card) => cards.push(card),
                Err(error) => errors.push(AtRow {
                    column: error,
                    row: line_idx,
                }),
            },
            Err(a) => panic!("Couldn't read line {line_idx} in the file because: {a}"),
        }
    }

    if errors.is_empty() {
        let a = SaveState::new_for_deck(cards);

        let a = serde_json::to_string_pretty(&a).unwrap();

        std::fs::write(cli.output, a).unwrap();
    } else {
        for x in errors {
            eprintln!("{x}");
            eprintln!();
        }
    }
}

impl SaveState {
    fn new_for_deck(deck: Vec<(i64, CustomDeckState)>) -> SaveState {
        let (deck_ids, custom_deck, contained_objects) = generate_deck_data(deck);
        let (deck_ids, contained_objects) = (Some(deck_ids), Some(contained_objects));
        let object_state = ObjectState {
            guid: generate_guid(),
            name: "Deck".to_string(),
            transform: TransformState {
                rot_y: 180.0,
                ..Default::default()
            },
            nickname: String::new(),
            description: String::new(),
            gm_notes: String::new(),
            alt_look_angle: Vector3::default(),
            color_difuse: ColourState {
                r: 0.713_235_259,
                g: 0.713_235_259,
                b: 0.713_235_259,
            },
            layout_group_sort_index: 0,
            value: 0,
            locked: false,
            grid: true,
            snap: true,
            ignore_fow: false,
            measure_movement: false,
            drag_selectable: true,
            autoraise: true,
            sticky: true,
            tooltip: true,
            grid_projection: false,
            hide_when_face_down: true,
            hands: false,
            card_id: None,
            sideways_card: false,
            deck_ids,
            custom_deck,
            lua_script: String::new(),
            lua_script_state: String::new(),
            xml_ui: String::new(),
            contained_objects,
        };
        let object_states = vec![object_state];
        SaveState {
            save_name: String::new(),
            date: String::new(),
            version_number: String::new(),
            game_mode: String::new(),
            game_type: String::new(),
            game_complexity: String::new(),
            tags: vec![],
            gravity: 0.5,
            play_area: 0.5,
            table: String::new(),
            sky: String::new(),
            note: String::new(),
            tab_states: HashMap::new(),
            lua_script: String::new(),
            lua_script_state: String::new(),
            xml_ui: String::new(),
            object_states,
        }
    }
}

fn generate_deck_data(
    deck: Vec<(i64, CustomDeckState)>,
) -> (Vec<i64>, HashMap<i64, CustomDeckState>, Vec<ObjectState>) {
    let mut card_ids = vec![];
    let mut custom_deck = HashMap::new();
    let mut contained_objects = vec![];
    for (idx, (amt, card)) in deck
        .into_iter()
        .enumerate()
        .map(|(idx, card)| ((idx + 1) as i64, card))
    {
        let id = idx * 100;
        custom_deck.insert(idx, card.clone());
        for _ in 0..amt {
            card_ids.push(id);
            contained_objects.push(ObjectState {
                guid: generate_guid(),
                name: "CardCustom".to_string(),
                transform: TransformState::default(),
                nickname: String::new(),
                description: String::new(),
                gm_notes: String::new(),
                alt_look_angle: Vector3::default(),
                color_difuse: ColourState {
                    r: 0.713_235_259,
                    g: 0.713_235_259,
                    b: 0.713_235_259,
                },
                layout_group_sort_index: 0,
                value: 0,
                locked: false,
                grid: true,
                snap: true,
                ignore_fow: false,
                measure_movement: false,
                drag_selectable: true,
                autoraise: true,
                sticky: true,
                tooltip: true,
                grid_projection: false,
                hide_when_face_down: true,
                hands: true,
                card_id: Some(id),
                sideways_card: false,
                deck_ids: None,
                custom_deck: {
                    let mut hm = HashMap::new();
                    hm.insert(idx, card.clone());
                    hm
                },
                lua_script: String::new(),
                lua_script_state: String::new(),
                xml_ui: String::new(),
                contained_objects: None,
            });
        }
    }
    (card_ids, custom_deck, contained_objects)
}

fn generate_guid() -> String {
    Uuid::new_v4().to_string()
}

fn parse_line(string: &str) -> Result<(i64, CustomDeckState), AtColumn> {
    let mut parserstate = ParserState::Numbering;
    let mut number_str = String::new();
    let mut name = String::new();
    for (idx, char) in string.char_indices() {
        match parserstate {
            ParserState::Numbering => match char {
                ch @ ('0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9') => {
                    number_str.push(ch);
                }
                ' ' | '\t' => parserstate = ParserState::Exing,
                'x' => parserstate = ParserState::Naming,
                a => {
                    return Err(AtColumn {
                        error: Error::UnexpectedChar {
                            obtained: a,
                            expected: vec![
                                "`x` (number separator)".to_string(),
                                "a number".to_string(),
                                "a space".to_string(),
                                "tab".to_string(),
                            ],
                        },
                        column: idx + 1,
                    })
                }
            },
            ParserState::Exing => match char {
                ' ' | '\t' => continue,
                'x' => parserstate = ParserState::Naming,
                a => {
                    return Err(AtColumn {
                        error: Error::UnexpectedChar {
                            obtained: a,
                            expected: vec![
                                "`x` (number separator)".to_string(),
                                "space".to_string(),
                                "tab".to_string(),
                            ],
                        },
                        column: idx + 1,
                    })
                }
            },
            ParserState::Naming => name.push(char),
        }
    }
    let name = name.trim().to_owned();

    let card_data = CustomDeckState {
        face_url: get_filegarden_link(&name),
        back_url: "https://file.garden/ZJSEzoaUL3bz8vYK/bloodlesscards/00%20back.png".to_string(),
        num_width: 1,
        num_height: 1,
        back_is_hidden: true,
        unique_back: false,
        r#type: 0,
    };

    Ok((
        number_str.parse().expect("couldn't parse number"),
        card_data,
    ))
}

enum ParserState {
    Numbering,
    Naming,
    Exing,
}

fn get_filegarden_link(name: &str) -> String {
    format!(
        "https://file.garden/ZJSEzoaUL3bz8vYK/bloodlesscards/{}.png",
        name.replace(' ', "").replace('ä', "a")
    )
}

enum Error {
    UnexpectedChar {
        obtained: char,
        expected: Vec<String>,
    },
}

struct AtColumn {
    error: Error,
    column: usize,
}

struct AtRow {
    column: AtColumn,
    row: usize,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::UnexpectedChar { obtained, expected } => {
                write!(
                    f,
                    "\n Obtained character `{obtained}`, expected one of the following: "
                )?;

                for expected in expected {
                    write!(f, "\n - {expected}")?;
                }

                Ok(())
            }
        }
    }
}

impl Display for AtRow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Error at line {}, column {}: {}",
            self.column.column, self.row, self.column.error
        )
    }
}

#[cfg(target_os = "windows")]
fn get_tts_dir() -> Option<PathBuf> {
    let mut dir = dirs::home_dir();
    if let Some(dir) = dir.as_mut() {
        dir.push("Documents\\My Games\\Tabletop Simulator\\Saves\\Saved Objects");
    }
    dir
}

#[cfg(target_os = "macos")]
fn get_tts_dir() -> Option<PathBuf> {
    let mut dir = dirs::home_dir();
    if let Some(dir) = dir.as_mut() {
        dir.push("Library/Tabletop Simulator/Saves/Saved Objects");
    }
    dir
}

#[cfg(target_os = "linux")]
fn get_tts_dir() -> Option<PathBuf> {
    let mut dir = dirs::home_dir();
    if let Some(dir) = dir.as_mut() {
        dir.push(".local/share/Tabletop Simulator/Saves/Saved Objects");
    }
    dir
}
