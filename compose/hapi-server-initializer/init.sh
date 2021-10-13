#!/bin/sh

echo 'Installing curl ...';
apk add curl;

echo 'Waiting for hapi-server to be ready ...';
curl -o /dev/null --retry 5 --retry-max-time 40 --retry-connrefused http://hapi-server:8080

echo 'Initializing hapi-server ...';
curl -i -d '@/hapi-server-initializer/patient1.xml' -H 'Content-Type: application/fhir+xml' http://hapi-server:8080/fhir/Patient?_format=xml