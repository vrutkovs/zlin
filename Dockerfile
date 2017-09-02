FROM fedora:26

RUN dnf -y update --refresh && \
    dnf -y install binutils gcc glibc-devel file sudo make && \
    dnf clean all && \
    curl -s https://static.rust-lang.org/rustup.sh | sh -s -- \
        --channel=nightly --prefix=/usr

WORKDIR /zlin
COPY Cargo.toml Cargo.lock /zlin/
RUN mkdir src && touch src/lib.rs && cargo build --release --lib && rm -rf src
COPY src /zlin/src
COPY static /zlin/static
COPY templates /zlin/templates
RUN cargo build --release

VOLUME ["/zlin/upload"]

EXPOSE 80

CMD ROCKET_ENV=production cargo run --release
