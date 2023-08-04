#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(unused_assignments)]

use std::collections::HashSet;
use image::RgbaImage;
use std::env::args;
use byteorder::{ReadBytesExt, BigEndian};
use clap::{ Parser, ValueEnum };
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

    /// Input format 
    #[arg(short, long)]
    input: Option<InputFormat>,

    /// Output format 
    #[arg(short, long)]
    output: Option<OutputFormat>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum OutputFormat {
    Yaml,
    Obj,
    Bin,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum InputFormat {
    Level,
    Setup,
    Yaml,
}

fn main() {
    let args = Args::parse();
    let filename = &args.filename;
    let output = Path::new(&args.filename);
    let output = output.file_stem().unwrap();
    let output = output.to_str().unwrap();

    let input = if let Some(input) = args.input {
        input
    } else if filename.ends_with(".lvl_setup.bin") {
        InputFormat::Setup
    } else if filename.ends_with(".lvl.bin") {
        InputFormat::Level
    } else if filename.ends_with(".yaml") {
        InputFormat::Yaml
    } else {
        panic!("Can't detect the format. Rename the file to .lvl.bin/.lvl_setup.bin or use the --input argument.");
    };

    match input {
        InputFormat::Setup => {
            if let Some(format) = args.output {
                if format != OutputFormat::Yaml {
                    panic!("level setup files can only be converted to YAML.");
                }
            }

            let output = format!("{}.yaml", output);
            match SetupFile::read_bin(filename) {
                Ok(file) => file.write_yaml(&output),
                Err(e) => panic!("{:?}", e)
            };
        },
        InputFormat::Level => {
            match Level::read_bin(filename) {
                Ok(file) => {
                    let format = if let Some(format) = args.output { format } else { OutputFormat::Obj };
                    match format {
                        OutputFormat::Yaml => {
                            let output = format!("{}.yaml", output);
                            println!("LVL YAML");
                        },
                        OutputFormat::Obj => { println!("LVL OBJ"); },
                        OutputFormat::Bin => panic!("Why would you want to convert .bin to .bin?"),
                    };
                },
                Err(e) => panic!("{:?}", e),
            };
        },
        InputFormat::Yaml => {
            if let Some(setupfile) = SetupFile::read_yaml(filename) {
                let output = format!("{}_repack.bin", output);
                setupfile.write_bin(&output).unwrap();
            } else if let Some(level) = Level::read_yaml(filename) {
                let format = if let Some(format) = args.output { format } else { OutputFormat::Bin };
                match format {
                    OutputFormat::Bin => {
                        let output = format!("{}_repack.bin", output);
                        level.write_bin(&output).unwrap();
                    },
                    OutputFormat::Obj => {
                        level.write_obj(&output).unwrap();
                    },
                    OutputFormat::Yaml => panic!("Why would you want to convert YAML to YAML?"),
                };
            } else {
                panic!("{} is not a valid YAML file.", filename);
            }
        },
    };
}
