extern crate encoding;
extern crate xml;
extern crate xlsx;

#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate toml;

extern crate glob;

use std::env;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

use encoding::{Encoding, DecoderTrap};
use encoding::all::WINDOWS_1251;

use xml::reader::{EventReader, XmlEvent};

use xlsx::workbook::Workbook;

use glob::glob;

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
    match detect_xml_file() {
        Some(filename) => {
            let filename = &filename.to_str().unwrap();
            println!("File detected: {}", filename);

            let config = fetch_config();
            // println!("{:#?}", config);

            let offers = fetch_offers(filename);
            // println!("{:#?}", offers[0]);

            generate_xlsx(&offers, &config);
        }
        None => {
            println!("No XML file found. Put it in this folder and try again.");
        }
    }
}

fn detect_xml_file() -> Option<PathBuf> {
    match glob("*.xml").unwrap().nth(0) {
        Some(file) => file.ok(),
        _ => None
    }
}

fn fetch_offers(filename: &str) -> Vec<HashMap<String, String>> {
    let mut file = File::open(filename).unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer);

    let content = WINDOWS_1251.decode(&buffer, DecoderTrap::Strict).ok().unwrap();
    let parser = EventReader::new(content.as_bytes());

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
                        offer.insert(current_node.clone().to_lowercase(), s.clone());
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
    let mut file = File::open("config.toml").unwrap();
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
