#!/usr/bin/env python3
# Modeled after the simple-watcher.py V1.0
# modified to write to csv file 'Data.csv'
#
# Simple version of Hive Podping watcher - no options, just runs
# The only external library needed is "beem" - pip install beem
# Beem is the official Hive accessing library for Python.
#
# Version 1.0

from typing import Set
import json
import os
import logging
import sys
import time
import datetime
import csv

import beem
from beem.account import Account
from beem.blockchain import Blockchain

WATCHED_OPERATION_IDS = ["podping", "hive-hydra"]

def get_allowed_accounts(acc_name="podping") -> Set[str]:
    """get a list of all accounts allowed to post by acc_name (podping)
    and only react to these accounts"""

    # This is giving an error if I don't specify api server exactly.
    # TODO reported as Issue on Beem library https://github.com/holgern/beem/issues/301
    h = beem.Hive(node="https://api.hive.blog")
    master_account = Account(acc_name, blockchain_instance=h, lazy=True)
    return set(master_account.get_following())

def allowed_op_id(operation_id) -> bool:
    """Checks if the operation_id is in the allowed list"""
    if operation_id in WATCHED_OPERATION_IDS:
        return True
    else:
        return False

def configure_logging():
    try: # logging errors should never throw errors so:
        log_dir = os.path.join(os.path.dirname(os.path.abspath(__file__)),"logs")
        log_name = os.path.splitext(os.path.basename(os.path.abspath(__file__)))[0] + ".log"
        if not os.path.exists(log_dir):
            os.mkdir(log_dir)
        logging.basicConfig(filename=os.path.join(log_dir,"errors-"+log_name), encoding='utf-8', level=logging.ERROR)
    except:
        try:
            logging.basicConfig(filename="errors-"+log_name, encoding='utf-8', level=logging.ERROR)
        except:
            logging.basicConfig(filename="errors.log", encoding='utf-8', level=logging.ERROR)

def write_post_to_csv(post,filepath_data):
    fieldnames = [
        'timestamp_seen','timestamp_post',
        'id','type','trx_id','trx_num','block_num',
        'required_auths','required_posting_auths','json'
    ]
    # build a new custom dictionary from the post
    post_row={
        'timestamp_seen':repr(time.time()),
        'timestamp_post':repr(post.get("timestamp").timestamp()),
        'id':str(post.get("id")),
        'type':str(post.get("type")),
        'trx_id':str(post.get("trx_id")),
        'trx_num':repr(post.get("trx_num")),
        'block_num':repr(post.get("block_num")),
        'required_auths':repr(post.get("required_auths")),
        'required_posting_auths':repr(post.get("required_posting_auths")),
        'json':repr(json.dumps(json.loads(post.get("json")), indent=4))
    }
    # csv writer, see: https://docs.python.org/3/library/csv.html
    if os.path.isfile(filepath_data) :
        with open(filepath_data,'a', newline='') as csvfile:
            writer = csv.DictWriter(csvfile, fieldnames=fieldnames)
            writer.writerow(post_row)
            csvfile.close
    else :
        with open(filepath_data,'w', newline='') as csvfile:
            writer = csv.DictWriter(csvfile, fieldnames=fieldnames)
            writer.writeheader()
            writer.writerow(post_row)
            csvfile.close

def main():
    # setup file output
    configure_logging()
    """ Outputs URLs one by one as they appear on the Hive Podping stream """
    allowed_accounts = get_allowed_accounts()
    hive = beem.Hive()
    blockchain = Blockchain(mode="head", blockchain_instance=hive)
    # If you want instant confirmation, you need to instantiate
    # class:beem.blockchain.Blockchain with mode="head",
    # otherwise, the call will wait until confirmed in an irreversible block.
    # noinspection PyTypeChecker
    # Filter only for "custom_json" operations on Hive.
    stream = blockchain.stream(
        opNames=["custom_json"], raw_ops=False, threading=False, thread_num=4
    )
    for post in stream:
        try:
            # Filter only on post ID from the list above.
            if allowed_op_id(post["id"]):
                # Filter by the accounts we have authorised to podping
                if set(post["required_posting_auths"]) & allowed_accounts:
                    write_post_to_csv(post,'data.csv')
                else : # Write out the same data to the data-unauthorized.csv file...
                    write_post_to_csv(post,'data-unauthorized.csv')
            else : # the firehose - all other hive 'custom json' posts to data-not-podping_firehose.csv
                write_post_to_csv(post,'data-not-podping_firehose.csv')
        except: # catch *all* errors
            logging.error(sys.exc_info()[0])

if __name__ == "__main__":
    # Runs until terminated with Ctrl-C
    main()
