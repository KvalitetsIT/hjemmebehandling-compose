#!/bin/sh

echo "Waiting 30 seconds for container to initialize"
sleep 30

echo "Starting application..."
exec ./migrator
