use std::collections::{HashMap};
use std::sync::mpsc::channel;
use warp::Filter;
mod capture;
mod gui;

#[tokio::main]
async fn main() {
    let (sender, receiver) = channel();
    let route = warp::body::json()
        .map(move |simple_map: HashMap<String, Vec<u8>>| {
            let _ = sender.send(simple_map["content"].clone());
            "Got a JSON body!"
        });
    let server = warp::serve(route)
        .run(([0, 0, 0, 0], 3030));
    tokio::spawn(server);
    println!("gui start");
    gui::launch(receiver);
}