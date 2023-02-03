ARG BuildEnv
FROM rust:1.67-alpine as build

RUN apk add --no-cache openssl-dev libpq-dev musl-dev

WORKDIR /usr/src/rustix
COPY src src
COPY Cargo.toml Cargo.lock diesel.toml ./

# https://github.com/sfackler/rust-native-tls/issues/190
RUN if [ "$BuildEnv" = "prod" ] ;then \
        RUSTFLAGS=-Ctarget-feature=-crt-static cargo build --release; \
    else \
        RUSTFLAGS=-Ctarget-feature=-crt-static cargo build; \
    fi

######################
# Begin artifact stage
######################

FROM alpine

RUN apk add --no-cache openssl libpq libgcc

COPY --from=build /usr/src/rustix/target/debug/rustix /usr/bin/rustix
COPY merges.txt vocab.json ./

CMD ["rustix"]
