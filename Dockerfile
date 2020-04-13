FROM rust:latest AS builder 
WORKDIR /usr/src/app/
COPY . .
RUN cargo build --release

FROM ubuntu:18.04
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
RUN apt-get update && apt-get install ca-certificates -y && update-ca-certificates
COPY --from=builder \
    /usr/src/app/target/release/exchange-aggregator \
    /var/app/
WORKDIR /var/app
CMD ["./exchange-aggregator"]
