# Overview and purpose of podping.cloud

Podping is an alternative to WebSub for the open, RSS based podcasting ecosystem that allows for rapid, global notification of podcast feed updates.

<br>

## Problem

WebSub is a wonderful technology for the blogging world, and for small subscription bases.  But, there are three main issues that make it less than ideal for the open world of podcasting:

1. Not all hosting companies have chosen to support it.
2. The burden of resubscribing on a per-feed basis every 7-15 days goes up exponentially as the feed count grows into 6 or 7 digits.
3. The WebSub ecosystem of hubs consists mainly of SuperFeedr and Google.  Most feeds are concentrated on those two.

A final issue is that of reliability.  In our experience, hubs have proven to be unreliable at times.  Especially the free ones.  We hae seen what appear to be outage periods, or just silence.

<br>

## Solution

A better solution than subscribing and re-subscribing constantly to individual podcast feeds is for aggregators, directories and apps to
just subscribe to a single firehose of all podcast feed urls that publish a new episode.  Podcast publishers notify podping.cloud that a url they manage has updated.  The podping server validates that publisher's identity and then writes the feed url to the [Hive](https://hive.io/) blockchain.

Hive is an open blockchain that writes a new block every 3 seconds, and accepts custom data written into it.  We simply write the urls of the updated feeds as a custom JSON object into the blockchain as we receive them.  This allows anyone to just "watch" the Hive blockchain and see within about 20 seconds that a podcast feed url has updated and be confident that it's the Hosting company (or self-hosted podcaster) themselves that sent that notification.

Multiple podping servers in different global regions will be run to ensure that there is no single point of failure.  The "podping.cloud" host name will be load balanced to these servers.  Any of the servers can handle receiving and writing urls to the blockchain.

The bottom line is that podcasters and hosting companies just send a GET request with the url to a single web address, and everyone else in the industry can see that update within a few seconds with no subscribing (or re-subscribing) required by anyone.

<br>

## Rollout

Initially, podping will be run by Podcastindex.org to facilitate its beta testing and development.  Podcast Index will run multiple servers to distribute load and provide redundancy.  But, we are building this software as an open source project so that it can eventually be run quickly and easily by any hosting provider, or even baked into other podcasting CMS projects.

We encourage anyone and everyone to contribute to it's development.

## Cost and sustainability

You may be wondering how there seems to be no mention of the cost of using the Hive blockchain or how Podping is suddenly able to send 10's of 1,000's of transactions to a blockchain without mentioning some way of paying for this.

The only way to explain this properly is to understand a little about the background and workings of the Hive blockchain.

### About the Hive blockchain project

Hive is an open source blockchain which was first developed to build a censorship resistant social media platform based on the shared infrastructure of a blockchain. To compare with a centralized website like Facebook or Twitter, the back end systems and databases for content and authentication are replaced with a continually growing blockchain. Updates to the chain (i.e. posts or comments or likes in the social media example) are verified by an array of "Witness" servers, each run by individuals or groups committed to the project. Multiple front ends exist for blogging and other uses:

- [Hive.blog](https://hive.blog/) - the core open source functionality of Hive is know as "Conderser" and Hive.blog runs this.

- [PeakD](https://peakd.com/) - this is another very full featured front end for the blogging and social capabilities of Hive.

- [Ecency](https://ecency.com/) - mobile focused front end (with great search capabilities).

- [Leofinance](https://leofinance.io/) - Crypto and finance focused site with its own reward token and DeFi spin off.

- [Splinterlands](https://splinterlands.com/) - One of the most popular blockchain based games with tradable cards and a full economy.

And many more.... what these all have in common is using the authentication systems and back end storage of Hive. The back end of Hive consists of Witnesses and API servers
### Witnesses

Witnesses are rewarded with the Hive cryptocurrency for processing each block and verifying that all the information in it was correctly submitted and signed by a valid Hive user account. Unlike "proof of work" blockchains such as Bitcoin or Ethereum, this verification process is as mathematically efficient as it can be, whilst still being cryptographically secure. This allows blocks to be produced and verified regularly every 3s by many witness computers without huge amounts of processing power, or other computing resources.

[You can watch these witnesses rotate to produce each block at this site](https://hive.arcange.eu/schedule/). The Top 20 witnesses are special with a 21'st witness being chosen in each rotation of 21 blocks from amongst all the others by a seniority ranking algorithm.

Nevertheless the combined hardware and resources which run Hive is considerable.

### API Servers

A blockchain, whilst good at being an immutable database, isn't very good at serving up information quickly and is hard to search. Hive solves this problem by maintaining, alongside the main blockchain data, a live database of important recent data. Everything can, ultimately, be rebuilt from the blockchain record, but this database, called Hivemind, is vital to allow apps to "feel" like centralized Web 2.0 sites, whilst still being decentralized.

This data is served up to web apps by API servers. Not all witnesses are running these but [there are sufficient](https://beacon.peakd.com/). If Hive is lacking in anything, it would be to add more API servers.

### User resources

There is an internal cost to creating accounts on Hive
### Hive and Podping

For the alpha and beta phases of Podping on Hive,
