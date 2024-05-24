# BUILD STAGE !
FROM rust.:1.69-buster as builder 
WORKDIR /app
ARG DATABASE_URL
ENV DATABASE_URL=$DATABASE_URL_URL
COPY . .
RUN cargo build --release

# PRODUNCTION STAGE!
FROM debian:buster-slim
WORKDIR /user/local/bin
COPY -form=builder /app/target/release/backend ./
CMD ["./backend"]