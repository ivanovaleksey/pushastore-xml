extern crate encoding;
extern crate xml;
extern crate xlsx;

#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate toml;

use std::env;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::collections::HashMap;

use xml::reader::{EventReader, XmlEvent};

use xlsx::workbook::Workbook;

#[derive(Deserialize, Debug)]
struct Config {
    columns: Vec<Column>
}

#[derive(Deserialize, Debug)]
struct Column {
    name: String,
    keys: Vec<String>
}

fn main() {
    match detect_file() {
        Some(filename) => {
            let filename = &filename.to_str().unwrap();
            println!("Detected file: {}", filename);

            let config = fetch_config();
            // println!("{:#?}", config);

            let offers = fetch_offers(filename);
            // println!("{:#?}", offers[0]);

            generate_xlsx(&offers, &config);
        }
        None => {
            println!("No XML or YML file found. Put it in this folder and try again.");
        }
    }
}

fn detect_file() -> Option<PathBuf> {
    fs::read_dir(".").unwrap()
        .map(|elem| elem.unwrap().path() )
        .find(|path_buf| {
            let path_str = format!("{}", path_buf.display());
            path_str.ends_with(".xml") || path_str.ends_with(".yml")
        })
}

fn decode(input: &[u8]) -> String {
    use encoding::{Encoding, DecoderTrap};
    use encoding::all::{WINDOWS_1251, UTF_8};

    let trap = DecoderTrap::Strict;

    if input.starts_with(&[0xEF, 0xBB, 0xBF]) {
        println!("Detected encoding: UTF-8 with BOM");
        UTF_8.decode(&input[3..], trap).ok().unwrap()
    } else {
        match UTF_8.decode(&input, trap) {
            Ok(utf_data) => {
                println!("Detected encoding: UTF-8");
                utf_data
            },
            Err(_) => {
                println!("Detected encoding: WINDOWS-1251");
                WINDOWS_1251.decode(&input, trap).unwrap()
            }
        }
    }
}

fn fetch_offers(filename: &str) -> Vec<HashMap<String, String>> {
    let mut file = fs::File::open(filename).unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer);

    let content = decode(&buffer);
    let parser  = EventReader::new(content.as_bytes());

    let mut inside_offer_node = false;
    let mut current_node = String::from("");

    let mut offers: Vec<HashMap<String, String>> = vec![];
    let mut offer = HashMap::new();

    for e in parser {
        match e {
            Ok(e) => match e {
                XmlEvent::StartElement { name, attributes, .. } => {
                    match name.local_name.as_ref() {
                        "offer" => {
                            inside_offer_node = true;
                            offer = HashMap::new();
                        },
                        "param" => {
                            if inside_offer_node {
                                let name_attribute = attributes.iter().find(|&attr| attr.name.local_name == "name").unwrap();
                                current_node = name_attribute.value.clone();
                            }
                        },
                        _ => {
                            if inside_offer_node {
                                current_node = name.local_name.clone();
                            }
                        }
                    }
                }
                XmlEvent::EndElement { name } => {
                    match name.local_name.as_ref() {
                        "offer" => {
                            inside_offer_node = false;
                            offers.push(offer.clone());
                        },
                        _ => {}
                    }
                }
                XmlEvent::Characters(s) => {
                    if inside_offer_node {
                        let node  = current_node.clone().to_lowercase();
                        let value = match node.as_ref() {
                            "picture" => match offer.get("picture") {
                                Some(current_value) => format!("{};;;{}", current_value, s),
                                None => s,
                            },
                            _ => s
                        };

                        offer.insert(node, value);
                    }
                }
                _ => {}
            },
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
        }
    }

    offers
}

fn fetch_config() -> Config {
    let mut file = fs::File::open("config.toml").unwrap();
    let mut content = String::from("");
    file.read_to_string(&mut content);

    toml::from_str(&content).unwrap()
}

fn generate_xlsx(offers: &Vec<HashMap<String, String>>, config: &Config) {
    let mut w = Workbook::new("", "Rust", true);
    w.initialize();
    let mut s = w.new_worksheet("Sheet 1", 2);

    // Headers
    for column in config.columns.iter() {
        s.cell_txt(w.value(&column.name));
    }
    s.row();

    // Empty row
    s.row();

    for offer in offers.iter() {

        for column in config.columns.iter() {
            let mut has_value = false;

            // TODO: refactor with Iter
            for key in column.keys.iter() {
                match offer.get(&key.to_lowercase()) {
                    Some(value) => {
                        s.cell_txt(w.value(value));
                        has_value = true;
                        break;
                    },
                    None => {}
                }
            }

            if !has_value {
                s.cell_txt(w.value(""));
            }
        }

        s.row();
    }

    s.flush();
    w.flush();

    w.xlsx("test.xlsx");
}
