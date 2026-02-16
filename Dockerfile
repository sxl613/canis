FROM rust:alpine3.23 AS builder
WORKDIR /code
COPY . /code
RUN cargo build -r

FROM alpine:3.23.3 AS canis
COPY --from=builder /code/target/release/canis /usr/local/bin/canis
ENTRYPOINT ["/usr/local/bin/canis"]
