use std::{error::Error, fmt};

pub fn parse_args(args: Vec<String>) -> Result<(), ArgumentError> {
    if args.len() == 2 {
        match args[1].as_str() {
            "-h" | "--help" => {
                print_help();
                Ok(())
            }
            "-v" | "--version" => {
                print_version();
                Ok(())
            }
            _ => Err(ArgumentError::InvalidArgument),
        }
    } else {
        Err(ArgumentError::InvalidNumberOfArguments)
    }
}

fn print_help() {
    println!(
        "\nUsage: music-library [OPTIONS]\n
    \nOptions: 
    \n  <NONE> \t\tRun music library
    \n  -v, --version \tPrint version information
    \n  -h, --help \t\tPrint help (you are here)\n"
    );
}

fn print_version() {
    println!(
        "\nmusic-library v{} \n
    Written by Jack Lee\n
    Source: github.com/kcajeel/music-library\n",
        env!("CARGO_PKG_VERSION")
    );
}

#[derive(Debug)]
pub enum ArgumentError {
    InvalidArgument,
    InvalidNumberOfArguments,
}
impl Error for ArgumentError {}
impl fmt::Display for ArgumentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidArgument => write!(f, "Error: Invalid argument. Use \"-h\" or \"--help\" for usage information."),
            Self::InvalidNumberOfArguments => write!(f, "Error: Invalid number of arguments. Use \"-h\" or \"--help\" for usage information."),
        }
    }
}
