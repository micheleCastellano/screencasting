
use std::process::Command;
// use std::fs;
// use std::path::Path;

pub fn make(input:&str, output:&str) {
    // Directory contenente le immagini
    // let images_dir = "path/to/images";

    // Leggi i file immagine dalla directory
    // let mut images: Vec<String> = fs::read_dir(images_dir)
    //     .unwrap()
    //     .map(|entry| entry.unwrap().path().display().to_string())
    //     .collect();

    // Ordina le immagini se necessario (es. per nome file)
    // images.sort();

    // Specifica il nome del file video output
    // let output_file = "output.mp4";

    // Genera il comando ffmpeg
    let ffmpeg_command = Command::new("ffmpeg")
        .arg("-y") // sovrascrivi l'output senza chiedere
        .arg("-framerate")
        .arg("30") // imposta il framerate
        .arg("-i")
        // .arg(format!("{}/%d.png", images_dir)) // specifica il pattern dei nomi delle immagini
        .arg(input)

        // .arg("-c:v")
        // .arg("libx264") // usa il codec H.264
        // .arg("-pix_fmt")
        // .arg("yuv420p") // specifica il formato dei pixel
        .arg(output)
        .output()
        .expect("failed to execute ffmpeg");

    // Stampa l'output di ffmpeg (utile per il debugging)
    println!("FFmpeg output: {:?}", ffmpeg_command);
}
