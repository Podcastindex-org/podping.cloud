# Hive
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

There is an internal cost to creating accounts on Hive and new accounts are constrained in what they can do by the "Resource Credit" system. This limits the number of posts, comments and likes a user can perform. To get more resource credits, the user must either buy and "power up" Hive or have this Hive Power delegated to them. Users with more Hive Power have a greater number of resource credits and their resource credits recharge faster.

The process of staking Hive is called powering up. Powered up Hive (Hive Power) can be powered down with a 13 week process in which 1/13 of the Hive is returned every week following a power down request.

Hive Power gives that account increased Resource Credits, it also gives the account added weight when voting on content: that weight turns directly into financial payouts for the creators of content voted on and for the account holder (so called "curator rewards"). Witness positions are also determined by the weight of votes from users.
### Hive and Podping

For the alpha and beta phases of Podping on Hive, the account being used by PodcastIndex to send the Podpings onto the Hive chain is called [@hivehydra](https://hive.ausbit.dev/@hivehydra). This account has around 1500 Hive Power (worth $600 at time of writing). This is sufficient. There is no cost for usage, however this stake is set aside for this purpose.
