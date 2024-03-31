// Custom Error types live here

use std::{error::Error, fmt};

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
