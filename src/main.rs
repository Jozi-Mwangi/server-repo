

use std::net::{TcpListener, TcpStream, Shutdown};
use std::thread;
use std::io::{Read, Write};
use std::fs;
use std::time::Instant;
use server_side::{decode_from_base64, process_input_file};
use env_logger::{Builder, Env};
use log::{info, error};

fn init_logger(){
    let env = Env::default()
        .filter("info");

    Builder::from_env(env)
        .format_level(false)
        .format_timestamp_nanos()
        .init();
}

fn main() {
    // Initialize a logger
    init_logger();
    
    std::env::set_var("RUST_LOG", "info");

    let branches = vec![
        "ALBNM",
        "CTONGA"
    ];
    
    // Create a folder called "data" if it does not exist
    let output_dir = "data/data/weekly_summary";
    if !fs::metadata(&output_dir).is_ok() {
        fs::create_dir_all(&output_dir).expect("Failed to create output directory");
    }

    let start = Instant::now();

    for branch in &branches {
        match process_input_file(branch) {
            Ok(_) => println!("Processed {}", branch),
            Err(e) => eprintln!("Failed to process {}: {}", branch, e),
        }
    }
   
    start_listening("127.0.0.1:8080");

    
    let duration = start.elapsed();
    println!("Processing time: {:?}", duration);

    println!("Phew! I am done.");
}

fn start_listening(server_address: &str) {
    let listener = TcpListener::bind(server_address)
        .expect("Failed to bind to the server");
    info!("Server listening on port {}" , server_address);


    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // Spawn a new thread to handle the client
                thread::spawn(|| handle_client(stream));
            }
            Err(e) => {
               error!("Error accepting connection: {:?}", e);
            }
        }
    }
}

fn handle_client(mut stream : TcpStream){
    // Read branch code from the client
    let mut branch_code = String::new();
    if let Err(e)  = stream.read_to_string(&mut branch_code){
        error!("Error reading branch code: {:?}", e);
        return;
    };
    info!("Received branch code {} ", branch_code);

    // Create a folder for the branch in the "data" directory
    let branch_folder = format!("data/{}", branch_code.trim());
    if let Err(e) = fs::create_dir_all(&branch_folder){
        error!("Error creating branch folder: {:?}", e);
        return;
    };

    // Send acknowledgment to the client
    if let Err(e) = stream.write(b"OK") {
        error!("Error writing acknowledgment to stream: {:?}", e);
        return;
    };

    // Receive Base64 content from client
    let mut base64_content = String::new();
    if let Err(e) = stream.read_to_string(&mut base64_content) {
        error!("Error reading Base64 content: {:?}", e);
        return;
    };
    info!("Received Base64 content");

    
    // Remove the "~" from the beginning and end of the Base64 content
    let trimmed_content = base64_content.trim_start_matches('~').trim_end_matches('~');

    // Decode Base64 Content
    let decoded_content = match decode_from_base64(trimmed_content){
        Ok(content) => content,
        Err(err )=>{
            error!("Error decoding Base64 content: {:?}", err);
            return;
        }
    };

    // Save decoded content to a file
    let file_path = format!("{}/branch_weekly_sales.txt", branch_folder);
    if let Err(e) = fs::write(file_path, &decoded_content) {
        error!("Error writing to file: {:?}", e);
        return;
    };
    info!("Sales report file saved successfully");

    //  Send acknowledgment to the client and close the connection
    if let Err(e) = stream.write(b"OK") {
        error!("Error writing acknowledgment to stream: {:?}", e);
    };

    // Close the connection in a finally block
    if let Err(e) = stream.shutdown(Shutdown::Both) {
        error!("Failed to close the connection: {:?}", e);
    }
}