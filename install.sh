#!/bin/bash

if [[ "${1}" != "--prefix" ]] ; then
    echo -e "Usage:\n  ${0} --prefix PREFIX"
    exit -1
fi

prefix=$(echo "${2}" | sed 's:/*$::')

if [[ -z "${prefix}" ]] ; then
    echo -e "Missing PREFIX operand"
    exit -1
fi


if [[ -z "${SUDO_USER}" ]] ; then
    THEUSER="${USER}"
    cargo build --release
else
    THEUSER="${SUDO_USER}"
    echo "Detected as sudo. Build as normal user: ${SUDO_USER}"
    su "${SUDO_USER}" -c "cargo build --release"
fi

echo "Install in '${prefix}'" && \
    mkdir -p "${prefix}" && \
    mkdir -p "${prefix}/bin" && \
    mkdir -p "${prefix}/lib/systemd/user" && \
    mkdir -p "${prefix}/env" && \
    install ./target/release/i3-autolayout "${prefix}/bin" && \
    sed "s#ExecStart=i3-autolayout#ExecStart=${prefix}/bin/i3-autolayout#g" \
        ./systemd/i3-autolayout.service \
        > "${prefix}/lib/systemd/user/i3-autolayout.service" && \
    sed "s#AUTOLAYOUT_BIN_DIR=%%%#AUTOLAYOUT_BIN_DIR=${prefix}/bin#g" \
        ./env/env \
        > "${prefix}/env/env"

THEHOME=$(eval echo "~${THEUSER}")

read -r -p "Source environment for user '${THEUSER}'? [y/N]: " ans
case "${ans}" in
    [yY])
        line=". ${prefix}/env/env"
        envs=".profile .bashrc"

        for vv in ${envs}; do
            file="${THEHOME}/${vv}"
            if [[ -f "${file}" ]]; then
                if ! grep -Fxq "${line}" "${file}"
                then
                    echo -e "\n${line}" >> "${file}"
                    echo "  Sourced file '${file}'"
                fi
            fi
        done
        ;;
    *)
        ;;
esac
