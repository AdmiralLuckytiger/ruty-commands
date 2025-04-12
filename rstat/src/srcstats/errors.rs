use std::{
    fmt,
    io,
    num,
};

#[derive(Debug)]
pub struct StatsError{
    pub warn: String,
}


impl fmt::Display for StatsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}
    
impl From<&str> for StatsError {
    fn from(value: &str) -> Self {
        StatsError {
            warn: value.to_string(),
        }
    }
}

impl From<io::Error> for StatsError {
    fn from(value: io::Error) -> Self {
        StatsError {
            warn: value.to_string(),
        }   
    }
}

impl From<num::TryFromIntError> for StatsError {
    fn from(_value: num::TryFromIntError) -> Self {
        StatsError {
            warn: "Number conversion error".to_string(),
        }
    }
}