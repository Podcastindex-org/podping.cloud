#!/usr/bin/env bash

export PPUSERAGENT="Awesome Hosting Company"
export PPAUTHORIZATION="31D0T5IwXp6UNpNyK6r2i52RYucZJKYXUPpSey6YWK8t"


wget --header="User-Agent: $PPUSERAGENT" --header="Authorization: $PPAUTHORIZATION" https://podping.cloud/?url=http://feeds.example.com/podcast.xml -O/dev/null
