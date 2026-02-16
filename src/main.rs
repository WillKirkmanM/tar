use clap::{Parser, Subcommand};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs::File;
use std::path::PathBuf;

// tar
#[derive(Parser)]
#[command(name = "rtar")]
#[command(about = "A simple tar implementation in Rust", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create an archive from a list of files/directories
    Create {
        /// The name of the output tar file (e.g., archive.tar.gz)
        #[arg(short, long)]
        file: PathBuf,

        /// List of files or directories to add to the archive
        #[arg(required = true)]
        inputs: Vec<PathBuf>,
    },
    /// Extract an archive to the current directory
    Extract {
        /// The name of the archive file to extract
        #[arg(short, long)]
        file: PathBuf,

        /// Optional output directory (defaults to current dir)
        #[arg(short = 'C', long)]
        output_dir: Option<PathBuf>,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Create { file, inputs } => {
            create_archive(&file, &inputs)?;
            println!("Successfully created archive: {:?}", file);
        }
        Commands::Extract { file, output_dir } => {
            let out = output_dir.unwrap_or_else(|| PathBuf::from("."));
            extract_archive(&file, &out)?;
            println!("Successfully extracted {:?} to {:?}", file, out);
        }
    }

    Ok(())
}

fn create_archive(archive_path: &PathBuf, inputs: &[PathBuf]) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create the output file
    let file = File::create(archive_path)?;
    
    // 2. Wrap it in a Gzip encoder (default compression)
    let enc = GzEncoder::new(file, Compression::default());
    
    // 3. Wrap that in a Tar builder
    let mut tar = tar::Builder::new(enc);

    // 4. Add data to the archive
    for input in inputs {
        if input.is_dir() {
            // Recursively add directory contents
            tar.append_dir_all(input, input)?;
        } else if input.is_file() {
            // Add a single file
            tar.append_path(input)?;
        } else {
            eprintln!("Skipping {:?}: not a file or directory", input);
        }
    }

    // 5. Finish writing the archive
    tar.finish()?;
    
    Ok(())
}

fn extract_archive(archive_path: &PathBuf, output_dir: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Open the archive file
    let file = File::open(archive_path)?;

    // 2. Wrap it in a Gzip decoder
    let tar = GzDecoder::new(file);

    // 3. Create a Tar archive wrapper
    let mut archive = tar::Archive::new(tar);

    // 4. Unpack to the destination
    // unpack() preserves file permissions and directory structures
    archive.unpack(output_dir)?;

    Ok(())
}