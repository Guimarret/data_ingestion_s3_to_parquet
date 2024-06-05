// use tokio_stream::StreamExt;
use aws_config::{BehaviorVersion, SdkConfig};
use std::{fs::File, io::Write, path::PathBuf, process::exit};
use aws_sdk_s3::Client;
use clap::Parser;
use tracing::trace;
use dotenv::dotenv;
use std::env;
use zip::read::ZipArchive;
use std::path::Path;
use tokio::io;

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

    let bucket = env::var("BUCKET").expect("BUCKET must be set in .env");
    let object = env::var("OBJECT").expect("OBJECT must be set in .env");
    let destination = env::var("DESTINATION").expect("DESTINATION must be set in .env");
    let destination = PathBuf::from(destination);

    let opt = Opt {
        bucket,
        object,
        destination,
    };

    let shared_config: SdkConfig = aws_config::load_defaults(BehaviorVersion::v2024_03_28()).await;
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

    let file = File::open(zip_path);

    let mut archive = ZipArchive::new(file);

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => Path::new(output_dir).join(path),
            None => continue,
        };

        if let Some(p) = outpath.parent() {
            std::fs::create_dir_all(p);
        }

        let mut outfile = File::create(&outpath);
        io::copy(&mut file, &mut outfile).await?;
    }

    println!("All files extracted successfully to {}", output_dir);

    Ok(()).expect("Reason")
}

// snippet-start:[s3.rust.get_object]
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