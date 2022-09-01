#!/usr/bin/bash

source "$(dirname "${BASH_SOURCE}")"/../bin/imosh || exit 1
eval "${IMOSH_INIT}"

main() {
    if ! sub::isset UNAGI_PASSWORD; then
        LOG ERROR "UNAGI_PASSWORD is not set"
        exit 1
    fi

    local password_hash="$(sub::sha1 "${UNAGI_PASSWORD}")"

    if [ "${password_hash:0:8}" != '4c1b5873' ]; then
        LOG ERROR "UNAGI_PASSWORD has an unexpected password."
        exit 2
    fi

    LOG INFO "UNAGI_PASSWORD is set as expected."
}

main
