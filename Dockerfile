FROM rust:1.46.0
MAINTAINER Mesosphere <support@mesosphere.io>

ADD . /star
WORKDIR /star

RUN cargo build
ENTRYPOINT ["target/debug/star-probe"]
