use std::path::PathBuf;

pub async fn compute_file_sha512_streaming(path: &PathBuf) -> Result<String, std::io::Error> {
    use sha2::{Digest, Sha512};
    use tokio::{
        fs::File,
        io::{AsyncReadExt, BufReader},
    };

    let file = File::open(path).await?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha512::new();
    let mut buf = [0u8; 8192];

    loop {
        let n = reader.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

pub async fn compute_file_sha1_streaming(path: &PathBuf) -> Result<String, std::io::Error> {
    use sha1::{Digest, Sha1};
    use tokio::{
        fs::File,
        io::{AsyncReadExt, BufReader},
    };

    let file = File::open(path).await?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha1::new();
    let mut buf = [0u8; 8192];

    loop {
        let n = reader.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}
