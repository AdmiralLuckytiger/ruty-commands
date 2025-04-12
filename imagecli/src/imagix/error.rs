use std::{convert::From, io, fmt};

/// Data strcture for handling the multiple kinderrors in the library
#[derive(Debug)]
pub enum ImagixError {
    FileIOError(String),
    UserInputError(String),
    ImageResizingError(String),
    FormatError(String),
}

impl From<io::Error> for ImagixError {
    fn from(error: io::Error) -> Self {
        match error.kind() {
            io::ErrorKind::NotFound => ImagixError::FileIOError("File or folder doesn't exist".to_string()),
            io::ErrorKind::PermissionDenied => ImagixError::FileIOError("Denied action by permissions".to_string()),
            _ => ImagixError::FileIOError("Unexpected error ocurrer".to_string()),
        }
    }
}

impl From<io::ErrorKind> for ImagixError {
    fn from(error: io::ErrorKind) -> Self {
        match error {
            io::ErrorKind::InvalidData => ImagixError::UserInputError("Invalid data was used".to_string()),
            _ => ImagixError::FileIOError("Unexpected error ocurred".to_string()),
        }
    }
}

impl From<image::ImageError> for ImagixError {
    fn from(error: image::ImageError) -> Self {
        match error {
            _ => ImagixError::ImageResizingError("Error related to image resizing".to_string()),
        }
    }
}

impl fmt::Display for ImagixError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ImagixError::FileIOError(e) => {
                write!(f, "FileIOError: {e}")
            },
            ImagixError::UserInputError(e) => {
                write!(f, "UserInputError: {e}")
            },
            ImagixError::ImageResizingError(e) => {
                write!(f, "ImageResizingError: {e}")
            },
            ImagixError::FormatError(e) => {
                write!(f, "FormatError: {e}")
            },
        }
    }
}