use clap::Parser as _;
use clap_file::{Input, Output};
use nbt::binary::Endianness;
use nbt::binary::read::ReadNbt;
use nbt::binary::write::WriteNbt;
use std::io::{Read, Write};

#[derive(clap::Parser)]
#[command(version, about)]
struct Arguments {
    #[command(subcommand)]
    command: Subcommand,
}

#[derive(clap::Subcommand)]
enum Subcommand {
    NbtToSnbt {
        #[arg(value_name = "NBT_FILE")]
        input: Input,
        #[arg(short, long, default_value = "-", value_name = "SNBT_FILE")]
        output: Output,
        #[arg(long)]
        header: bool,
    },
    SnbtToNbt {
        #[arg(value_name = "SNBT_FILE")]
        input: Input,
        #[arg(short, long, default_value = "-", value_name = "NBT_FILE")]
        output: Output,
        #[arg(long)]
        header: bool,
        #[arg(long, default_value = "10")]
        header_version: u32,
    },
}

fn main() {
    let arguments = Arguments::parse();
    match arguments.command {
        Subcommand::NbtToSnbt {
            mut input,
            mut output,
            header,
        } => {
            if header {
                let header = nbt::BedrockHeader::read_nbt(&mut input, Endianness::Little);
                eprintln!("{header:?}");
            };

            let nbt::NamedTag(key, value) =
                nbt::NamedTag::read_nbt(&mut input, Endianness::Little).unwrap();
            if key != "" {
                todo!()
            }

            output.write_all(value.to_string().as_bytes()).unwrap();
            output.flush().unwrap();
        }
        Subcommand::SnbtToNbt {
            mut input,
            mut output,
            header,
            header_version,
        } => {
            let mut source = String::new();
            input.read_to_string(&mut source).unwrap();
            let parser = nbt::snbt::read::parse::Parser::new(&mut source);
            let nbt = parser.parse_variant_and_finish().unwrap();
            let nbt = nbt::Variant::try_from(nbt).unwrap();
            let nbt = nbt::NamedTag(String::new(), nbt);

            if header {
                let mut payload = Vec::new();
                nbt.write_nbt(&mut payload, Endianness::Little).unwrap();
                let payload = payload;
                let header = nbt::BedrockHeader {
                    version: header_version,
                    size: payload.len().try_into().unwrap(),
                };

                header.write_nbt(&mut output, Endianness::Little).unwrap();
                output.write_all(&payload).unwrap();
                output.flush().unwrap();
            } else {
                nbt.write_nbt(&mut output, Endianness::Little).unwrap();
                output.flush().unwrap();
            }
        }
    }
}
