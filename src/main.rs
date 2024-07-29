use std::thread;
use std::time::Duration;
use crate::capture::{capture_screen, save};
use crate::make_video::make;

mod capture;
mod make_video;
mod gui;

fn main() {
    /*let mut images = vec![];

    let (v, w, h) = capture_screen().unwrap();
    images.push(v);
    for _ in 1..200 {
        if let Ok(res) = capture_screen() {
            images.push(res.0);
        }
        thread::sleep(Duration::from_millis(30));
    }

    for (i, image) in images.into_iter().enumerate() {
        let mut path = String::from("./images5/");
        path += i.to_string().as_str();
        path += ".png";
        save(image, w, h, path.as_str());
    }


    make("./images5/%d.png", "./images5/video.mp4");
    */

    gui::launch();

}