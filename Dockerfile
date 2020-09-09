# First stage - build
FROM rust:latest AS build

# Dependencies
RUN apt-get update
RUN apt-get install openssl

# Compile the server
WORKDIR /usr/src/agate
COPY ./Cargo.* ./
COPY ./src ./src/
RUN ls -l
RUN cargo build --release

# We'll create a short-lived TLS certificate as well just to have a working
# container, although the user should substititute their own
RUN openssl req -x509 -newkey rsa:4096 -keyout gemini-key.rsa \
  -out gemini-cert.pem -days 2 -nodes -subj "/CN=agate.docker"

# Second stage - release
FROM debian:buster-slim

WORKDIR /usr/local/gemini
COPY ./index.gmi geminidocs/index.gmi
COPY --from=build /usr/src/agate/target/release/agate /usr/local/bin
COPY --from=build /usr/src/agate/gemini-key.rsa   conf/gemini-key.rsa
COPY --from=build /usr/src/agate/gemini-cert.pem  conf/gemini-cert.pem

# And finally, run the thing
EXPOSE 1965/tcp
CMD [ "agate", "0.0.0.0:1965", "geminidocs", "conf/gemini-cert.pem", "conf/gemini-key.rsa" ]