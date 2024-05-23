
use postgres::{Client, NoTls};
use postgres::Error as PostgresError;
use std::net::{TcpListener,TcpStream};
use std::io::{Read,Write};
use std::env;

#[macro_use]
extern crate serde_derive;

#[derive(Serialize, Deserialize)]
struct User{
    id: Option<i32>,
    name: String,
    email: String,
}

// DATABASE URL!
const DB_URL:&str = env!("DATABASE_URL");

// CONSTANTS!
const OK_RESPONSE: &str = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n";
const NOT_FOUND: &str = "HTTP/1.1 404 NOT FOUND\r\n\r\n";
const INTERNAL_ERROR: &str = "HTTP/1.1 500 INTERNAL ERROR\r\n\r\n";


// main function!
fn  main(){

    // SET THE DATABASE!
    if let Err(_) = set_database() {
        println!("Error setting database!");
        return;
    }

    // start server and print port!
    println!("Server is listening on port 8080");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_client(stream)
            }
            Err(e) => {
                println!("Unable to connect: {}", e);
            }
        }
    }
}


fn set_database() -> Result<(), PostgresError>{
    let mut client = Client::connect(DB_URL,NoTls)?;
    client.batch_execute(
        "
        CREATE TABLE IF NOT EXISTS users (
            id SERIAL PRIMARY KEY,
            name VARCHAR(255) NOT NULL,
            email VARCHAR(255) NOT NULL
        )"
    )?;

    Ok(())
}

// GET ID FROM THE REQUEST URL!
fn get_id(request: &str) -> &str {
    request.split("/").nth(4).unwrap_or_default().split_whitespace().next().unwrap_or_default()
}

// Deserialize SER FROM THE REQUEST BODY!
fn get_user_request_body(request:&str) -> Result<User, serde_json::Error> {
    serde_json::from_str(request.split("\r\n\r\n").last().unwrap_or_default())
}
