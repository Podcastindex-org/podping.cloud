# Overview and purpose of podping.cloud

Podping is an alternative to WebSub for the open, RSS based podcasting ecosystem that allows for rapid, global notification of podcast feed updates.

<br>

## Problem

WebSub is a wonderful technology for the blogging world, and for small subscription bases. But, there are three main issues that make it less than ideal for the open world of podcasting:

1. Not all hosting companies have chosen to support it.
2. The burden of resubscribing on a per-feed basis every 7-15 days goes up exponentially as the feed count grows into 6 or 7 digits.
3. The WebSub ecosystem of hubs consists mainly of SuperFeedr and Google. Most feeds are concentrated on those two.

A final issue is that of reliability. In our experience, hubs have proven to be unreliable at times. Especially the free ones. We have seen what appear to be outage periods, or just silence.

<br>

## Solution

A better solution than subscribing and re-subscribing constantly to individual podcast feeds is for aggregators, directories and apps to just subscribe to a single firehose of all podcast feed urls that publish a new episode. Podcast publishers notify podping.cloud that a url they manage has updated. The podping server validates that publisher's identity and then writes the feed url to the [Hive](https://hive.io/) blockchain.

Hive is an open blockchain that writes a new block every 3 seconds, and accepts custom data written into it. We simply write the urls of the updated feeds as a custom JSON object into the blockchain as we receive them. This allows anyone to just "watch" the Hive blockchain and see within about 20 seconds that a podcast feed url has updated and be confident that it's the Hosting company (or self-hosted podcaster) themselves that sent that notification.

Multiple podping servers in different global regions will be run to ensure that there is no single point of failure. The "podping.cloud" host name will be load balanced to these servers. Any of the servers can handle receiving and writing urls to the blockchain.

The bottom line is that podcasters and hosting companies just send a GET request with the url to a single web address, and everyone else in the industry can see that update within a few seconds with no subscribing (or re-subscribing) required by anyone.

<br>

## Rollout

Initially, podping will be run by Podcastindex.org to facilitate its beta testing and development. Podcast Index will run multiple servers to distribute load and provide redundancy. But, we are building this software as an open source project so that it can eventually be run quickly and easily by any hosting provider, or even baked into other podcasting CMS projects.

We encourage anyone and everyone to contribute to its development.

<br>

[If you are not familiar with Hive, you may wish to see the explanation of Hive linked here.](https://github.com/Podcastindex-org/podping.cloud/blob/update-overview/explanaing_hive.md)
