FROM fedora:26

RUN dnf -y update --refresh && \
    dnf -y install rust cargo

WORKDIR /zlin
COPY Cargo.toml Cargo.lock /zlin/
RUN mkdir src && touch src/lib.rs && cargo build --release --lib && rm -rf src
COPY src /zlin/src
RUN cargo build --release

VOLUME ["/zlin/upload"]

EXPOSE 8000

CMD ["env", "ROCKET_ENV=production", "cargo", "run", "--release"]
