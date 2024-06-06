# Rust Data Ingestion Project with AWS S3 Integration

Welcome to my Rust-based data ingestion project! This repository aims to provide a robust solution for ingesting, filtering, and transforming data from AWS S3 buckets using Rust. I leverage the powerful AWS SDK for Rust to interact with S3, utilize Polars for efficient data filtering and transformation, Nix for environment setup, and Cargo for managing Rust dependencies.

## Features

- Seamless integration with AWS S3 for data ingestion.
- Efficient data filtering and transformation using Polars.
- Simplified environment setup with Nix.
- Easy dependency management with Cargo.

## Prerequisites

Before getting started, ensure you have the following installed:

- Nix: [Install Nix](https://nixos.org/download.html)
- AWS CLI: [Install AWS CLI](https://aws.amazon.com/cli/)

## Installation

To install the project dependencies and set up the environment, follow these steps:

1. Clone the repository:

   ```Shell
   git clone https://github.com/Guimarret/rust_data_ingesting
   ```

2. Navigate to the project directory:

   ```Shell
   cd rust_data_ingesting
   ```

3. Run Nix to set up the environment:

   ```Shell
   nix develop
   ```

4. Build the project:
   
   ```Shell
   cargo build
   ```

## Usage

After installation, you can use the project for data ingestion from AWS S3. Here's a basic usage guide:

1. Set up AWS credentials using AWS CLI:

   ```Shell
   aws configure
   ```

2. Run the data ingestion script:

   ```Shell
   cargo run --bin data_ingest -- <options>
   ```

   Replace `<options>` with appropriate parameters for data ingestion.

## Contributing

We welcome contributions from the community. To contribute to this project, follow these steps:

1. Clone the repository.
2. Make your changes.
3. Use as you want.

## License

This project is licensed under the [MIT License](LICENSE).
