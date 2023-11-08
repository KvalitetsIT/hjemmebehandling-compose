#!/bin/bash
service=$1
docker-compose stop $service && docker-compose rm $service && docker-compose up -d $serivce

