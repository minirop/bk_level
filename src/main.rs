use clap::{ Parser, ValueEnum };
use std::path::Path;

mod types;

mod setupfile;
use setupfile::SetupFile;

mod model;
use model::Model;

/// Convert models and level setup files
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
    Model,
    Setup,
    Yaml,
}

fn main() {
    let args = Args::parse();
    let filename = &args.filename;
    let output_name = Path::new(&args.filename);
    let output_name = output_name.file_stem().unwrap();
    let output_name = output_name.to_str().unwrap();

    let input = if let Some(input) = args.input {
        input
    } else if filename.ends_with(".lvl_setup.bin") {
        InputFormat::Setup
    } else if filename.ends_with(".model.bin") {
        InputFormat::Model
    } else if filename.ends_with(".yaml") {
        InputFormat::Yaml
    } else {
        panic!("Can't detect the format. Rename the file to .model.bin/.lvl_setup.bin or use the --input argument.");
    };

    match input {
        InputFormat::Setup => {
            if let Some(format) = args.output {
                if format != OutputFormat::Yaml {
                    panic!("level setup files can only be converted to YAML.");
                }
            }

            let output_name = format!("{}.yaml", output_name);
            match SetupFile::read_bin(filename) {
                Ok(file) => file.write_yaml(&output_name),
                Err(e) => panic!("{:?}", e)
            };
        },
        InputFormat::Model => {
            match Model::read_bin(filename) {
                Ok(model) => {
                    let format = if let Some(format) = args.output { format } else { OutputFormat::Yaml };
                    match format {
                        OutputFormat::Yaml => {
                            let output_name = format!("{}.yaml", output_name);
                            model.write_yaml(&output_name);
                        },
                        OutputFormat::Obj => {
                            std::fs::create_dir_all(output_name).unwrap();
                            //model.write_obj(&output_name).unwrap();
                            Model::read_bin_obj(filename).unwrap();
                        },
                        OutputFormat::Bin => panic!("Why would you want to convert .bin to .bin?"),
                    };
                },
                Err(e) => panic!("{:?}", e),
            };
        },
        InputFormat::Yaml => {
            if let Some(setupfile) = SetupFile::read_yaml(filename) {
                let format = if let Some(format) = args.output { format } else { OutputFormat::Bin };
                match format {
                    OutputFormat::Bin => {
                        let output_name = format!("{}_repack.bin", output_name);
                        setupfile.write_bin(&output_name).unwrap();
                    },
                    OutputFormat::Obj => panic!("Can't convert setup file to .obj"),
                    OutputFormat::Yaml => panic!("Why would you want to convert .yaml to .yaml?"),
                };
            } else if let Some(mut model) = Model::read_yaml(filename) {
                let format = if let Some(format) = args.output { format } else { OutputFormat::Bin };
                match format {
                    OutputFormat::Bin => {
                        let output_name = format!("{}_repack.bin", output_name);
                        model.write_bin(&output_name).unwrap();
                    },
                    OutputFormat::Obj => {
                        model.write_obj(&output_name).unwrap();
                    },
                    OutputFormat::Yaml => panic!("Why would you want to convert .yaml to .yaml?"),
                };
            } else {
                panic!("{} is not a valid YAML file.", filename);
            }
        },
    };
}
