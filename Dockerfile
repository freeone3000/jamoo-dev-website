# syntax=docker/dockerfile:1
FROM rust:1.69.0-alpine3.17 as builder
COPY . .
RUN cargo build --release

FROM alpine:3.17
COPY --from=builder /target/release/jamoo-website-dev .
COPY static /static
COPY templates /templates
EXPOSE 3000
CMD ["./jamoo-website-dev"]