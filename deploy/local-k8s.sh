#!/usr/bin/env zsh
set -e

REPO="localhost:5001"
# build and push
docker build -t "${REPO}/jamoo-website-dev:latest" -f ../Dockerfile ..
docker push "${REPO}/jamoo-website-dev:latest"
# and run
kubectl apply -f jamoo-website-dev.yaml