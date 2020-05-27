use {
    agate::{Server, Request, Result},
    async_std::{
        io::prelude::*,
        path::PathBuf,
        task::block_on,
    },
    once_cell::sync::Lazy,
    std::{
        ffi::OsStr,
        fs::File,
        io::BufReader,
    },
};

struct Args {
    sock_addr: String,
    content_dir: String,
    cert_file: String,
    key_file: String,
}

fn args() -> Option<Args> {
    let mut args = std::env::args().skip(1);
    Some(Args {
        sock_addr: args.next()?,
        content_dir: args.next()?,
        cert_file: args.next()?,
        key_file: args.next()?,
    })
}
const ARGS: Lazy<Args> = Lazy::new(|| args()
    .expect("usage: agate <addr:port> <dir> <cert> <key>"));

fn main() -> Result {
    let cert = BufReader::new(File::open(&ARGS.cert_file)?);
    let key = BufReader::new(File::open(&ARGS.key_file)?);

    block_on(async {
        let server = Server::bind(&ARGS.sock_addr, cert, key).await?;
        server.serve(handle_request).await
    })
}

async fn handle_request(request: Request) {
    if let Err(e) = send_response(request, &ARGS.content_dir).await {
        eprintln!("{}", e);
    }
}

/// Send the client the file located at the requested URL.
async fn send_response(request: Request, dir: &str) -> Result {
    let Request { url, mut tls_stream } = request;
    let mut path = PathBuf::from(dir);
    if let Some(segments) = url.path_segments() {
        path.extend(segments);
    }
    if path.is_dir().await {
        if url.as_str().ends_with('/') {
            path.push("index.gemini");
        } else {
            // Redirect to add a missing slash.
            tls_stream.write_all(b"31 ").await?;
            tls_stream.write_all(url.as_str().as_bytes()).await?;
            tls_stream.write_all(b"/\r\n").await?;
            return Ok(())
        }
    }
    match async_std::fs::read(&path).await {
        Ok(body) => {
            if path.extension() == Some(OsStr::new("gemini")) {
                tls_stream.write_all(b"20 text/gemini\r\n").await?;
            } else {
                let mime = tree_magic::from_u8(&body);
                let header = format!("20 {}\r\n", mime);
                tls_stream.write_all(header.as_bytes()).await?;
            }
            tls_stream.write_all(&body).await?;
        }
        Err(e) => {
            tls_stream.write_all(b"51 Not found, sorry.\r\n").await?;
            Err(e)?
        }
    }
    Ok(())
}
