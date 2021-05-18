# Podping.cloud
The server code that runs a podping.cloud node.

<br>

## Node Overview

There are two main components of a podping.cloud node.  The first is a web HTTP front-end just called `podping` that accepts GET requests like so:

```http
GET https://podping.cloud/?url=https://feeds.example.org/podcast/rss
```

The next component is `hive-writer` - a python script that listens on localhost port `5555` for incoming urls terminated by a newline character.  When it receives one, it attemps to write it as a custom JSON notification message to the Hive blockchain.

<br>

## Web Front-End (podping.rs)

The front-end accepts this request and does a few things:

1. Ensures that the sending publisher has included a valid 'Authorization' header token.
2. Validates that the format of the given podcast feed url looks sane
3. Saves the url into the `queue.db` sqlite database in the `queue` table.
4. Returns `200` to the sending publisher.

A separate thread runs in a loop every 3 seconds as a queue checker and does the following:

1. Checks the `queue.db` database fetches 10 feeds at a time in FIFO order.
2. Opens a ZEROMQ tcp socket to the `hive-writer` listener on port `5555`.
3. Sends the url string to `hive-writer` socket for processing and waits for "OK" to be returned.
4. If "OK" is returned from the hive writer python script, the url is removed from `queue.db`.
5. If "ERR" is returned or an exception is raised, another attempt is made on the next cycle.

The web front-end looks up incoming request 'Authorization' header tokens in the `auth.db` sqlite db to ensure they are valid before processing the incoming HTTP GET request.

There is a dummy auth token in the `auth.db` that is ready to use for testing.  The token value is:

<br>

```text
Blahblah^^12345678
```

In order to avoid running as a root user, please set the `PODPING_RUNAS_USER` environment variable to the non-root user you want the
front-end executable to run as.  Something like this:

```bash
PODPING_RUNAS_USER="podping" ./target/release/podping
```

<br>

## Blockchain Writer (hive-writer.py)

The python script `hive-writer.py` does the following:

### Enviornment

In order to operate, ```hive-watcher``` must be given two ENV variables. The third ENV variable will use a test version of Hive which may or may not be available:
```
"env": {
    "HIVE_SERVER_ACCOUNT" : "blocktvnews",
    "HIVE_POSTING_KEY": "5KRBCq3D7NiYH2E8AgshtthisisfakeJW4uCJWn8Qrpe9Rei2ZYx",
    "USE_TEST_NODE": "False"
}
```
Hive accounts can be created with the tool [Hive On Board](https://hiveonboard.com?ref=brianoflondon). However, only *Podpings* from accounts which are approved by Podping and PodcastIndex will be recognised. The current authorised list can always be seen [here](https://peakd.com/@podping/following).

The stream of *podpings* can be watched with the ```hive-watcher.py``` code. In addition there is a simplified version of this code ```simple-watcher.py``` which should be used to understand what is going on. There is javascript version in [hive-watcher.js](https://github.com/Podcastindex-org/podping.cloud/blob/main/hive-watcher-js/hive-watcher.js)


```
usage: hive-watcher [options]

PodPing - Watch the Hive Blockchain for notifications of new Podcast Episodes This code will run until terminated reporting every notification of a new Podcast Episode sent to the Hive blockchain by any PodPing servers.

With default arguments it will print to the StdOut a log of each new URL that has updated interspersed with summary lines every 5 minutes that list the number of PodPings and the number of other 'custom_json' operations seen on the blockchain. This interval can be set with the --reports command line.

optional arguments:
  -h, --help          show this help message and exit
  -H, --history-only  Report history only and exit
  -r , --reports      Time in MINUTES between periodic status reports, use 0 for no periodic reports
  -s , --socket       <IP-Address>:<port> Socket to send each new url to
  -t, --test          Use a test net API
  -q, --quiet         Minimal output
  -v, --verbose       Lots of output

  -b , --block        Hive Block number to start replay at or use:
  -o , --old          Time in HOURS to look back up the chain for old pings (default is 0)
```
### What it does

Hive Writer uses the [beem](https://beem.readthedocs.io/en/latest/) Python library to connect to the Hive blockchain using any one of the [currently available API nodes](https://beacon.peakd.com/). These are determined at run time and will be switched in and out if any prove unreliable.

Depending on the user options ```--zmq <port>``` or ```--socket <port>``` it will start listening on that port.

```hive-writer``` will run through a series of checks including checking that the supplied Hive account and ```posting key``` are valid and can write to the blockchain. It will also check for enough ```Resource Credits```. Writing operations to Hive does not have a financial cost, but there are resource limits based on the value of the account writing to the chain.

For the regular socket and the ZMQ socket, ```hive-watcher``` will listen for a new line terminated string. Every 3s it will write to the Hive chain including multiple URLs if they arrive in that period. This will be writen to the Hive blockchain as a ```custom_json``` operation with ```id='podping'```. On the blockchain this [results in the following](https://hiveblocks.com/tx/22d0da53aada998de9b249fba473e47b79f31c65):

```
{
    "ref_block_num": 57104,
    "ref_block_prefix": 3291545262,
    "expiration": "2021-05-18T09:09:36",
    "operations": [
        [
            "custom_json",
            {
                "required_auths": [],
                "required_posting_auths": [
                    "hivehydra"
                ],
                "id": "podping",
                "json": {
                    "version": "0.2",
                    "num_urls": 2,
                    "reason": "feed_update",
                    "urls": [
                        "https://rss.whooshkaa.com/rss/podcast/id/8209",
                        "https://feeds.buzzsprout.com/262529.rss"
                    ]
                }
            }
        ]
    ],
    "extensions": [],
    "signatures": [
        "204521cdcd6edc9a4e7f3551b8e28d811be101b0f2c4251c2bd53ef8b1403c99bd166c234ab31368f9f3b3217b17bb27660202bcf8245029f9ca8687e03c903405"
    ],
    "transaction_id": "63e8cacfc3622e166707e7307e0b728f9658b051",
    "block_num": 53993248,
    "transaction_num": 28
}
```

```hive-writer``` returns either ```OK``` or ```ERR```.


The write operation usually takes about 0.8s. At present ```hive-writer``` is not multi-threaded for write operations however this could be done.

<br>

## Running

First clone this repo.

Make sure that libzmq-dev is installed:

`apt-get install libzmq3-dev`

Build and launch podping like so:

`cd podping && crate run`

Then launch hive-write like this:

`python3 ./hive-writer/hive-writer.py`

<br>

## Important Note

This software is unstable and not ready for production.  Please do not use it except for tinkering.

The hive-writer agent requires permission keys that are not included in this repo.  They should be exported into your
shell environment before launching the script.

<br>

## The Podping Network Idea

![Framework Overview 1](framework1.png)