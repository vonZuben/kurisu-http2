#!/bin/bash

# requires openssl to be installed
# the server files need to be put in the right place for the sever to initialize properly

# generate rootCA key, this is what signs the servers cert and is used
openssl genrsa -out rootCA.key 2048
# make the rootCA cert, this is what you put in your browser to say you trust sites by certs signed by this authority
openssl req -x509 -new -nodes -key rootCA.key -days 365 -out rootCA.crt

# now generate the servers key and make cert sign request
openssl genrsa -out server.key 2048
openssl req -new -key server.key -out server.csr

# sign our own servers cert, this will be trusted now for browsers that install our rootCA.crt
openssl x509 -req -in server.csr -CA rootCA.crt -CAkey rootCA.key -CAcreateserial -out server.crt -days 365 -extfile v3.ext

