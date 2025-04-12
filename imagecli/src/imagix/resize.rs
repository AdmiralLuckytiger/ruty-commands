use std::{fs, io, path::PathBuf, str::FromStr, time::Instant};
use image::ImageFormat;

use super::{error::ImagixError, stats::Elapsed};

/// Data structure that specifies the scope of the process
#[derive(Debug)]
pub enum Mode {
    Single,
    All,
}

impl FromStr for Mode {
    type Err = ImagixError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "single" | "Single" => return Ok(Mode::Single),
            "all" | "All"       => return Ok(Mode::All),
            _ => return Err(ImagixError::FormatError("Invalid input".to_string()))
        }
    }   
}

/// Data structure that specifies the output size of the given images
#[derive(Debug)]
pub enum SizeOption {
    Small, // size = 200px
    Medium, // size = 400px
    Large, // size = 800px
}

impl FromStr for SizeOption {
    type Err = ImagixError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "small" | "Small" => {
                return Ok(SizeOption::Small);
            },
            "medium" | "Medium" => {
                return Ok(SizeOption::Medium);
            },
            "large" | "Large" => {
                return Ok(SizeOption::Large);
            },
            _ => return Err(ImagixError::FormatError("Invalid input".to_string()))
        }
    }   
}


/// Public interface for interacting with the library
pub fn process_resize_request(size: SizeOption, mode: Mode, src_folder: &mut PathBuf) -> Result<(), ImagixError> {
    match mode {
        Mode::Single => {
            resize_single(src_folder, size)
        },
        Mode::All => {
            resize_all(src_folder, size)
        },
    }
}

/// This functions wrap the functionality of resize image for a specified image
fn resize_single(path: &mut PathBuf, size: SizeOption) -> Result<(), ImagixError> {
    let size: u32 = match size {
        SizeOption::Large => {
            200
        },
        SizeOption::Medium => {
            400
        }
        SizeOption::Small => {
            800
        }
    };

    resize_image(size, path)
}

/// This function wrap the functionality of resize image for a all folder
fn resize_all(path: &mut PathBuf, size: SizeOption) -> Result<(), ImagixError>{
    let size: u32 = match size {
        SizeOption::Large => {
            200
        },
        SizeOption::Medium => {
            400
        }
        SizeOption::Small => {
            800
        }
    };

    if let Ok(mut entries) = get_images_files(path.clone()) {
        for entry in &mut entries {
            resize_image(size, entry)?
        };

        Ok(())
    } else {
        Err(ImagixError::FileIOError("Unable to read images!".to_string()))
    }
}

/// This functions generetes the resize image and the necesary folder
fn resize_image(size: u32, src_folder: &mut PathBuf) -> Result<(), ImagixError>{
    // Cosntruct destination filename with .png extension
    let new_file_name = src_folder
        .file_stem()
        .expect("We are working with only valid inputs")
        .to_str().ok_or(std::io::ErrorKind::InvalidInput)
        .map(|f| format!("{}.png", f));

    // Construct path to destination folder i.e. create /tmp
    // under source folder if not exists
    let mut dest_folder = src_folder.clone();
    dest_folder.pop();
    dest_folder.push("tmp/");
    if !dest_folder.exists() {
        fs::create_dir(&dest_folder)?;
    }
    dest_folder.pop();
    dest_folder.push("tmp/tmp.png");
    dest_folder.set_file_name(new_file_name?.as_str());

    //dbg!(&src_folder);
    // Resize image and take some measuraments
    let timer = Instant::now();
    let img = image::open(&src_folder)?;
    let scaled = img.thumbnail(size, size);
    let mut output = fs::File::create(&dest_folder)?;
    scaled.write_to(&mut output, ImageFormat::Png)?;
    println!(
        "Thumbnailed file: {:?} to size {}x{} in {}. Output file in {:?}",
        src_folder,
        size,
        size,
        Elapsed::from(&timer),
        dest_folder
    );
    Ok(())
}

/// This function retrieves the list of images files contained in a source folder
pub fn get_images_files(src_folder: PathBuf) -> Result<Vec<PathBuf>, ImagixError> {

    // 1. retrieve the directory entries in the source folder and collect tem in a vector
    let entries: Vec<PathBuf> = fs::read_dir(src_folder)
        .map_err(|e| ImagixError::from(e))?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()?
        .into_iter()
        .filter(|r| {
            r.extension() == Some("JPG".as_ref())
                || r.extension() == Some("jpg".as_ref())
                || r.extension() == Some("PNG".as_ref())
                || r.extension() == Some("png".as_ref())
        })
        .collect();
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_image_resize() {
        let mut path = PathBuf::from("/tmp/images/image1.jpg");

        let destination_path = PathBuf::from("/tmp/images/tmp/image1.png");

        match process_resize_request(SizeOption::Small, Mode::Single, &mut path) {
            Ok(_) => println!("Successful resize of single image"),
            Err(e) => println!("Error in single image: {:?}", e),
        }

        assert_eq!(true, destination_path.exists())
    }

    #[test]
    fn test_multiple_image_resize() {
        let mut path = PathBuf::from("/tmp/images/");
        let _res = process_resize_request(SizeOption::Small, Mode::All, &mut path);

        let destination_path1 = PathBuf::from("/tmp/images/tmp/image1.png");
        let destination_path2 = PathBuf::from("/tmp/images/tmp/image2.png");

        assert_eq!(true, destination_path1.exists());
        assert_eq!(true, destination_path2.exists());
    }
}