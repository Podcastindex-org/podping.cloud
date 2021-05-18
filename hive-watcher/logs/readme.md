# Temporary API Disconnects

Sometimes, while watching the Hive blockchain live, the watcher may experience issues with an API (RPC) server. When that happens, the ```hive-watcher.py``` output may look like this:

```
2021-05-18T16:06:44+0300 - INFO root MainThread : -  2021-05-18 12:40:45+00:00 - Podpings:      10 / 733 - Count: 676 - BlockNum: 53997996
2021-05-18T16:06:48+0300 - INFO root MainThread : -  2021-05-18 12:41:48+00:00 - Podpings:       4 / 737 - Count: 692 - BlockNum: 53997997
2021-05-18T16:06:49+0300 - WARNING beemapi.node MainThread : -  Error: Service Temporarily Unavailable
2021-05-18T16:06:49+0300 - WARNING beemapi.node MainThread : -  Retry RPC Call on node: https://api.openhive.network (1/5)

2021-05-18T16:06:49+0300 - WARNING beemapi.node MainThread : -  Retrying in 0 seconds

2021-05-18T16:06:50+0300 - WARNING beemapi.node MainThread : -  Error: Service Temporarily Unavailable
2021-05-18T16:06:50+0300 - WARNING beemapi.node MainThread : -  Retry RPC Call on node: https://api.openhive.network (1/5)

2021-05-18T16:06:50+0300 - WARNING beemapi.node MainThread : -  Retrying in 0 seconds

2021-05-18T16:06:51+0300 - WARNING beemapi.node MainThread : -  Error: Service Temporarily Unavailable
2021-05-18T16:06:51+0300 - WARNING beemapi.node MainThread : -  Retry RPC Call on node: https://api.openhive.network (1/5)

2021-05-18T16:06:51+0300 - WARNING beemapi.node MainThread : -  Retrying in 0 seconds

2021-05-18T16:06:53+0300 - WARNING beemapi.node MainThread : -  Error: Service Temporarily Unavailable
2021-05-18T16:06:53+0300 - WARNING beemapi.node MainThread : -  Retry RPC Call on node: https://api.openhive.network (1/5)

2021-05-18T16:06:53+0300 - WARNING beemapi.node MainThread : -  Retrying in 0 seconds

2021-05-18T16:06:54+0300 - INFO root MainThread : -  2021-05-18 12:42:51+00:00 - Podpings:       4 / 741 - Count: 624 - BlockNum: 53998000
2021-05-18T16:06:55+0300 - WARNING beemapi.node MainThread : -  Error: Service Temporarily Unavailable
```

This is nothing to worry about. In this folder are three files:

- [Original log with disconnects](https://github.com/Podcastindex-org/podping.cloud/blob/simple-watcher/hive-watcher/logs/oringal-with-disconnects.log)
- [Original log filtered showing only 1 minute reports](https://github.com/Podcastindex-org/podping.cloud/blob/simple-watcher/hive-watcher/logs/original-filtered-without-disconnects.log)

- [Output run later with a replay of the chain](https://github.com/Podcastindex-org/podping.cloud/blob/simple-watcher/hive-watcher/logs/verification-history.log)

It is easy to verify that the reported output live, despite temporary network interference and drop outs, matches the report from a replay of the blockchain later. No pings were missed in the live example. Whilst the logs aren't identical because aligning the minute by minute boundary is had to do on replay, however the totals are within a few pings arrising from a slightly different replay time window.

## Live filterd:
```
2021-05-18 13:14:21+00:00 - Podpings:       5 / 989 - Count: 574 - BlockNum: 53998149
2021-05-18 13:15:24+00:00 - Podpings:      12 / 1001 - Count: 599 - BlockNum: 53998170
2021-05-18 13:16:27+00:00 - Podpings:       7 / 1008 - Count: 612 - BlockNum: 53998191
2021-05-18 13:17:30+00:00 - Podpings:      10 / 1018 - Count: 610 - BlockNum: 53998212
2021-05-18 13:18:33+00:00 - Podpings:      10 / 1028 - Count: 659 - BlockNum: 53998233

1 Hour total: 461
```

## Replay:
```
2021-05-18 13:14:15+00:00 - Podpings:       7 /        426 - Count: 583 - BlockNum: 53998147 - Time Delta: 0:03:29.226417
2021-05-18 13:15:18+00:00 - Podpings:      11 /        437 - Count: 569 - BlockNum: 53998168 - Time Delta: 0:02:26.651736
2021-05-18 13:16:21+00:00 - Podpings:       9 /        446 - Count: 599 - BlockNum: 53998189 - Time Delta: 0:01:23.672277
2021-05-18 13:17:24+00:00 - Podpings:       9 /        455 - Count: 637 - BlockNum: 53998210 - Time Delta: 0:00:20.696488
2021-05-18 13:17:51+00:00 - Podpings:       2 /        457 - Count: 272 - BlockNum: 53998219 - Time Delta: 0:00:01.657393
block_num: 53998219
Finished catching up at block_num: 53998219 in 0:00:18.011789

1 Hour total: 457
```