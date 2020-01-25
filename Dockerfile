FROM rust:1.40

WORKDIR /usr/src/mbtileserver
COPY . .

RUN mkdir /tiles

RUN cargo install --path .

CMD ["mbtileserver -d /tiles"]