#!/usr/bin/bash

gcloud auth activate-service-account icfpc2022@icfpc-primary.iam.gserviceaccount.com --key-file=/service_account.json

"$@" &
pid="$!"

count=0
while :; do
    if ! kill -0 "${pid}" 2>/dev/null; then
        wait
        exit "$?"
    fi
    if [ -f /watchdog ]; then
        count=0
        rm /watchdog
    else
        : $(( count = count + 1 ))
        if (( count > 30 )); then
            echo "ERROR: watchdog failed" >&2
            exit 1
        fi
    fi
    sleep 1
done
