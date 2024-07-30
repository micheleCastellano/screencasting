use scrap::{Capturer, Display};
use std::io::ErrorKind::WouldBlock;
use std::thread;
use std::time::Duration;
use std::fs::File;
use std::io::BufWriter;
// use std::time::{SystemTime, UNIX_EPOCH};

pub fn save(buffer: Vec::<u8>, w: usize, h: usize, path: &str){

    let mut bitflipped = Vec::with_capacity(w * h * 4);
    let stride = buffer.len() / h;

    for y in 0..h {
        for x in 0..w {
            let i = stride * y + 4 * x;
            bitflipped.extend_from_slice(&[
                buffer[i + 2],
                buffer[i + 1],
                buffer[i],
                255,
            ]);
        }
    }

    repng::encode(
        // File::create("screenshot.png").unwrap(),
        File::create(path).unwrap(),
        w as u32,
        h as u32,
        &bitflipped,
    ).unwrap();

    println!("Image saved to {}.",path);
}

pub fn capture_screen() -> Result<(Vec<u8>, usize, usize), Box<dyn std::error::Error>>{
    let one_second = Duration::new(1, 0);
    let one_frame = one_second / 60;

    let display = Display::primary().expect("Couldn't find primary display.");
    let mut capturer = Capturer::new(display).expect("Couldn't begin capture.");
    let (w, h) = (capturer.width(), capturer.height());

    loop {
        match capturer.frame() {
            Ok(buffer) => {
                let frame = buffer.to_vec();

                // Converte il frame in un'immagine RGBA
                let buffer: image::ImageBuffer<image::Rgba<u8>, Vec<u8>> = image::ImageBuffer::from_raw(w as u32, h as u32, frame)
                    .expect("Failed to convert frame to ImageBuffer");

                // Salva l'immagine come PNG in memoria
                let mut png_data = Vec::new();
                {
                    let writer = BufWriter::new(&mut png_data);
                    let mut encoder = png::Encoder::new(writer, buffer.width(), buffer.height());
                    encoder.set_color(png::ColorType::Rgba);
                    encoder.set_depth(png::BitDepth::Eight);
                    let mut writer = encoder.write_header().expect("Failed to write PNG header");
                    writer
                        .write_image_data(&buffer)
                        .expect("Failed to write PNG data");
                }

                return Ok((png_data, w, h))
            },
            Err(error) => {
                if error.kind() == WouldBlock {
                    // Keep spinning.
                    thread::sleep(one_frame);
                    continue;
                } else {
                    return Err(Box::new(error))
                }
            }
        };
    }



}






/*
pub fn capture_screen() -> Result<(Vec<u8>, usize, usize), Box<dyn std::error::Error>> {
    let display = Display::primary()?;
    let mut capturer = Capturer::new(display)?;
    let (width, height) = (capturer.width(), capturer.height());

    loop {
        match capturer.frame() {
            Ok(frame) => return Ok((frame.to_vec(), width, height)),
            Err(error) => match error.kind() {
                WouldBlock => continue,
                _ => return Err(Box::new(error)),
            },
        }
    }
}

pub fn save(buffer: Vec::<u8>, width: usize, height: usize, path: &str) {
    let buffer: Vec<u8> = buffer.chunks(4)
        .flat_map(|bgra| vec![bgra[2], bgra[1], bgra[0]])  // BGRA a RGB
        .collect();

    // let _ = std::fs::create_dir(path);
    // Salva l'immagine
    // let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs().to_string();
    // let path = path.to_string() + "/screenshot_" + ts.as_str() + ".png";



    // let mut file = File::create(file_name).expect("Errore nella creazione del file");
    image::save_buffer(
        &path,
        &buffer,
        width as u32,
        height as u32,
        image::ColorType::Rgb8,
    ).expect("Errore nel salvataggio dell'immagine");

    println!("Screenshot salvato come {}", path);
}
*/
