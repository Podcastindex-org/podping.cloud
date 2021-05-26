import argparse
import json
import logging
import os
from datetime import date, datetime, timedelta
from ipaddress import IPv4Address, IPv6Address, AddressValueError
from socket import AF_INET, SOCK_STREAM, socket
from time import strptime
from typing import Set, Optional, Union

import beem
from beem import block
from beem.account import Account
from beem.block import Block
from beem.blockchain import Blockchain

from config import Config



WATCHED_OPERATION_IDS = ["podping", "hive-hydra"]
DIAGNOSTIC_OPERATION_IDS = ["podping-startup"]



class Pings:
    total_pings = 0



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


def output(post) -> int:
    """Prints out the post and extracts the custom_json"""

    data = json.loads(post.get("json"))
    if Config.diagnostic:
        logging.info(
            f"Diagnostic - {post.get('timestamp')} "
            f"- {data.get('server_account')} - {post.get('trx_id')} - {data.get('message')}"
            )
        logging.info(
            json.dumps(data, indent=2)
        )

    if Config.quiet:
        if data.get("num_urls"):
            return data.get("num_urls")
        else:
            return 1

    if Config.urls_only:
        if data.get("url"):
            print(data.get("url"))
            return 1
        elif data.get("urls"):
            for url in data.get("urls"):
                print(url)
            return data.get("num_urls")

    data["required_posting_auths"] = post.get("required_posting_auths")
    data["trx_id"] = post.get("trx_id")
    data["timestamp"] = post.get("timestamp")

    count = 0
    if Config.use_test_node:
        data["test_node"] = True

    if data.get("url"):
        logging.info(
            f"Feed Updated - {data.get('timestamp')} - {data.get('trx_id')} "
            f"- {data.get('url')}"
        )
        count = 1
    elif data.get("urls"):
        for url in data.get("urls"):
            count += 1
            logging.info(
                f"Feed Updated - {data.get('timestamp')} - {data.get('trx_id')} - {url}"
            )
    return count


def output_status(
    timestamp: str,
    pings,
    count_posts,
    time_to_now="",
    current_block_num="",
) -> None:
    """Writes out a status update at with some count data"""
    if not Config.reports and Config.quiet:
        return None
    if time_to_now:
        logging.info(
            f"{timestamp} - Podpings: {pings:7} / {Pings.total_pings:10} - Count:"
            f" {count_posts} - BlockNum: {current_block_num} - Time Delta:"
            f" {time_to_now}"
        )
    else:
        logging.info(
            f"{timestamp} - Podpings: {pings:7} / {Pings.total_pings:1} - Count:"
            f" {count_posts} - BlockNum: {current_block_num}"
        )


def output_to_socket(
    post,
    client_socket: Optional[socket] = None,
) -> None:
    """Take in a post and a socket and send the url to a socket"""
    if not client_socket:
        return
    data = json.loads(post.get("json"))
    url = data.get("url")
    if url:
        try:
            client_socket.send(url.encode())
        except Exception as ex:
            error_message = f"{ex} occurred {ex.__class__}"
            logging.error(error_message)

    # Do we need to receive from the socket?

def get_stream(blockchain, block_num=None):
    """ Open up a stream from Hive either live or history """

    # If you want instant confirmation, you need to instantiate
    # class:beem.blockchain.Blockchain with mode="head",
    # otherwise, the call will wait until confirmed in an irreversible block.
    # noinspection PyTypeChecker

    if block_num:
        # History
        stream = blockchain.stream(
            opNames=["custom_json"],
            start=block_num,
            max_batch_size=50,
            raw_ops=False,
            threading=False,
        )
    else:
        # Live
        stream = blockchain.stream(
            opNames=["custom_json"],
            raw_ops=False,
            threading=False
        )
    return stream




def scan_live(
    hive: beem.Hive,
    report_freq: int = 5,
    reports=True,
    use_test_node=False,
    client_socket: Optional[socket] = None,
    quiet=False,
    diagnostic=False,
    urls_only=False
):
    """watches the stream from the Hive blockchain"""
    report_timedelta = timedelta(minutes=report_freq)

    allowed_accounts = get_allowed_accounts()

    blockchain = Blockchain(mode="head", blockchain_instance=hive)
    current_block_num = blockchain.get_current_block_num()
    if reports:
        logging.info(f"Watching live from block_num: {current_block_num}")

    stream = get_stream(blockchain)

    start_time = datetime.utcnow()
    count_posts = 0
    pings = 0

    for post in stream:
        count_posts += 1
        time_dif = post["timestamp"].replace(tzinfo=None) - start_time
        if reports:
            if time_dif > report_timedelta:
                current_block_num = str(blockchain.get_current_block_num())
                timestamp = post["timestamp"]
                output_status(
                    timestamp,
                    pings,
                    count_posts,
                    current_block_num=current_block_num,
                    reports=reports,
                    quiet=quiet,
                )
                start_time = post["timestamp"].replace(tzinfo=None)
                count_posts = 0
                pings = 0

        if allowed_op_id(post["id"]):
            if set(post["required_posting_auths"]) & allowed_accounts:
                count = output(post, quiet, use_test_node, urls_only=urls_only)
                if client_socket:
                    output_to_socket(post, client_socket)
                pings += count
                Pings.total_pings += count

        if diagnostic:
            if post["id"] in DIAGNOSTIC_OPERATION_IDS:
                output(post,quiet,use_test_node,diagnostic)

        if time_dif > timedelta(hours=1):
            # Re-fetch the allowed_accounts every hour in case we add one.
            allowed_accounts = get_allowed_accounts()


