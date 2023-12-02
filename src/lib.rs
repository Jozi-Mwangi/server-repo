use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::fs::OpenOptions;
use base64;

pub fn decode_from_base64(data: &str) -> Result<Vec<u8>, base64::DecodeError>{
    base64::decode(data)
}


pub fn process_input_file(branch: &str) -> Result<String, io::Error> {
    let input_file_path = format!("data/data/{}/branch_weekly_sales.txt", branch);
    let file = File::open(&input_file_path)?;
    let reader = BufReader::new(file);

    let mut total_sales = 0;
    let mut product_code = String::new();
    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        let parts: Vec<&str> = line.split(", ").collect();
        if index == 0 {
            product_code = parts[1].to_owned();
        }
        total_sales += parts[2].parse::<i32>().map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to parse sales data"))?;
    }

    let output_str = format!("{}, {}, {}", branch, product_code, total_sales);
    write_to_summary_file(&output_str)?;
    Ok("OK".to_string())
}

fn write_to_summary_file(output_str: &str) -> Result<(), io::Error> {
    let output_file_path = "data/data/weekly_summary/weekly_sales_summary.txt";
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(output_file_path)?;
    writeln!(file, "{}", output_str)?;
    Ok(())
}
