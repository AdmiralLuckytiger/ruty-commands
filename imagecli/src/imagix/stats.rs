use std::path::PathBuf;

use super::{error::ImagixError, resize::get_images_files};

use std::{fmt, convert::From, time};

/// Data structure that stores the duration of the resize process.
pub struct Elapsed {
    duration: u32,
}

impl fmt::Display for Elapsed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Elapsed {{ duration: {} }}", self.duration)
    }
}

impl From<&time::Instant> for Elapsed {
    fn from(input: &time::Instant) -> Self {
        Elapsed {
            duration: input.elapsed().as_secs() as u32,
        }
    }
}

/// Function that analyze the number of images and the total size of the given path.
pub fn get_stats(src_folder: PathBuf) -> Result<(usize, f64), ImagixError>{
    let image_files = get_images_files(src_folder.to_path_buf())?;
    let size = image_files
        .iter()
        .map(move |f| f.metadata().unwrap().len())
        .sum::<u64>();
    Ok((image_files.len(), (size / 1000) as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_stats() {
        let path = PathBuf::from("/tmp/images");

        match get_stats(path) {
            Ok((size,num)) => {
                println!("{} images t => {}KB", size, num);
            }
            Err(_) => {
                panic!("Error in test!!!");
            }
        }
    }
}