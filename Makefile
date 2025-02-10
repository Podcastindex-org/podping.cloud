# Created By: suorcd

SHELL := /usr/bin/env bash


.PHONY: podping_container
podping_container:
	docker build -f ./docker/Dockerfile -t podcastindexorg/podcasting20-podping.cloud .
