use ::zip::ZipArchive;
use anyhow::Context;
use aws_config::{BehaviorVersion, SdkConfig};
use aws_sdk_s3::Client;
use dotenv::dotenv;
use polars::{
    error::PolarsError,
    frame::DataFrame,
    prelude::{CsvReadOptions, ParquetWriter, SerReader},
};
use std::{
    fs::File,
    io::{self, Write},
    path::PathBuf,
    process::exit,
    path::Path,
    env
};
use tracing::{error, info, trace, warn};

struct Opt {
    bucket: String,
    object: String,
    destination: PathBuf,
}

#[tokio::main]
async fn main() {
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    dotenv().ok();

    let unzipped_data = env::var("UNZIPPED_DATA_1").unwrap();

    if Path::new(&unzipped_data).exists() {
        info!("File {} exists.", unzipped_data);
        column_verifier(&unzipped_data);
        let mut df: DataFrame = column_filter(&unzipped_data).expect("Dataframe passed");

        let mut file = std::fs::File::create("data/datafile.parquet").unwrap();
        ParquetWriter::new(&mut file).finish(&mut df).unwrap();
    } else {
        info!("File {} does not exist.", unzipped_data);
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
                info!("Wrote {bytes} bytes");
            }
            Err(err) => {
                error!("Error: {}", err);
                exit(1);
            }
        }
        let zip_path = "data/zip/data.zip";
        let output_dir = "data/unzipped";
        unzip(zip_path, output_dir);

        info!("Starting the actual data filtering and nasty codes hehe");
        column_verifier(&unzipped_data);
        let mut df: DataFrame = column_filter(&unzipped_data).expect("Dataframe passed");

        let mut file = std::fs::File::create("data/datafile.parquet").unwrap();
        ParquetWriter::new(&mut file).finish(&mut df).unwrap();
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
    let mut all_columns_present = true;

    for &col in &expected_columns {
        if !column_names.contains(&col) {
            warn!("Missing expected column: {}", col);
            all_columns_present = false;
        }
    }

    for &col in &column_names {
        if !expected_columns.contains(&col) {
            info!("Unexpected column found: {}", col);
        }
    }
    if all_columns_present {
        info!("All columns were verified correctly!");
    }
}

fn column_filter(unzipped_data: &String) -> Result<DataFrame, PolarsError> {
    let df = CsvReadOptions::default()
        .try_into_reader_with_file_path(Some(unzipped_data.into()))
        .unwrap()
        .finish()
        .unwrap();

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

    let filtered_df: Result<DataFrame, PolarsError> = df.select(desired_columns);
    info!("{:?}", filtered_df);
    return filtered_df;
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

    info!("All files extracted successfully to {}", output_dir);
}

async fn get_object(client: Client, opt: Opt) -> Result<usize, anyhow::Error> {
    trace!("bucket:      {}", opt.bucket);
    trace!("object:      {}", opt.object);
    trace!("destination: {}", opt.destination.display());

    let mut file = File::create(opt.destination.clone())
        .with_context(|| format!("Failed to create file at {}", opt.destination.display()))?;

    let mut object = client
        .get_object()
        .bucket(opt.bucket)
        .key(opt.object)
        .send()
        .await
        .context("Failed to get object from S3")?;

    let mut byte_count = 0_usize;
    while let Some(bytes) = object
        .body
        .try_next()
        .await
        .context("Failed to read bytes from object stream")?
    {
        let bytes_len = bytes.len();
        file.write_all(&bytes).with_context(|| {
            format!(
                "Failed to write bytes to file at {}",
                opt.destination.display()
            )
        })?;
        file.sync_all().unwrap();
        trace!("Intermediate write of {bytes_len}");
        byte_count += bytes_len;
    }

    Ok(byte_count)
}
