#!/usr/bin/env bash

find_cddl() {
    local i filename

    # List all in directory order (breadth-first), and remove the files
    # that aren't part of the repository from the list.
    # Sort by basename.
    for i in 0 1 2 3 4 5 6 7 8; do
        find "$1" -mindepth "$i" -maxdepth "$i" -iname \*.cddl | while read -r filename
        do
            echo "$(basename "$filename"):$filename"
        done | sort -n | awk -F: '{ print $2 }'
    done
}

write_all() {
    find_cddl "$1"/spec
    find_cddl "$1"/attributes/network
    find_cddl "$1"/attributes/request
    find_cddl "$1"/attributes/response
}

root="$(dirname "$(dirname "$0")")"

# If no argument passed or empty output, just output to stdout.
if [ "$1" == '' ]; then
    write_all "$root" | xargs cat
else
    write_all "$root" | xargs cat > "$1"
fi
