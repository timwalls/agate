use {
    agate::{Server, Request, Result},
    async_std::{
        io::prelude::*,
        net::{TcpListener, TcpStream},
        path::PathBuf,
        stream::StreamExt,
        task::{block_on, spawn},
    },
    async_tls::TlsAcceptor,
    once_cell::sync::Lazy,
    std::{error::Error, ffi::OsStr, marker::Unpin, str, sync::Arc},
    url::Url,
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

fn main() -> Result {
    let args = args().expect("usage: agate <addr:port> <dir> <cert> <key>");

    let cert = BufReader::new(File::open(&args.cert_file)?);
    let key = BufReader::new(File::open(&args.key_file)?);

    block_on(async {
        let server = Server::bind(args.sock_addr, cert, key).await?;
        let mut incoming = listener.incoming();
        while let Some(request) = incoming.next().await {
            spawn(async {
                if let Err(e) = connection(stream).await {
                    eprintln!("Error: {:?}", e);
                }
            });
        }
        Ok(())
    })
}

/// Handle a single client session (request + response).
async fn connection(stream: TcpStream) -> Result {
    match parse_request(&mut stream).await {
        Ok(url) => {
            eprintln!("Got request for {:?}", url);
            send_response(&url, &mut stream).await
        }
        Err(e) => {
            stream.write_all(b"59 Invalid request.\r\n").await?;
            Err(e)
        }
    }
}

/// Send the client the file located at the requested URL.
async fn send_response<W: Write + Unpin>(url: &Url, mut stream: W) -> Result {
    let mut path = PathBuf::from(&ARGS.content_dir);
    if let Some(segments) = url.path_segments() {
        path.extend(segments);
    }
    if path.is_dir().await {
        if url.as_str().ends_with('/') {
            path.push("index.gemini");
        } else {
            return redirect_slash(url, stream).await;
        }
    }
    match async_std::fs::read(&path).await {
        Ok(body) => {
            if path.extension() == Some(OsStr::new("gemini")) {
                stream.write_all(b"20 text/gemini\r\n").await?;
            } else {
                let mime = tree_magic::from_u8(&body);
                let header = format!("20 {}\r\n", mime);
                stream.write_all(header.as_bytes()).await?;
            }
            stream.write_all(&body).await?;
        }
        Err(e) => {
            stream.write_all(b"51 Not found, sorry.\r\n").await?;
            Err(e)?
        }
    }
    Ok(())
}

/// Send a redirect when the URL for a directory is missing a trailing slash.
async fn redirect_slash<W: Write + Unpin>(url: &Url, mut stream: W) -> Result {
    stream.write_all(b"31 ").await?;
    stream.write_all(url.as_str().as_bytes()).await?;
    stream.write_all(b"/\r\n").await?;
    return Ok(())
}
