extern crate xml;
extern crate encoding;

use std::env;
use std::fs::File;
use std::io::Read;
use std::collections::HashMap;

use encoding::{Encoding, DecoderTrap};
use encoding::all::WINDOWS_1251;

use xml::reader::{EventReader, XmlEvent};

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];

    let offers = fetch_offers(filename);

    println!("{:#?}", offers[0]);
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
