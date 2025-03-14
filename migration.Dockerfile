FROM rust:1.81-alpine AS build

RUN apk add --no-cache openssl-dev libpq-dev musl-dev &&\
    RUSTFLAGS=-Ctarget-feature=-crt-static cargo install --root /app --version 2.0.1 diesel_cli --no-default-features --features "postgres"


######################
# Begin artifact stage
######################

FROM alpine

RUN apk add --no-cache openssl libpq libgcc

COPY --from=build /app/bin/diesel /bin

WORKDIR app
COPY migrations migrations
