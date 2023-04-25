use std::env::temp_dir;
use std::fs::OpenOptions;
use std::io::Write;
use std::time::UNIX_EPOCH;

use zune_image::codecs::png::PngEncoder;
use zune_image::image::Image;
use zune_image::traits::EncoderTrait;

pub fn open_in_default_app(image: &Image)
{
    let time = format!(
        "{}.png",
        std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );
    let mut path = temp_dir();

    path.push(time);

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(&path)
        .unwrap();

    let data = PngEncoder::new().encode(image).unwrap();
    file.write_all(&data).unwrap();

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(path.to_str().unwrap())
            .spawn()
            .unwrap();
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("start")
            .arg(path.to_str().unwrap())
            .spawn()
            .unwrap();
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(path.to_str().unwrap())
            .spawn()
            .unwrap();
    }
}
