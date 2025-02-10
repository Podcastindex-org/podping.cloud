
ifeq ($(UNAME),Darwin)
	SHELL := /opt/local/bin/bash
	OS_X  := true
else ifneq (,$(wildcard /etc/redhat-release))
	RHEL := true
else
	OS_DEB  := true
	SHELL := /usr/bin/env bash
endif

.PHONY: podping_container
podping_container:
	docker  build -f docker/Dokcerfile -t podcastindexorg/podcasting20-podping.cloud .