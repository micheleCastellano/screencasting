use std::thread;
use std::time::Duration;
use serde::{Deserialize, Serialize};
// use warp::Filter;
use reqwest::Client;
use tokio::task;
use crate::capture::{capture_screen};


mod capture;

#[derive(Serialize, Deserialize)]
struct Message{
    content:Vec<u8>
}


async fn send_request(client:Client,v:Vec<u8>){

            match client.post("http://192.168.10.114:3030/")
                .json(&Message { content: v })
                .send()
                .await
            {
                Ok(_response) => {
                    println!("ok");
                },
                Err(e) => println!("Error: {}", e),
            }
}
#[tokio::main]
async fn main() {

    for _ in 1..1000 {
        let (v, w, h) = capture_screen().unwrap();
        println!("w:{}, h:{}", w, h);
        let client = Client::new();

        // Send the POST request
        task::spawn(send_request(client,v));
        thread::sleep(Duration::from_millis(10));
    }



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

    gui::launch();
    */

}