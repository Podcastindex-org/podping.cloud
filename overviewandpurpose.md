# Overview and purpose of podping.cloud

Podping is an alternative to WebSub for the open podcasting ecosystem.  WebSub is a wonderful technology.  But, there are three main issues that 
make it less than ideal for the open podcasting ecosystem:

1. Not all hosting companies have chosen to support it.
2. The burden of resubscribing on a per-feed basis every 7-15 days becomes a burden as the feed count grows into 6 digits and up.
3. The WebSub ecosystem of hubs consists mainly of SuperFeedr and Google.  Most feeds are concentrated on those two.

A better solution than subscribing and re-subscribing constantly to individual podcast feeds is for aggregators, directories and apps to 
just subscribe to a single firehose of all podcast feed urls that publish a new episode.  Podcast publishers notify podping.cloud that a url they manage
has updated.  The software validates that publisher's identify and then writes that feed url to the [Hive](https://hive.io/) blockchain.

The Hive blockchain writes a new block every 3 seconds, and accepts custom data written into it.  We simply write the url of the updated feed as a 
custom JSON object into the blockchain as we recieve them.  This allows anyone to just "watch" the Hive blockchain and see immediately (within 5-10) seconds that a 
feed url has updated and be confident that it's the Hosting company (or self-hosted podcaster) themselves that sent the notification.

Multiple podping servers in different global regions will be run to ensure that there is no single point of failure.  The "podping.cloud" host name will be 
round-robin addressed to these servers.  Any of the servers can handle receiving and writing urls to the blockchain.

The bottom line is that podcasters and hosting companies just send a GET request with the url to a single web address, and everyone else in the industry can see that updated within a few seconds with no subscribing (or re-subscribing) required by anyone.
