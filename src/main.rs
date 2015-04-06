#![feature(rustc_private)]
extern crate syntax;

use std::env;
use std::fs;
use std::fs::File;
use std::path::Path;
use std::io::Error;
use std::io::prelude::*;

mod typesetting;
mod token_handling;

use typesetting::Typesetter;

fn format_file(filename: &str) -> Result < (),
Error > {
    let mut f = try!(File::open(filename));
    let mut source = String::new();
    try!(f.read_to_string(&mut source));

    f = try!(File::create(filename));

    let typesetter = Typesetter::new(source.as_ref());
    try!(f.write_all(typesetter.to_string().as_bytes()));
    Ok(())
}

pub fn main() {

    let mut args:Vec < String > = env::args().collect();
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

/*
//#[test]
fn test_cases() {
    let paths_to_test = fs::read_dir(&Path::new("tests")).unwrap();

    for path_to_test in paths_to_test {
        let mut filename_pre = path_to_test.clone().unwrap().path();
        filename_pre.push("pre_format");
        filename_pre.set_extension("rs");
        let mut filename_post = path_to_test.clone().unwrap().path();
        filename_post.push("post_format");
        filename_post.set_extension("rs");

        let mut f_pre = File::open(filename_pre).unwrap();
        let mut source_pre = String::new();
        f_pre.read_to_string(&mut source_pre).unwrap();

        let mut f_post = File::open(filename_post).unwrap();
        let mut source_post = String::new();
        f_post.read_to_string(&mut source_post).unwrap();

        println!("{:?}, {:?}", source_pre, source_post);

    }
}*/