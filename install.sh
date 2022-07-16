#!/bin/bash

if [[ "$1" != "--prefix" ]] ; then
    echo -e "Usage:\n  $0 --prefix PREFIX"
    exit -1
fi

prefix="$2"

if [[ -z "$prefix" ]] ; then
    echo -e "Missing PREFIX operand"
    exit -1
fi

if [[ -z "$SUDO_USER" ]] ; then
    cargo build --release
else
    echo "Detected as sudo. Build as normal user: $SUDO_USER"
    su "$SUDO_USER" -c "cargo build --release"
fi

echo "Install in '$prefix'" && \
    mkdir -p "$prefix" && \
    mkdir -p "$prefix/bin" && \
    mkdir -p "$prefix/lib/systemd/user" && \
    install ./target/release/i3-autolayout "${prefix}/bin" && \
    sed "s#ExecStart=i3-autolayout#ExecStart=${prefix}/bin/i3-autolayout#g" \
        ./systemd/i3-autolayout.service \
        > "$prefix/lib/systemd/user/i3-autolayout.service"
