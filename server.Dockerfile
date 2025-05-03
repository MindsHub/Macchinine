FROM rust:alpine
WORKDIR /usr/src/app
RUN apk update
RUN apk add --no-cache git
RUN apk add --no-cache musl-dev
RUN git clone https://github.com/johanhelsing/matchbox.git
RUN cd matchbox && cargo build --release --bin matchbox_server
RUN cp matchbox/target/release/matchbox_server .

ENTRYPOINT [ "./matchbox_server"]