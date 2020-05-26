use {
    async_std::{
        io::prelude::*,
        net::{TcpListener, TcpStream, ToSocketAddrs},
    },
    async_tls::{TlsAcceptor, server::TlsStream},
    futures::stream::{Stream, TryStreamExt},
    std::{
        error::Error,
        str,
        sync::Arc
    },
    url::Url,
};

pub type Result<T=()> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

pub struct Server {
    tcp_listener: TcpListener,
    tls_acceptor: TlsAcceptor,
}

pub struct Request {
    pub url: Url,
    pub tls_stream: TlsStream<TcpStream>,
}

impl Server {
    pub async fn bind<R>(addr: impl ToSocketAddrs, cert_file: R, key_file: R) -> Result<Self>
    where
        R: std::io::BufRead,
    {
        Ok(Server {
            tcp_listener: TcpListener::bind(addr).await?,
            tls_acceptor: tls_acceptor(cert_file, key_file)?,
        })
    }

    pub fn incoming(&self) -> impl Stream<Item = Result<Request>> + '_ {
        self.tcp_listener.incoming()
            .map_err(|e| e.into())
            .and_then(move |tcp_stream| self.parse_request(tcp_stream))
    }

    /// Return the URL requested by the client.
    async fn parse_request(&self, tcp_stream: TcpStream) -> Result<Request> {
        // TLS handshake.
        let mut tls_stream = self.tls_acceptor.accept(tcp_stream).await?;

        // Because requests are limited to 1024 bytes (plus 2 bytes for CRLF), we
        // can use a fixed-sized buffer on the stack, avoiding allocations and
        // copying, and stopping bad clients from making us use too much memory.
        let mut request = [0; 1026];
        let mut buf = &mut request[..];
        let mut len = 0;

        // Read until CRLF, end-of-stream, or there's no buffer space left.
        loop {
            let bytes_read = tls_stream.read(buf).await?;
            len += bytes_read;
            if request[..len].ends_with(b"\r\n") {
                break;
            } else if bytes_read == 0 {
                Err("Request ended unexpectedly")?
            }
            buf = &mut request[len..];
        }
        let request = str::from_utf8(&request[..len - 2])?;

        // Handle scheme-relative URLs.
        let url = if request.starts_with("//") {
            Url::parse(&format!("gemini:{}", request))?
        } else {
            Url::parse(request)?
        };

        // Validate the URL. TODO: Check the hostname and port.
        if url.scheme() != "gemini" {
            Err("unsupported URL scheme")?
        }
        Ok(Request { url, tls_stream })
    }
}

fn tls_acceptor<R>(mut cert_file: R, mut key_file: R) -> Result<TlsAcceptor>
where
    R: std::io::BufRead,
{
    use rustls::internal::pemfile::{certs, pkcs8_private_keys};
    let certs = certs(&mut cert_file).or(Err("bad cert"))?;
    let mut keys = pkcs8_private_keys(&mut key_file).or(Err("bad key"))?;

    let mut config = rustls::ServerConfig::new(rustls::NoClientAuth::new());
    config.set_single_cert(certs, keys.remove(0))?;
    Ok(TlsAcceptor::from(Arc::new(config)))
}
