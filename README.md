# Podping.cloud
The server code that runs a podping.cloud node.

<br>

## Overview

There are two main components of a podping.cloud node.  The first is a web HTTP front-end just called `podping` that accepts GET requests like so:

```http
GET https://podping.cloud/?url=https://feeds.example.org/podcast/rss
```

The front-end accepts this request and does a few things:

1. Ensures that the sending publisher has included a valid 'Authorization' header token.
2. Validates that the format of the given podcast feed url looks sane
3. Opens a socket to the `hive-writer` listener on port `5000`.
4. Sends the incoming url string to `hive-writer` for processing.
5. Returns `200` to the sending publisher.

The web front-end looks up incoming request 'Authorization' header tokens in a sqlite db to ensure they are valid before
processing the request.


The next component is `hive-writer` - a python script that listens on port `5000` for incoming urls terminated by a newline character.
When it receives one, it attemps to write it as a custom JSON notification message to the Hive blockchain.

<br>

## Running

First clone this repo.

Launch podping like so:

`./target/debug/podping`

Then launch hive-write like this:

`python3 ./hive-writer/hive-writer.py`

<br>

## Important Note

This software is unstable and not ready for production.  Please do not use it.

The hive-writer agent requires permission keys that are not included in this repo. 