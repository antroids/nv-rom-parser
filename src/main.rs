// SPDX-License-Identifier: MIT

use clap::{Parser, ValueEnum};
use nv_rom_parser::firmware::FirmwareBundleInfo;
use std::fs::File;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    rom_file: PathBuf,

    #[arg(short, long, value_enum, default_value_t = Command::VBios)]
    command: Command,

    #[arg(short, long, value_enum, default_value_t = Output::Debug)]
    output: Output,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Command {
    VBios,
    Full,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Output {
    Debug,
    Json,
}

pub fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    let args = Args::parse();
    let mut file = File::open(&args.rom_file)
        .expect(format!("Cannot open ROM file at {:?}", args.rom_file).as_str());
    let firmware_bundle_info = FirmwareBundleInfo::parse(&mut file).unwrap();

    match &args.command {
        Command::VBios => match &args.output {
            Output::Debug => {
                println!("{:#?}", firmware_bundle_info.v_bios_info());
            }
            Output::Json => {
                println!("{}", serde_json::to_string_pretty(&firmware_bundle_info.v_bios_info()).expect("Cannot serialize firmware bundle info into JSON, try another output format"));
            }
        },
        Command::Full => match &args.output {
            Output::Debug => {
                println!("{:#?}", firmware_bundle_info);
            }
            Output::Json => {
                println!("{}", serde_json::to_string_pretty(&firmware_bundle_info).expect("Cannot serialize firmware bundle info into JSON, try another output format"));
            }
        },
    }
}
