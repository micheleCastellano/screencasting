use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::error::Error;
use std::net::TcpListener;
//use tokio::net::TcpListener;
use warp::Filter;
mod capture;
mod gui;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    /*
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
  */

    //Crea il listener su 127.0.0.1:8080
    //let listener = TcpListener::bind("127.0.0.1:8080").await?;
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    println!("Server in ascolto su 127.0.0.1:8080");

    //loop {
    // Accetta una connessione
    let (socket, _) = listener.accept()?;
    gui::launch(socket);
    /*
    let socket_copy = TcpStream::try_clone(&socket);
    gui::launch(socket_copy);
    println!("Client connesso!");


    // Buffer per ricevere i dati
    let mut buffer = [0; 1024];

        loop {
            // Leggi dati dal client
            let n = socket.read(&mut buffer).await?;

            // Se non ci sono dati, termina la connessione
            if n == 0 {
                println!("Connessione terminata dal client.");
                break;
            }

            // Stampa il messaggio ricevuto
            let messaggio = String::from_utf8_lossy(&buffer[..n]);
            println!("Messaggio ricevuto: {}", messaggio);

            // Invia una risposta al client
            socket.write_all(b"Messaggio ricevuto!").await?;
            gui::launch(socket);
        }
     */
    Ok(())
    //}
}