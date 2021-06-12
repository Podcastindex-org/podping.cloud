# Podping.cloud

Podping is a blockchain based global notification system for podcasting.  Feed urls are written by the publisher to the blockchain within seconds of a new episode being published.  Anyone can monitor for those updates and only pull a copy of that feed when it shows up on the chain.

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

### NOTE: Development of hive-writer.py has moved to https://github.com/Podcastindex-org/podping-hivewriter

The python script `hive-writer.py` sends podpings to Hive:

```
usage: hive-writer [options]

PodPing - Runs as a server and writes a stream of URLs to the Hive Blockchain or sends a single URL to Hive (--url option)

optional arguments:
  -h, --help      show this help message and exit
  -q, --quiet     Minimal output
  -v, --verbose   Lots of output
  -s , --socket   <port> Socket to listen on for each new url, returns either
  -z , --zmq      <port> for ZMQ to listen on for each new url, returns either
  -u , --url      <url> Takes in a single URL and sends a single podping to Hive, needs HIVE_SERVER_ACCOUNT and HIVE_POSTING_KEY ENV variables set
  -e , --errors   Deliberately force error rate of <int>%
```

There are two main modes of operation:
1. Run as a server waiting for URLs either as a simple socket (--socket) or as ZMQ socket (--zmq)
2. Send a single URL from the command line (--url)

### Enviornment

In order to operate, ```hive-writer.py``` must be given two ENV variables. The third ENV variable will use a test version of Hive which may or may not be available:
```
"env": {
    "HIVE_SERVER_ACCOUNT" : "blocktvnews",
    "HIVE_POSTING_KEY": "5KRBCq3D7NiYH2E8AgshtthisisfakeJW4uCJWn8Qrpe9Rei2ZYx",
    "USE_TEST_NODE": "False"
}
```

