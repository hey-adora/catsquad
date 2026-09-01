#!/usr/bin/env sh

set -e

POS_DATA="./target/tmp/db_posgres"
POS_SOCK="$PWD/target"

if [ ! -d $POS_DATA ]; then
  echo "INITIALIZING DATABASE $POS_DATA"
  initdb $POS_DATA
  createdb -h "localhost" catsquad
fi

echo "DATABAE STARTING $POS_DATA"
# pg_ctl -D $POS_DATA -l logfile -o "--unix_socket_directories='$POS_SOCK'" start
postgres -D $POS_DATA --unix_socket_directories=$POS_SOCK
