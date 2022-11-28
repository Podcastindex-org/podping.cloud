# Podping.cloud

Podping.cloud is the hosted front-end to the [Podping](https://github.com/Podcastindex-org/podping) notification 
system.  It stands in front of the back-end writer(s) to provide a more friendly HTTP based API.

<br>

## Overview

There are two main components of a podping.cloud node.  The first is a web HTTP front-end just called `podping` that 
accepts GET requests like so:

```http
GET https://podping.cloud/?url=https://feeds.example.org/podcast/rss
```

You can also append 2 additional parameters (`reason` and/or `medium`):

```http
GET https://podping.cloud/?url=https://feeds.example.org/livestream/rss&reason=live&medium=music
```

If `reason` is not present, the default is "update".  If `medium` is not present, the default is "podcast".  A full
explanation of these options and what they mean  
is [here](https://github.com/Podcastindex-org/podping-hivewriter#podping-reasons).

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

A separate thread runs in a loop as a queue checker and does the following:

1. Checks the `queue.db` database and fetches up to 1000 feeds at a time in FIFO order ("live" reason is prioritized).
2. Checks the ZEROMQ tcp socket to the `hive-writer` listener on port `9999`.
3. Construct one or more `Ping` objects in protocol buffers to send over the socket to the writer(s).
4. Sends the `Ping` objects to `hive-writer` socket for processing and waits for success or error to be returned.
5. If success is returned from the writer, the url is removed from `queue.db`.
6. If an error is returned or an exception is raised, another attempt is made after 180 seconds.

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

The best way to run a podping.cloud node is with docker compose.  There is a [docker] folder for this.  Just clone this
repo, switch to the `docker` folder and issue `docker compose up`.  It is expected that the database files will live in
a directory called `/data`.  If this directory doesn't exist, you will need to create it.

Initially, the `auth.db` and `queue.db` will be blank.  You will need to populate the "publishers" table in the
`auth.db` file to have a funcional system.  See the example files in the `databases` directory in this repo for an 
example of the format for publisher token records.

<br>

## The Podping Network Idea

![Framework Overview 1](framework1.png)
