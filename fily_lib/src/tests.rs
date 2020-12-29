#[test]
fn it_works() {
    assert_eq!(2 + 2, 4);
}

use crate::check_image_formats::{image_format_guess_from_path, ImageFormatFromPath};
use std::path::PathBuf;

#[test]
fn image_format_from_path_test() {
    assert_eq!(image_format_guess_from_path("image.jpg"), ImageFormatFromPath::Format(image::ImageFormat::Jpeg));
    
    assert_eq!(image_format_guess_from_path("picture.png"), ImageFormatFromPath::Format(image::ImageFormat::Png));

    assert_eq!(image_format_guess_from_path("abcdefg.abc"), ImageFormatFromPath::UnknownFormat(PathBuf::from("abc")));

    assert_eq!(image_format_guess_from_path("maybeNotAnImage."), ImageFormatFromPath::UnknownFormat(PathBuf::from("")));

    assert_eq!(image_format_guess_from_path("notAnImage"), ImageFormatFromPath::NoExtension);
}

use crate::delete::overwrite_with_zeroes;
use std::io::Cursor;

#[test]
fn overwrite_with_zeroes_test() {
    let mut buf = vec![4_u8; 100_003];

    overwrite_with_zeroes(Cursor::new(&mut buf), 100_003).unwrap();

    assert_eq!(buf, vec![0_u8; 100_003]);

    overwrite_with_zeroes(Cursor::new(&mut buf), 100_004).unwrap();

    assert_eq!(buf, vec![0_u8; 100_004]);
}

use crate::duplicates::crc32_from_bytes;

#[test]
fn crc32_from_bytes_test() {
    let input = b"Super important text it is very important that the crc32 hash of this text never gets calculated";

    assert_eq!(crc32_from_bytes(input), 0x28873A5C);
}
