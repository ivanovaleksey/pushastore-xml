extern crate encoding;
extern crate xml;
extern crate xlsx;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate toml;

extern crate clap;

mod converter;

fn main() {
    converter::call();
}
