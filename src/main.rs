#![feature(rustc_private)]
extern crate syntax;
use std::env;
use std::fs::File;
use std::io::Error;
use std::io::prelude::*;

mod typesetting;
mod token_handling;

use typesetting::Typesetter;

fn format_file(filename: &str) -> Result<(), Error> {
    let mut f = try!(File::open(filename));
    let mut source = String::new();
    try!(f.read_to_string(&mut source));

    f = try!(File::create(filename));

    let typesetter = Typesetter::new(source.as_ref());
    try!(f.write_all(typesetter.to_string().as_bytes()));
    Ok(())
}

pub fn main() {
    let mut args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} [one or more rust files]", args.first().unwrap());
        return;
    }
    args.remove(0);

    for filename in args {
        match format_file(filename.as_ref()) {
            Err(e) => {
                println!("{:?}", e);
                return;
            },
            Ok(_) => {},
        }
    }
}