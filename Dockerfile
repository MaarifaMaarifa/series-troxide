FROM rust AS builder
RUN apt update && apt -y install libgtk-3-dev
RUN cargo install series-troxide

FROM ghcr.io/linuxserver/baseimage-kasmvnc:ubuntujammy

ENV TITLE=series-troxide

RUN apt update && apt install libgtk-3-dev mesa-vulkan-drivers -y
COPY --from=builder /usr/local/cargo/bin/series-troxide /series-troxide
COPY /assets/logos/series-troxide.ascii /etc/s6-overlay/s6-rc.d/init-adduser/branding

RUN mkdir -p /defaults/ && echo "/series-troxide" > /defaults/autostart && \
    sed -i 's|</applications>|  <application title="Series Troxide" type="normal">\n    <maximized>yes</maximized>\n  </application>\n</applications>|' /etc/xdg/openbox/rc.xml 

EXPOSE 3000

VOLUME /config