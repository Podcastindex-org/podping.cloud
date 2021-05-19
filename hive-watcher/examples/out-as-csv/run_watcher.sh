#!/usr/bin/env bash
# just a linux script runner to kickoff writing the hive-watcher script.
./create_data.csv.py
./hive-watcher.py >> data.csv
