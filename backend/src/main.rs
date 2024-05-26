
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
const DB_URL:&str = !env("DATABASE_URL");

// CONSTANTS!
const OK_RESPONSE: &str = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n";
const NOT_FOUND: &str = "HTTP/1.1 404 NOT FOUND\r\n\r\n";
const INTERNAL_SERVER_ERROR: &str = "HTTP/1.1 500 INTERNAL ERROR\r\n\r\n";


// main function!
fn  main(){

    // SET THE DATABASE!
    if let Err(e) = set_database() {
        println!("Error setting database! {}",e);
        return;
    }

    let listener = TcpListener::bind(format!(0.0.0.0:8080)).unwrap();

    // start server and print port!
    println!("Server is listening on port 8080");

    // handle the client
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
    request.split("/").nth(2).unwrap_or_default().split_whitespace().next().unwrap_or_default()
}

// Deserialize USER FROM THE REQUEST BODY!
fn get_user_request_body(request:&str) -> Result<User, serde_json::Error> {
    serde_json::from_str(request.split("\r\n\r\n").last().unwrap_or_default())
}



// ADD
fn handle_client(mut stream:TcpStream) {
    let mut buffer = [0;1024];
    let mut request = String::new();

    match stream.read(&mut buffer){
        Ok(size) => {
           request.push_str(String::from_utf8_lossy(&buffer[..size]).as_ref());

           let (status_line, content) = match &*request{
            r if r.starts_with("OPTIONS") => (OK_RESPONSE.to_string(), "".to_string()),
            r if r.starts_with("POST /api/rust/users") => handle_post_request(r),
            r if r.starts_with("GET /api/rust/users") => handle_get_request(r),
            r if r.starts_with("GET /api/rust/users") => handle_get_all_request(r),
            r if r.starts_with("PUT /api/rust/users") => handle_put_request(r),
            r if r.starts_with("DELETE /api/rust/users") => handle_delete_request(r),
            _ => (NOT_FOUND.to_string(), "404 not found".to_string())
           };
           stream.write_all(format!("{}{}",status_line,content).as_bytes()).unwrap();
        }
        Err(e) => eprintln!("Unable to read stream: {}",e),
    }
}

// CONTROLLERS!

fn handle_post_request(request: &str) -> (String, String) {
    match (get_user_request_body(&request), Client::connect(DB_URL,NoTls)){
        (Ok(user), Ok(mut client)) => {
            client.execute(
                "INSERT INTO users (name, email) VALUES ($1, $2)",
                &[&user.name, &user.email]
            ).unwrap();
            (OK_RESPONSE.to_string(),"User created".to_string())
        }
        _ => (INTERNAL_SERVER_ERROR.to_string(), "Error".to_string())
    }
}


// handle get request function!
fn handle_get_request(request: &str) -> (String, String) {
    match(get_id(&request).parse::<i32>, Client::connect(DB_URL,NoTls)){
        (Ok(id), Ok(mut client)) => 
            match client.query("SELECT * FROM users WHERE id = $1", &[&id]){
                OK(row) => {
                    let user = User {
                        id: row.get(0),
                        name : row.get(1),
                        email: row.get(2),
                    };

                    (OK_RESPONSE.to_string(), serde_json::to_string(&user).unwrap())
                }
                _ => (NOT_FOUND.to_string(), "404 not found".to_string())
            }
         _ => (INTERNAL_SERVER_ERROR.to_string(), "Error".to_string())
    }
}


// handle get all request!
fn handle_get_all_request(request: &str) -> (String, String) {
    match Client::connect(DB_URL,NoTls) {
        Ok(mut client) => {
            let mut users = Vec::new();

            for row in client.query("SELECT * FROM users", &[]).unwrap(){
                users.push(User {
                    id: row.get(0),
                    name: row.get(1),
                    email: row.get(2),
                });
            }

            (OK_RESPONSE.to_string(), serde_json::to_string(&users).unwrap())
        }
        _ => (INTERNAL_SERVER_ERROR.to_string(), "ERROR".to_string()),
    }
}