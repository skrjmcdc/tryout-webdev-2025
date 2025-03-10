FROM ubuntu:25.04

RUN apt-get -y update
RUN apt-get -y upgrade
RUN apt-get -y install curl

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | bash -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN cargo --version

RUN apt-get -y install git
RUN git clone "https://github.com/skrjmcdc/tryout-webdev-2025.git"
WORKDIR tryout-webdev-2025

RUN mkdir data
RUN mkdir data/tryouts

RUN apt-get -y install build-essential
RUN cargo build

EXPOSE 12345

CMD ["cargo", "run"]
