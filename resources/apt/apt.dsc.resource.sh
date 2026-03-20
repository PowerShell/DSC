#!/bin/bash

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

get_apt() {
    pkgname=$1
    InstalledSection=0
    apt list --installed $pkgname 2>&1 | while read line; do
        if [[ $line == Listing* ]]; then
            InstalledSection=1
        elif [[ $InstalledSection = 1 ]]; then
            echo $line | awk '{
                split($0, a, " ");
		split(a[1], pn, "/");
                printf("{ \"_exist\": %s, \"packageName\": \"%s\", \"version\": \"%s\", \"source\": \"%s\" }\n", ENVIRON["exist"], pn[1], a[2], pn[2]);
            }'
        fi
    done
}

if [[ "$#" -eq "0" ]]; then
    echo "Command not provided, valid commands: get, set, export"
    exit 1
elif [[ "$1" == "get" ]]; then
    check_args
    output="$(get_apt $packageName)"
    if [[ -z $output ]]; then
	    printf '{"_exist":"false","packageName":"%s","version":"","source":""}\n' $packageName
    else
	    echo $output
    fi
elif [[ "$1" == "set" ]]; then
    check_args
    if [[ -z $_exist ]]; then
        # if $_exist is not defined in the input, it defaults to `true`
        _exist=true
    fi
    if [[ $_exist = true ]]; then
        apt install -y "${packageName}"
    else
        apt remove -y "${packageName}"
    fi
elif [[ "$1" == "export" ]]; then
    get_apt
else
    echo "Invalid command, valid commands: get, set, export"
    exit 1
fi
