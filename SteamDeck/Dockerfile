ARG PLATFORM=linux/amd64
FROM --platform=${PLATFORM} rust:1-slim-bookworm

# For building C code
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config build-essential cmake

# For building SDL (See: https://github.com/libsdl-org/SDL/blob/main/docs/README-linux.md#build-dependencies)
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential git make \
    pkg-config cmake ninja-build gnome-desktop-testing libasound2-dev libpulse-dev \
    libaudio-dev libfribidi-dev libjack-dev libsndio-dev libx11-dev libxext-dev \
    libxrandr-dev libxcursor-dev libxfixes-dev libxi-dev libxss-dev libxtst-dev \
    libxkbcommon-dev libdrm-dev libgbm-dev libgl1-mesa-dev libgles2-mesa-dev \
    libegl1-mesa-dev libdbus-1-dev libibus-1.0-dev libudev-dev libthai-dev libusb-1.0-0-dev \
    libpipewire-0.3-dev libwayland-dev libdecor-0-dev liburing-dev

# Our RTLSDR deps
RUN apt-get update && apt-get install -y --no-install-recommends \
    libusb-1.0-0-dev \
    librtlsdr-dev

WORKDIR /workspace/app

# CMD ["/bin/bash"]
CMD ["make", "build.steamdeck.local"]
