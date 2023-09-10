FROM rust AS builder
RUN apt update && apt -y install libgtk-3-dev
RUN cargo install series-troxide

FROM ghcr.io/linuxserver/baseimage-kasmvnc:ubuntujammy

ENV TITLE=series-troxide

RUN apt update && apt install libgtk-3-dev mesa-vulkan-drivers -y
COPY --from=builder /usr/local/cargo/bin/series-troxide /series-troxide
RUN mkdir -p /defaults/ && echo "/series-troxide" > /defaults/autostart

EXPOSE 3000

VOLUME /config