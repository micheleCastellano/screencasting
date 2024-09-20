//use egui::mutex::Mutex;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::io::Write;
use std::sync::{Arc, Mutex};
//use std::net::TcpStream;
use std::thread;
use std::time::Duration;
// use warp::Filter;
use crate::capture::capture_screen;
//use reqwest::Client;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::task;
use std::fs::File;

mod capture;

#[derive(Serialize, Deserialize)]
struct Message {
    content: Vec<u8>,
}

/*
async fn send_request(stream: Arc<Mutex<TcpStream>>, v: &[u8]) -> Result<(), Box<dyn Error>> {
    /*
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
    */
    let mut stream = stream.lock().await;
    stream.write_all(v).await.expect("something went wrong");
    println!("messaggio inviato");
    Ok(())
}
*/

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    /*
    let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
    let stream = Arc::new(Mutex::new(stream));
    for _ in 1..1000 {
        let (v, w, h) = capture_screen().unwrap();
        println!("w:{}, h:{}", w, h);
        //let client = Client::new();
        let stream_clone = Arc::clone(&stream);

        // Send the POST request
        //task::spawn(send_request(client,v));
        //task::spawn(send_request(&mut stream, v));
        task::spawn(send_request(stream_clone, b"ciao"));
        thread::sleep(Duration::from_millis(10));
    }
    Ok(())
    */

    // Connessione al server
    let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
    println!("Connesso al server!");

    let (v, w, h) = capture_screen().unwrap();
    let chunk_size = 64 * 1024;
    let mut offset = 0;

    let mut file = File::create("frame_sender.txt").unwrap();
    for item in &v {
        writeln!(file, "{}", item).unwrap();
    }

    while offset < v.len(){
        let end = std::cmp::min(offset + chunk_size, v.len());
        let chunk = &v[offset..end];
        stream.write_all(chunk).await?;
        //println!("Scrittura, {:?}", chunk);
        offset = end;
    }

    /*
    // Invia molti messaggi numerati al server
    for i in 1..=100 {
        let (v, w, h) = capture_screen().unwrap();
        println!("w:{}, h:{}", w, h);
        //let v = [10, 100, 200];
        stream.write_all(&v).await?;
        println!("Inviato {}!", i);

        // Buffer per ricevere la risposta
        let mut buffer = [0; 1024];
        let n = stream.read(&mut buffer).await?;

        // Stampa la risposta ricevuta dal server
        println!(
            "Risposta dal server: {}",
            String::from_utf8_lossy(&buffer[..n])
        );
    }
     */

    Ok(())

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
