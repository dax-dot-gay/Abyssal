#! /bin/sh

COUNTRY="US"
STATE="NY"
CITY="Rochester"
ORG="dax-dot-gay"

if [ ! -d "crates/abyssal/certs" ]; then
    mkdir -p crates/abyssal/certs
    openssl req -x509 -newkey rsa:4096 -keyout crates/abyssal/certs/key.pem -out crates/abyssal/certs/cert.pem -sha256 -days 3650 -nodes -subj "/C=$COUNTRY/ST=$STATE/L=$CITY/O=$ORG/CN=abyssal-server"
fi

if [ ! -d "client/abyssal/certs" ]; then
    mkdir -p client/abyssal/certs
    openssl req -x509 -newkey rsa:4096 -keyout client/abyssal/certs/key.pem -out client/abyssal/certs/cert.pem -sha256 -days 3650 -nodes -subj "/C=$COUNTRY/ST=$STATE/L=$CITY/O=$ORG/CN=abyssal-client"
fi