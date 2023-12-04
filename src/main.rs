use std::net::{TcpListener, TcpStream, Shutdown};
use std::thread;
use std::io::{Read, Write, BufReader, BufRead};
use std::fs;
use std::time::Instant;
use server_side::{decode_from_base64, process_input_file};
use env_logger::{Builder, Env};
use log::{error};

fn init_logger(){
    let env = Env::default()
        .filter("println");

    Builder::from_env(env)
        .format_level(false)
        .format_timestamp_nanos()
        .init();
}

fn main() {
    // Initialize a logger
    init_logger();
    
    std::env::set_var("RUST_LOG", "println");

    let branches = vec![
        "ALBNM",
        "CTONGA"
    ];
    start_listening("127.0.0.1:8080");

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
   
    
    let duration = start.elapsed();
    println!("Processing time: {:?}", duration);

    println!("Phew! I am done.");
}

fn start_listening(server_address: &str) {
    let listener = TcpListener::bind(server_address)
        .expect("Failed to bind to the server");
    println!("Server listening on port {}" , server_address);


    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {

                let client_address = stream.peer_addr().expect("Failed to get the client address");
                println!("Accepted connection from a client, with IP address: {} ", client_address);
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
    println!("Starting to handle the client");
    // Read branch code from the client
    // let mut branch_code_buffer = String::new();
    // if let Err(e) = stream.read_to_string(&mut branch_code_buffer) {
    //     println!("Error reading branch code: {:?}", e);
    //     return;
    // }else {
    //     println!("The branch code is: {:?}", branch_code_buffer);
    // };

        let mut branch_length_buff = [0; 4];
        stream.read_exact(&mut branch_length_buff).expect("Failed to read length");
        let length = u32::from_be_bytes(branch_length_buff);


    let mut branch_code_buffer = vec![0; length as usize];
    if let Err(e) = stream.read_exact(&mut branch_code_buffer) {
        eprintln!("Error reading branch code: {:?}", e);
        return;
    }

    // Process the branch code
    let branch_code = String::from_utf8_lossy(&branch_code_buffer);
    println!("Received branch code: {}", branch_code);


    if branch_code_buffer.is_empty(){
        println!("Error, received empty code from client")
    }else {
        println!("Not empty")
    };


    // Create a folder for the branch in the "data" directory
    let branch_folder = format!("data/{}", branch_code);
    if let Err(e) = fs::create_dir_all(&branch_folder){
        error!("Error creating branch folder: {:?}", e);
        return;
    };

    // Send acknowledgment to the client
    if let Err(e) = stream.write_all(b"OK") {
        error!("Error writing acknowledgment to stream: {:?}", e);
        return;
    };

    // Receive Base64 content from client
    let mut base64_content = String::new();
    if let Err(e) = stream.read_to_string(&mut base64_content) {
        error!("Error reading Base64 content: {:?}", e);
        return;
    };
    println!("Received Base64 content: {}", base64_content);

    
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
    // let file_path = format!("{}/branch_weekly_sales.txt", branch_folder);
    // if let Err(e) = fs::write(file_path, &decoded_content) {
    //     error!("Error writing to file: {:?}", e);
    //     return;
    // };
    println!("Sales report file saved successfully");

    //  Send acknowledgment to the client and close the connection
    if let Err(e) = stream.write(b"OK") {
        error!("Error writing acknowledgment to stream: {:?}", e);
    };

    // Close the connection in a finally block
    if let Err(e) = stream.shutdown(Shutdown::Both) {
        error!("Failed to close the connection: {:?}", e);
    }
}