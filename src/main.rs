use tokio_stream::StreamExt;
use aws_config::{meta::region::RegionProviderChain, BehaviorVersion};
use std::{fs::File, io::Write, path::PathBuf, process::exit};
use aws_sdk_s3::Client;
use clap::Parser;
use tracing::trace;
use dotenv::dotenv;
use std::env;

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

    let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
    let shared_config = aws_config::from_env();
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