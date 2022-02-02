docker run \
  --env MEESEEKER_PUBLISH_OP_CUSTOM_ID=true \
  --env MEESEEKER_EXPIRE_KEYS=300 \
  --name customjsons \
  --env MEESEEKER_STREAM_MODE=head \
  -d -p 6380:6379 inertia/meeseeker:latest meeseeker sync hive:op:custom_json:podping