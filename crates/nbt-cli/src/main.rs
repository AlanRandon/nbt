use clap::Parser as _;
use clap_file::{Input, Output};
use nbt::binary::write::Writeable as _;
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
            let nbt = if header {
                nbt::BedrockNbtFile::read_le_with_header(&mut input)
            } else {
                nbt::BedrockNbtFile::read_le_without_header(&mut input)
            }
            .unwrap();

            let nbt::NamedTag(key, value) = nbt.tag;
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
        } => {
            let mut source = String::new();
            input.read_to_string(&mut source).unwrap();
            let parser = nbt::snbt::read::parse::Parser::new(&mut source);
            let nbt = parser.parse_variant_and_finish().unwrap();
            let nbt = nbt::Variant::try_from(nbt).unwrap();

            let file = nbt::BedrockNbtFile {
                header: header.then_some(nbt::BedrockHeader {
                    version: 8,
                    size: 0,
                }),
                tag: nbt::NamedTag(String::new(), nbt),
            };

            file.write_le(&mut output).unwrap();
            output.flush().unwrap();
        }
    }
}
