FROM rust:1.40

WORKDIR /usr/src/mbtileserver
COPY . .

RUN cargo install --path .
CMD ["mbtileserver"]