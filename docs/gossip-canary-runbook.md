# Gossip dual-write canary runbook

Target: ONE of the 5 production podping.cloud servers.

## Prepare
1. `docker pull podcastindexorg/podping-gossipwriter:0.7.0` and the 3.0.0
   front-end image — confirm both pull before touching anything.
2. `sudo mkdir -p /data/gossip && sudo chown 1000:1000 /data/gossip`
3. Do NOT touch `/opt/podping.cloud/hivewriter.env`.
4. Back up the current compose file:
   `cp docker-compose.yaml docker-compose.yaml.pre-gossip`

## Deploy
5. Replace the compose file with the gossip-dual-write version
   (front-end image 3.0.0, `GOSSIP_WRITER_ENABLED=true`, gossip-writer service).
6. `docker compose pull && docker compose up -d`
7. `docker compose logs -f podping-cloud | head -50` — expect
   "Gossip PAIR socket: [...] connected." alongside the normal startup.
8. `docker compose logs gossip-writer | head -50` — note the printed
   signing pubkey; you'll trust it in step 10.

## Verify (all 5 must pass)
9.  HIVE UNHARMED: existing Hive monitoring shows podpings from this
    server continuing at the normal rate. THE MUST-NOT-BREAK INVARIANT.
10. GOSSIP LIVE: on a separate machine, run gossip-listener and
    gossip-monitor (from podping.alpha), add the canary's pubkey (step 8)
    to trusted_publishers.txt, and observe verified notifications arriving.
11. ARCHIVE FILLING: on the server:
    `sqlite3 /data/gossip/archive.db 'select count(*) from messages;'`
    twice, a few minutes apart — count grows with traffic.
12. FAILURE DRILL (during real traffic):
    `docker compose stop gossip-writer` → front-end keeps serving HTTP,
    Hive podpings keep flowing (front-end logs show only
    "Gossip write failed" lines) →
    `docker compose start gossip-writer` → listener sees notifications
    resume within ~2 minutes.
13. SOAK 48h: watch `docker stats gossip-writer` for memory growth
    (iroh-gossip churn leak) and syslog for panic/abort from iroh-quinn.

## Rollback (any time)
- Soft: set `GOSSIP_WRITER_ENABLED: "false"` in compose,
  `docker compose up -d podping-cloud`.
- Full: `cp docker-compose.yaml.pre-gossip docker-compose.yaml &&
  docker compose up -d --remove-orphans`.
- The Hive path is untouched by design; rollback risk is near zero.

## Done
All 5 checks pass → merge the gossip-dual-write PR. Rollout to the other
4 servers is a separate follow-on effort.
