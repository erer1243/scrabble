#!/bin/sh
# Digest a directory into a sha1 hash
find "$1" -type f -exec sha1sum '{}' ';' | awk '{print $1}' | sort | sha1sum
