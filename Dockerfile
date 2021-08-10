FROM rust:1.54

WORKDIR /usr/src/mbtileserver
COPY . .

RUN mkdir /tiles

RUN cargo install --path .

CMD ["mbtileserver -d /tiles"]