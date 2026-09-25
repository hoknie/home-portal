#!/bin/sh
set -eu

printf 'arguments:'
for argument in "$@"; do
  printf ' [%s]' "$argument"
done
printf '\n'

env | grep '^PORTAL_' | sort

printf 'event: '
cat
printf '\n'
