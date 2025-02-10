#!/usr/bin/env bash

docker run --rm --privileged multiarch/qemu-user-static --reset -p yes
docker buildx rm builder
docker buildx create --name builder --driver docker-container --use
docker buildx inspect --bootstrap
sudo docker buildx build --platform linux/amd64 --tag podcastindexorg/podcasting20-podping.cloud:2.1.0 --tag podcastindexorg/podcasting20-podping.cloud:latest --no-cache --output "type=registry" .
