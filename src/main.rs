#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(unused_assignments)]

use std::collections::HashSet;
use image::RgbaImage;
use std::env::args;
use byteorder::{ReadBytesExt, BigEndian};
use clap::Parser;
use serde::{ Serialize, Deserialize };
use std::path::Path;
use std::fs::File;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Read;
use std::io::Write;

mod setupfile;
use setupfile::SetupFile;

mod level;
use level::Level;

/// Convert levels and level setup files
#[derive(Parser, Debug)]
#[command(author = None, version = None, about = None, long_about = None)]
struct Args {
    /// File to read
    filename: String,
}

fn main() {
    let args = Args::parse();
    let filename = &args.filename;
    let output = Path::new(&args.filename);
    let output = output.file_stem().unwrap();
    let output = output.to_str().unwrap();

    if filename.ends_with(".lvl_setup.bin") {
        let output = format!("{}.yaml", output);
        match SetupFile::read_bin(filename) {
            Ok(file) => file.write_yaml(&output),
            Err(e) => panic!("{:?}", e)
        };
    } else if filename.ends_with(".yaml") {
        SetupFile::read_yaml(filename).write_bin(filename);
    } else if filename.ends_with(".lvl.bin") {
        let output = format!("{}.yaml", output);
        match Level::read_bin(filename) {
            Ok(file) => file.write_yaml(&output),
            Err(e) => panic!("{:?}", e)
        };
    } else {
        panic!("Filename must ends with '.lvl_setup.bin', '.lvl.bin' or '.yaml'. Got '{}'.", filename);
    }
}
