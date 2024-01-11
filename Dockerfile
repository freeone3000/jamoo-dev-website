# syntax=docker/dockerfile:1
FROM rust:1.69.0-alpine3.17 as builder
COPY . .
RUN cargo build --release

FROM alpine:3.17
WORKDIR /website
COPY --from=builder /target/release/jamoo-website-dev .
EXPOSE 3000
CMD ["./jamoo-website-dev"]
