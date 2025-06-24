use clap::{Parser, Subcommand};
use std::{ffi::CStr, fs, io::{BufRead, BufReader, Read, Write}};
use flate2::read::ZlibDecoder;
use anyhow::{ Context};


#[derive(Debug, Subcommand)]
enum Command {
    /// The init command initializes a `rit` repository
    Init,

    CatFile {
        /// A pretty print flag.
        #[arg(short = 'p')]
        pretty_print: bool,

        /// The Hash of the object we're to print.
        #[arg(value_parser = hash_length_validation)]
        object_hash: String,
    },
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Pass subcommands.
    #[command(subcommand)]
    command: Option<Command>,

    /// Get a verbose response
    #[arg(short, long)]
    verbose: bool,
}

/// Validate the length and format of the hash.
fn hash_length_validation(hash: &str) -> Result<String, String> {
    const MIN_LENGTH: usize = 4; // Git allows partial hashes
    const MAX_LENGTH: usize = 40; // Full SHA-1 hash

    if hash.len() < MIN_LENGTH {
        return Err(format!("Hash too short. Minimum length: {MIN_LENGTH}"));
    }

    if hash.len() > MAX_LENGTH {
        return Err(format!("Hash too long. Maximum length: {MAX_LENGTH}"));
    }

    // Validate that hash contains only hexadecimal characters
    if !hash.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("Hash must contain only hexadecimal characters (0-9, a-f, A-F)".to_string());
    }

    Ok(hash.to_lowercase()) // Git hashes are typically lowercase
}

fn initialize() {
    fs::create_dir(".rit").expect("SHOULD NOT FAIL");
    fs::create_dir(".rit/objects").expect("SHOULD NOT FAIL");
    fs::create_dir(".rit/refs").expect("SHOULD NOT FAIL");
    fs::write(".rit/HEAD", "ref: refs/heads/main\n").unwrap();
    println!("Initialized rit directory");
}

enum Kind{
    Blob
}

fn cat_file(hash: String)-> anyhow::Result<(), anyhow::Error> {
    let file = fs::File::open(
        format!(
            ".rit/objects/{}/{}", &hash[..2], &hash[2..]
        )
    ).context("open in .rit/objects")?;

    let z = ZlibDecoder::new(file);
    let mut z = BufReader::new(z);
    let mut buf = Vec::new(); 
    
    // Debug: let's see what we're actually reading
    let bytes_read = z.read_until(0, &mut buf)
        .context("reader header from .rit/objects")?;
    
    println!("Debug: Read {} bytes: {:?}", bytes_read, buf);
    
    // Check if we actually have a null byte
    if buf.is_empty() || buf[buf.len() - 1] != 0 {
        anyhow::bail!("No null terminator found in object header");
    }
    
    // Get a Cstr from the buffer. 
    let header = CStr::from_bytes_with_nul(&buf)
        .context("Failed to create CStr from buffer")?;

    // Convert the content of the Cstr to valid UTF-8
    let header = header
            .to_str()
            .context(".rit/objects file header is not valid UTF-8")?;  

    // Split on the first " " to get the "kind". 
    let Some((kind, size)) = header.split_once(" ") else {
        anyhow::bail!(".rit/objects file header did not start with a known type: '{header}'"); 
    };

    let kind = match kind {
        "blob" => Kind::Blob,
        _ =>  anyhow::bail!("We do not yet know how to rpint a kind {kind}") 
    }; 
    // Convert the size to a value; 
    let size: usize = size.parse().context(format!(".rit/objects file header has invalid size: {}", size))?;    
    // Clear the buffer that previously contained the null-terminated string. 
    buf.clear(); 
    // Resize to "size" space in the buffer for the content of the file. 
    buf.resize(size, 0);  
    // Read the actual content of the file to the buffer. 
    z.read_exact(&mut buf[..])
        .context(".rit/objects file contents did not match expectations")?; 

    // Let's append a new line to the end of the buffer.
    buf.extend_from_slice("\n".as_bytes());

    let n = z.read(&mut [0]).context("Validate EOF in .rit/objects")?;

    // Ensure we've got no trailing bytes. 
    anyhow::ensure!(n == 0, ".rit/object file had {n} trailing bytes"); 
                // Get the standard output. 
                let stdout = std::io::stdout(); 
                // Lock it and return writeable guard. 
                let mut stdout = stdout.lock(); 
    match kind {
        Kind::Blob => {
                // Write all the data to the stdout. 
                stdout.write_all(&buf).context("write object content to stdout")?; 
        },
    }

    Ok(())

}



fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.command {
        Some(Command::Init) => initialize(),
        Some(Command::CatFile {
            pretty_print,
            object_hash,
        }) => {
            if pretty_print {
                println!("Pretty printing object: {}", object_hash);
            } else {
                println!("Raw object: {}", object_hash);
            }
            cat_file(object_hash)?; 
        }
        None => {
            println!("No command provided. Use --help for usage information.");
        }
    }

    Ok(())
}
