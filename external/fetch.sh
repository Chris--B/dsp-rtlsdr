#!/bin/bash

cd "$(dirname "$(realpath "${BASH_SOURCE[0]}")")" || exit 1

function label() {
    echo ""
    echo -e "\033[1;33m$* \033[0m"
}

function loudly() {
    { printf '+'; printf ' %q' "$@"; printf '\n'; } >&2
    "$@"
}

function fetch_repo() {
    local url="$1" path="$2" name="$3" ref="$4"

    mkdir -p "$path"
    label "Populating $name at $(realpath "$path")"
    if [ ! -d "$path/.git" ]; then
        loudly git clone "$url" "$path"
    else
        echo "+ already cloned"
    fi

    pushd $path > /dev/null

    label "Updating tags"
    loudly git fetch --tags --prune

    if [ -n "$ref" ]; then
        label "Checking out $ref"
        loudly git checkout "$ref"
        echo
        loudly git show --summary $ref
    fi

    popd > /dev/null
}

fetch_repo https://github.com/libsdl-org/SDL.git    ./SDL       "SDL3"      "release-3.2.0"
fetch_repo https://github.com/libusb/libusb.git     ./libusb    "libusb"    "v1.0.27"
fetch_repo https://github.com/steve-m/librtlsdr.git ./librtlsdr "librtlsdr" "v2.0.2"
