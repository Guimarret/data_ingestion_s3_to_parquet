use aws_config::{BehaviorVersion, SdkConfig};
use aws_sdk_s3::Client;
use clap::Parser;
use dotenv::dotenv;
use polars::prelude::*;
use std::env;
use std::path::Path;
use std::{
    fs::File,
    io::{self, Write},
    path::PathBuf,
    process::exit,
};
use tracing::trace;
use tracing_subscriber::registry::Data;

#[derive(Debug, Parser)]
struct Opt {
    #[structopt(long)]
    bucket: String,
    #[structopt(long)]
    object: String,
    #[structopt(long)]
    destination: PathBuf,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenv().ok();

    let unzipped_data = env::var("UNZIPPED_DATA_1").unwrap();

    if Path::new(&unzipped_data).exists() {
        println!("File {} exists.", unzipped_data);
        column_verifier(&unzipped_data)
    } else {
        println!("File {} does not exist.", unzipped_data);
        let bucket = env::var("BUCKET").expect("BUCKET must be set in .env");
        let object = env::var("OBJECT").expect("OBJECT must be set in .env");
        let destination = env::var("DESTINATION").expect("DESTINATION must be set in .env");
        let destination = PathBuf::from(destination);

        let opt = Opt {
            bucket,
            object,
            destination,
        };

        let shared_config: SdkConfig =
            aws_config::load_defaults(BehaviorVersion::v2024_03_28()).await;
        let client = aws_sdk_s3::Client::new(&shared_config);

        match get_object(client, opt).await {
            Ok(bytes) => {
                println!("Wrote {bytes} bytes");
            }
            Err(err) => {
                eprintln!("Error: {}", err);
                exit(1);
            }
        }
        let zip_path = "data/zip/data.zip";
        let output_dir = "data/unzipped";
        unzip(zip_path, output_dir);

        println!("Starting the actual data filtering and nasty codes hehe");
        column_verifier(&unzipped_data);
        column_filter(&unzipped_data);
    }
}

fn column_verifier(unzipped_data: &String) {
    let df = CsvReadOptions::default()
        .try_into_reader_with_file_path(Some(unzipped_data.into()))
        .unwrap()
        .finish()
        .unwrap();
    let column_names: Vec<&str> = df.get_column_names();

    let expected_columns = vec![
        "Date",
        "NO2",
        "O3",
        "PM10",
        "PM2.5",
        "Latitude",
        "Longitude",
        "station_name",
        "Wind-Speed (U)",
        "Wind-Speed (V)",
        "Dewpoint Temp",
        "Soil Temp",
        "Total Percipitation",
        "Vegitation (High)",
        "Vegitation (Low)",
        "Temp",
        "Relative Humidity",
        "code",
        "id",
    ];

    for &col in &expected_columns {
        if !column_names.contains(&col) {
            println!("Missing expected column: {}", col);
        }
    }

    for &col in &column_names {
        if !expected_columns.contains(&col) {
            println!("Unexpected column found: {}", col);
        }
    }
    // println!("{}", df);
}

fn column_filter(unzipped_data: &String) {
    let df = CsvReadOptions::default()
        .try_into_reader_with_file_path(Some(unzipped_data.into()))
        .unwrap()
        .finish()
        .unwrap();

    // Define the desired columns
    let desired_columns = vec![
        "Date",
        "NO2",
        "O3",
        "PM10",
        "PM2.5",
        "Latitude",
        "Longitude",
        "station_name",
    ];

    // let filtered_df = filter_columns(df, &desired_columns);
    let filtered_df: Result<DataFrame, PolarsError> = df.select(desired_columns);
    println!("{:?}", filtered_df);
}

fn unzip(zip_path: &str, output_dir: &str) {
    let file = File::open(zip_path).expect("Failed to open file");
    let mut archive = ZipArchive::new(file).expect("Failed to read zip file");

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).expect("Archive error");

        let outpath = match file.enclosed_name() {
            Some(path) => Path::new(output_dir).join(path),
            None => continue,
        };

        let mut outfile = File::create(&outpath).expect("Outpath error");
        io::copy(&mut file, &mut outfile).unwrap();
    }

    println!("All files extracted successfully to {}", output_dir);
}
async fn get_object(client: Client, opt: Opt) -> Result<usize, anyhow::Error> {
    trace!("bucket:      {}", opt.bucket);
    trace!("object:      {}", opt.object);
    trace!("destination: {}", opt.destination.display());

    let mut file = File::create(opt.destination.clone())?;

    let mut object = client
        .get_object()
        .bucket(opt.bucket)
        .key(opt.object)
        .send()
        .await?;

    let mut byte_count = 0_usize;
    while let Some(bytes) = object.body.try_next().await? {
        let bytes_len = bytes.len();
        file.write_all(&bytes)?;
        trace!("Intermediate write of {bytes_len}");
        byte_count += bytes_len;
    }

    Ok(byte_count)
}
