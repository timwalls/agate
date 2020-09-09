# Agate

## Simple Gemini server for static files

Agate is a server for the [Gemini] network protocol, built with the [Rust] programming language.
Agate has very few features, and can only serve static files. It uses async I/O, and should be quite efficient even when running on low-end hardware and serving many concurrent requests.

This is a fork of the original project developed by [mbrubeck], and packaged
as a Docker container by [Tim Walls](https://snowgoons.ro/).

## References

* Original homepage: [gemini://gem.limpet.net/agate/][originalhome]
* [Source code][source]

## What's in this image

This image contains the basic `agate` server process, configured and ready
to run.  The image looks for the following files:

| Path | Content |
| ---- | ------- |
| `/usr/local/gemini/conf` | Your TLS certificate and key, as `gemini-cert.pem` and `gemini-key.rsa` |
| `/usr/local/gemini/geminidocs` | Your static content to serve.  The 'default' file served is `index.gmi` |

> *Important note:* A default - and likely expired - TLS certificate is included
> in the image, just so you can get a server up and running with a simple
> `docker run` command.  But you *must* replace this with your own certificate
> before using in any kind of 'production', or anyone browsing your gemini
> site will be confronted with certificate expired errors.

The server exposes and listens on the 'standard' Gemini port, TCP:1965.

## Using the image with a Dockerfile in your own project
Create a Dockerfile like so:
```
FROM snowgoons/agate:latest
COPY ./geminidocs /usr/local/gemini/geminidocs
COPY cert.pem /usr/local/gemini/conf/gemini-cert.pem
COPY key.rsa  /usr/local/gemini/conf/gemini-key.rsa
```

Then build and run the Docker image:

```
docker build -t my-gemini-site .
docker run -d --name my-gemini-server -p 1965:1965 my-gemini-site
```

You should be able to point your Gemini browser to gemini://localhost/ and see
that it works.

## Generating TLS keys
Gemini works fine with self-signed TLS keys - so go ahead and generate your
own using [openssl]:
 
```
openssl req -x509 -newkey rsa:4096 -keyout gemini-key.rsa -out gemini-cert.pem \
    -days 3650 -nodes -subj "/CN=my-gemini-host.com"
```



[mbrubeck]: https://github.com/mbrubeck/
[Gemini]: https://gemini.circumlunar.space/
[Rust]: https://www.rust-lang.org/
[originalhome]: gemini://gem.limpet.net/agate/
[source]: https://github.com/timwalls/agate
[openssl]: https://www.openssl.org/
