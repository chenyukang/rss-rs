FROM ubuntu:20.04

RUN apt-get update

RUN mkdir /rss-rs
WORKDIR /rss-rs

COPY ./target/debug/rss-rs ./

RUN apt-get install build-essential -y
RUN ./rss-rs -d

RUN apt remove --purge nodejs npm
RUN apt clean
RUN apt install -f
RUN apt autoremove
RUN apt-get install curl -y
RUN curl -sL https://deb.nodesource.com/setup_14.x | bash -
RUN apt-get update -y
RUN apt-get install nodejs yarn -y
RUN node -v
RUN npm -v

RUN mkdir /rss-rs/front
WORKDIR /rss-rs/front
COPY ./front ./
RUN npm install
RUN npm run build