Hive accounts can be created with the tool [Hive On Board](https://hiveonboard.com?ref=brianoflondon). However, only *Podpings* from accounts which are approved by Podping and PodcastIndex will be recognised. The current authorised list can always be seen [here](https://peakd.com/@podping/following).

### Example

Send a single podping using account stored in ENV variable:

```python hive-writer/hive-writer.py --url http://feed.nashownotes.com/rss.xml ```

Output:
```
2021-05-24 10:42:18,258 INFO root MainThread : Podping startup sequence initiated, please stand by, full bozo checks in operation...
2021-05-24 10:42:19,962 INFO root MainThread : Startup of Podping status: SUCCESS! Hit the BOOST Button.
2021-05-24 10:42:19,962 INFO root MainThread : One URL Received: http://feed.nashownotes.com/rss.xml
2021-05-24 10:42:20,846 INFO root MainThread : Transaction sent: f91a73abd9905135ef4e1ed979cc20f184fbc72e - Num urls: 1 - Json size: 77
```

The transaction can be found on the Hive blockchain using the transaction number: [f91a73abd9905135ef4e1ed979cc20f184fbc72e](https://hiveblocks.com/tx/f91a73abd9905135ef4e1ed979cc20f184fbc72e)


Similarly, to run as a server:

```python hive-writer\hive-writer.py --zmq 9999```

This will initate a startup sequence which tests the ENV supplied credentials for the ability to write to Hive and makes a check on resource credits:

```
2021-05-24 11:29:49,495 INFO root MainThread : Podping startup sequence initiated, please stand by, full bozo checks in operation...
2021-05-24 11:29:51,594 INFO root MainThread : Testing Account Resource Credits - before 99.30%
2021-05-24 11:30:09,730 INFO root MainThread : Transaction sent: 9bfdac9088d75460bc9a560652eda2b86d2f49e9 - Num urls: 0 - Json size: 95
2021-05-24 11:30:09,730 INFO root MainThread : Testing Account Resource Credits.... 5s
2021-05-24 11:30:12,030 INFO root MainThread : Testing Account Resource Credits - after 99.29%
2021-05-24 11:30:12,030 INFO root MainThread : Capacity for further podpings : 28825.1
2021-05-24 11:30:28,903 INFO root MainThread : Transaction sent: e8573e27e02f561ea3e9f037fe6f7823f4445ecb - Num urls: 0 - Json size: 125
2021-05-24 11:30:28,903 INFO root MainThread : Startup of Podping status: SUCCESS! Hit the BOOST Button.
```

The line ```Capacity for further podpings : 28825.1``` gives a very rough indication of how many podpings this account can send in its present state.

This will start up and wait for a ZMQ connection on port 9999. The server waits for a single URL per connection return "OK" or "ERR" if something has failed. Every 3 seconds ```hive-writer.py``` will post the URLs received (up to a maximum of 130) to the Hive blockchain. If URLs come in faster than 130 every 3s they will be held in a buffer and written out.



<br>

## Blockchain Watcher (hive-watcher.py)

The stream of *podpings* can be watched with the ```hive-watcher.py``` code. In addition there is a simplified version of this code ```simple-watcher.py``` which should be used to understand what is going on. There is javascript version in [hive-watcher.js](https://github.com/Podcastindex-org/podping.cloud/blob/main/hive-watcher-js/hive-watcher.js)


```
usage: hive-watcher [options]

PodPing - Watch the Hive Blockchain for notifications of new Podcast Episodes This code will run until terminated reporting every notification of a new Podcast Episode sent to the Hive blockchain by any PodPing servers.

With default arguments it will print to the StdOut a log of each new URL that has updated interspersed with summary lines every 5 minutes that list the number of PodPings and the number of other 'custom_json' operations seen on the blockchain. This interval can be set with the --reports command line.

optional arguments:
  -h, --help          show this help message and exit
  -H, --history-only  Report history only and exit
  -d, --diagnostic    Show diagnostic posts written to the blockchain
  -r , --reports      Time in MINUTES between periodic status reports, use 0 for no periodic reports
  -s , --socket       <IP-Address>:<port> Socket to send each new url to
  -t, --test          Use a test net API
  -q, --quiet         Minimal output
  -v, --verbose       Lots of output

  -b , --block        Hive Block number to start replay at or use:
  -o , --old          Time in HOURS to look back up the chain for old pings (default is 0)
  -y , --startdate    <%Y-%m-%d %H:%M:%S> Date/Time to start the history
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

## Blockchain Watcher (hive-watcher.py)

The watcher script is how you see which podcast feed urls have signaled an update.

The python script `hive-watcher.py` is more full featured - allowing for socket listening, and other options.

<br>

### Simple Watcher (simple-watcher.py)

This is the easiest way to get started watching the blockchain for feed updates.  Simply do the following:

1. Clone this repo.
2. Switch to the `hive-watcher` sub-directory.
3. Make sure python3 and pip3 are installed.
4. Run `pip3 install beem`.
5. Launch the watcher script like this: `python3 ./simple-watcher.py`

Each time a feed update notification is detected on the blockchain, the full url of the feed is printed to STDOUT on a new line.  Each
FQDN that is output represents a new episode that has been published, or some other significant update to that podcast feed.

You can watch this output as a way to signal your system to re-parse a podcast feed.  Or you can use it as a starting template to
develop a more customized script for your environment.  It's dead simple!

<br>

## Running a Node

First clone this repo.

Make sure that libzmq-dev is installed:

`apt-get install libzmq3-dev`

Build and launch podping like so:

`cd podping && crate run`

Then launch hive-write like this:

`python3 ./hive-writer/hive-writer.py`

<br>

## Important Note

This software is in beta condition.  It is running in production.  But, you should still expect bugs.

The hive-writer agent requires a Hive account and Hive Posting key which are not included in this repo.  They should be exported into your
shell environment before launching the script.

### Hive authoization
If you want to write your own podpings directly to the Hive blockchain, in order for other watching clients to notice your podpings, your writing Hive account needs to be authorised. This authorisation is handled by @brianoflondon [contact via Podcastindex.social](https://podcastindex.social/@brianoflondon).

<br>

## The Podping Network Idea

![Framework Overview 1](framework1.png)