def scan_history():
    """Scans back in history timed time delta ago, reporting with report_freq
    if timed is an int, treat it as hours, if report_freq is int, treat as min"""

    # Very first transaction from Dave Testing:
    """2021-05-10 13:51:58,353 INFO root MainThread
     : Feed Updated - 2021-05-07 20:58:33+00:00
     - f0affd194524a6e0171d65d29d5c501865f0bd72
     - https://feeds.transistor.fm/retail-remix"""

    scan_start_time = datetime.utcnow()
    report_timedelta = timedelta(minutes=Config.report_minutes)

    allowed_accounts = get_allowed_accounts()

    count_posts = 0
    pings = 0

    if Config.reports:
        logging.info("Started catching up")

    stream = get_stream(Config.blockchain, Config.block_num)

    report_period_start_time = Config.start_time
    post = None
    for post in stream:
        post_time = post["timestamp"].replace(tzinfo=None)
        time_dif = post_time - report_period_start_time
        time_to_now = datetime.utcnow() - post_time
        count_posts += 1
        if Config.reports:
            if time_dif > report_timedelta:
                timestamp = post["timestamp"]
                current_block_num = post["block_num"]
                output_status(
                    timestamp,
                    pings,
                    count_posts,
                    time_to_now,
                    current_block_num
                )
                report_period_start_time = post["timestamp"].replace(tzinfo=None)
                count_posts = 0
                pings = 0

        if allowed_op_id(post["id"]):
            if set(post["required_posting_auths"]) & allowed_accounts:
                count = output(post)
                pings += count
                Pings.total_pings += count

        if Config.diagnostic:
            if post["id"] in DIAGNOSTIC_OPERATION_IDS:
                output(post)

        if time_to_now < timedelta(seconds=2) or post_time > Config.stop_at:
            timestamp = post["timestamp"]
            current_block_num = post["block_num"]
            if Config.show_reports:
                output_status(
                    timestamp,
                    pings,
                    count_posts,
                    time_to_now,
                    current_block_num
                    )

            if not (Config.urls_only):
                logging.info(f"block_num: {post['block_num']}")
            # Break out of the for loop we've caught up.
            break

    if post and (not (Config.urls_only)):
        scan_time = datetime.utcnow() - scan_start_time
        logging.info(
            f"Finished catching up at block_num: {post['block_num']} in {scan_time}"
        )


def open_socket(
    client_socket: socket, ip_address: Union[IPv4Address, IPv6Address], port: int
):
    """If a socket errors out and will try to reopen it"""
    try:
        client_socket.connect((ip_address.compressed, port))
    except Exception as ex:
        error_message = f"{ex} occurred {ex.__class__}"
        logging.error(error_message)





def main() -> None:
    logging.basicConfig(
        level=logging.INFO,
        format=f"%(asctime)s - %(levelname)s %(name)s %(threadName)s : -  %(message)s",
        datefmt="%Y-%m-%dT%H:%M:%S%z",
    )
    Config.setup()

    # Send every URL to a simple socket
    client_socket = None
    if Config.use_socket:
        # TODO: Socket needs testing or conversion to zmq
        ip_port = Config.use_socket.split(":")
        try:
            ip_address = IPv4Address(ip_port[0])
        except AddressValueError:
            ip_address = IPv6Address(ip_port[0])
        port = int(ip_port[1])
        client_socket = socket(AF_INET, SOCK_STREAM)
        open_socket(client_socket, ip_address, port)

    """ do we want periodic reports? """
    if Config.show_reports:
        if Config.use_test_node:
            logging.info("---------------> Using Test Node " + TEST_NODE[0])
        else:
            logging.info("---------------> Using Main Hive Chain ")

    # scan_history will look back over the last 1 hour reporting every 15 minute chunk
    if Config.old or Config.block or Config.startdate:

        scan_history()

    #     if Config.block_num:
    #         scan_history(
    #             hive,
    #             block_num=block_num,
    #             report_freq=report_minutes,
    #             reports=reports,
    #             quiet=quiet,
    #             diagnostic=diagnostic,
    #             urls_only=urls_only,
    #             stop_after=stop_after,
    #         )
    #     else:
    #         if my_args["startdate"]:
    #             arg_time = my_args["startdate"]
    #             start_date = datetime.strptime(arg_time, "%Y-%m-%d %H:%M:%S")
    #             hours_ago = datetime.now() - start_date
    #         else:
    #             hours_ago = timedelta(hours=Config.old)

    #         scan_history(
    #             hive,
    #             hours_ago=hours_ago,
    #             report_freq=report_minutes,
    #             reports=reports,
    #             quiet=quiet,
    #             diagnostic=diagnostic,
    #             urls_only=urls_only,
    #             stop_after=stop_after,
    #         )

    # history_only = my_args["history_only"]

    if not Config.history_only:
        # scan_live will resume live scanning the chain and report every 5 minutes or
        # when a notification arrives
        scan_live(
            hive,
            my_args["reports"],
            reports,
            quiet=quiet,
            client_socket=client_socket,
            urls_only=urls_only,
        )
    else:
        logging.info("history_only is set. exiting")


if __name__ == "__main__":
    main()
