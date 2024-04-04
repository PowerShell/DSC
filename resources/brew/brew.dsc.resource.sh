#!/bin/sh

# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

export exist=true
export NONINTERACTIVE=1

# $packageName and $_exist are sent as env vars by dsc converting the JSON input to name/value pairs

check_args() {
    if [[ -z $packageName ]]; then
        echo "packageName not set"
        exit 1
    fi
}

to_json() {
    while read line; do
        echo $line | awk '{print "{ \"packageName\": \""$1"\", \"version\": \""$2"\", \"_exist\": "ENVIRON["exist"]" }"}'
    done
}

if [ $# -eq 0 ]; then
    echo "Command not provided, valid commands: get, set, export"
    exit 1
elif [[ $1 == "get" ]]; then
    check_args
    output="$(brew list ${packageName} --versions)"
    if [[ $? -ne 0 ]]; then
        exist=false
        output="${packageName}"
    fi
    echo $output | to_json
elif [[ $1 == "set" ]]; then
    check_args
    if [[ -z $_exist ]]; then
        # if $_exist is not defined in the input, it defaults to `true`
        _exist=true
    fi
    if [[ $_exist = true ]]; then
        brew install "${packageName}"
    else
        brew uninstall "${packageName}"
    fi
elif [[ $1 == "export" ]]; then
    brew list --versions | to_json
else
    echo "Invalid command, valid commands: get, set, export"
    exit 1
fi