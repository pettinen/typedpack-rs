use std::{
    io::Read as _,
    path::{Path, PathBuf},
    process::ExitCode,
};

use clap::{Arg, ArgAction, Command, builder::PathBufValueParser};

use typedpack_codegen::{
    ParseError, Type,
    typescript::{FILE_HEADER, Options, OptionsInput},
};

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("failed to create output directory: {0}")]
    CreateDir(std::io::Error),
    #[error("when the input path is a directory, the output path must be as well")]
    OutputNotDir,
    #[error("error parsing file {0}:\n{1}")]
    Parse(String, ParseError),
    #[error("failed to read directory: {0}")]
    ReadDir(std::io::Error),
    #[error("failed to read file: {0}")]
    ReadFile(std::io::Error),
    #[error("failed to read from standard input: {0}")]
    ReadStdin(std::io::Error),
    #[error("failed to write output file: {0}")]
    Write(std::io::Error),
}

// typedpack-cli DIR OUT_DIR
// typedpack-cli FILE OUT_FILE
// typedpack-cli - OUT_FILE
// typedpack-cli FILE -
// typedpack-cli - -

fn main() -> ExitCode {
    let mut matches = Command::new(clap::crate_name!())
        .version(clap::crate_version!())
        .after_help(
            "\
<OUTPUT> must be a directory (or not exist, in which case a directory
will be created) if and only if <INPUT> is also a directory.
In this case, code from all files with the '.tp' extension under <INPUT>
will be generated into an identical directory structure under <OUTPUT>,
with the file extensions replaced with '.ts'.",
        )
        .arg(
            Arg::new("input")
                .help("Input file or directory. Use '-' to read from standard input.")
                .required(true)
                .value_name("INPUT")
                .value_parser(PathBufValueParser::new()),
        )
        .arg(
            Arg::new("output")
                .help("Output file or directory. Use '-' to print to standard output.")
                .required(true)
                .value_name("OUTPUT")
                .value_parser(PathBufValueParser::new()),
        )
        .arg(
            Arg::new("types_namespace")
                .help("Change the name of the namespace containing the types (default `Types`).")
                .long("types-namespace")
                .value_name("NAME")
        )
        .arg(
            Arg::new("encode_namespace")
                .help("Change the name of the namespace containing the encoding functions (default `Encode`).")
                .long("encode-namespace")
                .value_name("NAME")
        )
        .arg(
            Arg::new("decode_namespace")
                .help("Change the name of the namespace containing the decoding functions (default `Decode`).")
                .long("decode-namespace")
                .value_name("NAME")
        )
        .arg(
            Arg::new("encode_array_namespace")
                .help("Change the name of the namespace containing the array encoding functions (default `EncodeArray`).")
                .long("encode-array-namespace")
                .value_name("NAME")
        )
        .arg(
            Arg::new("decode_array_namespace")
                .help("Change the name of the namespace containing the array decoding functions (default `DecodeArray`).")
                .long("decode-array-namespace")
                .value_name("NAME")
        )
        .arg(
            Arg::new("export_decode_internal_namespace")
                .action(ArgAction::SetTrue)
                .help("Export the `TypedpackDecodeInternal` namespace. This is primarily meant for testing.")
                .long("export-decode-internal-namespace")
        )
        .get_matches();

    let input: PathBuf = matches.remove_one("input").expect("required argument");
    let output: PathBuf = matches.remove_one("output").expect("required argument");

    let mut options = OptionsInput::default();
    if let Some(types_namespace) = matches.remove_one("types_namespace") {
        options.types_namespace = Some(types_namespace);
    }
    if let Some(encode_namespace) = matches.remove_one("encode_namespace") {
        options.encode_namespace = Some(encode_namespace);
    }
    if let Some(decode_namespace) = matches.remove_one("decode_namespace") {
        options.decode_namespace = Some(decode_namespace);
    }
    if let Some(encode_array_namespace) = matches.remove_one("encode_array_namespace") {
        options.encode_array_namespace = Some(encode_array_namespace);
    }
    if let Some(decode_array_namespace) = matches.remove_one("decode_array_namespace") {
        options.decode_array_namespace = Some(decode_array_namespace);
    }
    if matches.get_flag("export_decode_internal_namespace") {
        options.export_decode_internal_namespace = Some(true);
    }
    let options = match options.try_into() {
        Ok(options) => options,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::FAILURE;
        }
    };

    if let Err(error) = run(&input, &output, &options) {
        eprintln!("{error}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn run(input: &Path, output: &Path, options: &Options) -> Result<(), Error> {
    if input != "-" && input.is_dir() {
        if output == "-" || output.is_file() {
            return Err(Error::OutputNotDir);
        }

        for input_path in scan_dir(input)? {
            let path_suffix = input_path
                .strip_prefix(input)
                .expect("not a file path inside `input` directory")
                .with_extension("ts");
            write_single_file(&input_path, &output.join(path_suffix), true, options)?;
        }
    } else {
        write_single_file(input, output, false, options)?;
    }
    Ok(())
}

fn scan_dir(root: &Path) -> Result<Vec<PathBuf>, Error> {
    let mut paths = Vec::new();
    for entry in root.read_dir().map_err(Error::ReadDir)? {
        let path = entry.map_err(Error::ReadDir)?.path();
        if path.is_dir() {
            paths.extend(scan_dir(&path)?);
        } else if path.extension() == Some("tp".as_ref()) && path.is_file() {
            paths.push(path);
        }
    }
    Ok(paths)
}

fn write_single_file(
    input: &Path,
    output: &Path,
    create_dir: bool,
    options: &Options,
) -> Result<(), Error> {
    if create_dir {
        std::fs::create_dir_all(output.parent().expect("not a file path inside a directory"))
            .map_err(Error::CreateDir)?;
    }

    let input_string = if input == "-" {
        let mut string = String::new();
        std::io::stdin()
            .read_to_string(&mut string)
            .map_err(Error::ReadStdin)?;
        string
    } else {
        std::fs::read_to_string(input).map_err(Error::ReadFile)?
    };

    let types = typedpack_codegen::parse(&input_string)
        .map_err(|error| Error::Parse(input.to_string_lossy().to_string(), error))?;

    let output_string = if types.is_empty() {
        let mut s = String::from("export namespace ");
        s.push_str(options.types_namespace());
        s.push_str(" {}\nexport namespace ");
        s.push_str(options.encode_namespace());
        s.push_str(" {}\nexport namespace ");
        if options.export_decode_internal_namespace() {
            s.push_str("TypedpackDecodeInternal {}\nexport namespace ");
        }
        s.push_str(options.decode_namespace());
        s.push_str(" {}\nexport namespace ");
        s.push_str(options.encode_array_namespace());
        s.push_str(" {}\nexport namespace ");
        s.push_str(options.decode_array_namespace());
        s.push_str("{}\n");
        s
    } else {
        let mut output_string = String::from(FILE_HEADER);
        for r#type in types {
            output_string.push_str("\n\n");
            match r#type {
                Type::Enum(r#enum) => {
                    output_string.push_str(&r#enum.typescript_enum(options));
                }
                Type::Struct(r#struct) => {
                    output_string.push_str(&r#struct.typescript_interface(options));
                }
            }
        }
        output_string.push('\n');
        output_string
    };

    if output == "-" {
        print!("{output_string}");
    } else {
        std::fs::write(output, output_string).map_err(Error::Write)?;
    }
    Ok(())
}
