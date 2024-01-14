# syntax=docker/dockerfile:1
FROM docker.io/rust:1.74-alpine3.18 as builder
RUN apk add --no-cache musl-dev
COPY . .
RUN cargo build --release

FROM docker.io/alpine:3.17
WORKDIR /website
COPY --from=builder /target/release/jamoo-website-dev .
EXPOSE 3000
CMD ["./jamoo-website-dev"]
