# Podping.cloud

Podping.cloud is the hosted front-end to the Podping notification system.  It stands in front of the back-end writer(s)
to provide a more friendly HTTP based API.

<br>

## Overview

There are two main components of a podping.cloud node.  The first is a web HTTP front-end just called `podping` that 
accepts GET requests like so:

```http
GET https://podping.cloud/?url=https://feeds.example.org/podcast/rss
```

The next component is one or more back-end writers that connect to the front-end over a ZMQ socket.  Currently, the 
only back-end writer is the [hive-writer](https://github.com/Podcastindex-org/podping-hivewriter), a python script that 
listens on localhost port `9999` for incoming events.  When it receives an event, it attempts to write that event as 
a custom JSON notification message to the Hive blockchain.

<br>

## Requests

The front-end accepts GET requests and does a few things:

1. Ensures that the sending publisher has included a valid 'Authorization' header token.
2. Validates that the token exists in the `auth.db` sqlite db.
3. Validates that the format of the given podcast feed url looks sane
4. Saves the url into the `queue.db` sqlite database in the `queue` table.
5. Returns `200` to the sending publisher.

A separate thread runs in a loop every 3 seconds as a queue checker and does the following:

1. Checks the `queue.db` database and fetches 10 feeds at a time in FIFO order.
2. Checks the ZEROMQ tcp socket to the `hive-writer` listener on port `9999`.
3. Sends the url string to `hive-writer` socket for processing and waits for "OK" to be returned.
4. If "OK" is returned from the hive writer python script, the url is removed from `queue.db`.
5. If "ERR" is returned or an exception is raised, another attempt is made on the next cycle.

There is a dummy auth token in the `auth.db` that is ready to use for testing.  The token value is:

<br>

```text
Blahblah^^12345678
```

In order to avoid running as a root user, please set the `PODPING_RUNAS_USER` environment variable to the non-root 
user you want the front-end executable to run as.  Something like this:

```bash
PODPING_RUNAS_USER="podping" ./target/release/podping
```

<br>

## Back-end Writers

 - [Podping-hivewriter](https://github.com/Podcastindex-org/podping-hivewriter):  Accepts events from the podping.cloud
                        front-end or from the command line and writes them to the Hive blockchain.


<br>

## Running a Full Podping.cloud Node

First clone this repo.

Make sure that libzmq-dev is installed:

`apt-get install libzmq3-dev`

Build and launch podping like so:

`cd podping && crate run`

Then launch hive-write like this:

`python3 ./hive-writer/hive-writer.py`

<br>

## The Podping Network Idea

![Framework Overview 1](framework1.png)
