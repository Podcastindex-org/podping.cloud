import json
import logging
from datetime import datetime, time, timedelta
from typing import Set

import beem
from beem.account import Account

from config import Config


class Pings:
    total_pings = 0

class UnspecifiedHiveException(Exception):
    pass


def get_allowed_accounts(acc_name: str = "podping") -> Set[str]:
    """get a list of all accounts allowed to post by acc_name (podping)
    and only react to these accounts"""

    # This is giving an error if I don't specify api server exactly.
    # TODO reported as Issue on Beem library https://github.com/holgern/beem/issues/301
    h = beem.Hive(node="https://api.hive.blog")

    master_account = Account(acc_name, blockchain_instance=h, lazy=True)

    return set(master_account.get_following())


def allowed_op_id(operation_id: str) -> bool:
    """Checks if the operation_id is in the allowed list"""
    if operation_id in Config.WATCHED_OPERATION_IDS:
        return True
    else:
        return False


def output(post) -> int:
    """Prints out the post and extracts the custom_json"""

    data = json.loads(post.get("json"))

    if Config.quiet:
        if data.get("num_urls"):
            return data.get("num_urls")
        else:
            return 1

    if Config.urls_only:
        if data.get("url"):
            print(data.get("url"))
            # These calls do nothing if sockets are not open
            # ZMQ Socket will block until it receives acknowledgement
            Config.socket_send(data.get("url"))
            Config.zsocket_send(data.get("url"))
            return 1
        elif data.get("urls"):
            for url in data.get("urls"):
                print(url)
                Config.socket_send(url)
                Config.zsocket_send(url)
            return data.get("num_urls")

    if Config.use_socket:
        if data.get("url"):
            Config.socket_send(data.get("url"))
        elif data.get("urls"):
            for url in data.get("urls"):
                Config.socket_send(url)

    if Config.use_zmq:
        if data.get("url"):
            Config.zsocket_send(data.get("url"))
        elif data.get("urls"):
            for url in data.get("urls"):
                Config.zsocket_send(url)

    data["required_posting_auths"] = post.get("required_posting_auths")
    data["trx_id"] = post.get("trx_id")
    data["timestamp"] = post.get("timestamp")

    count = 0
    if Config.use_test_node:
        data["test_node"] = True

    if data.get("url"):
        logging.info(
            f"Feed Updated - {data.get('timestamp')} - {data.get('trx_id')} "
            f"- {data.get('url')} - {data['required_posting_auths']}"
        )
        count = 1
    elif data.get("urls"):
        for url in data.get("urls"):
            count += 1
            logging.info(
                f"Feed Updated - {data.get('timestamp')} - {data.get('trx_id')}"
                f" - {url} - {data['required_posting_auths']}"
            )
    return count


def output_diagnostic(post: dict) -> None:
    """Just output Diagnostic messages recorded on the chain"""
    data = json.loads(post.get("json"))
    if Config.diagnostic:
        logging.info(
            f"Diagnostic - {post.get('timestamp')} "
            f"- {data.get('server_account')} - {post.get('trx_id')} - {data.get('message')}"
        )
        logging.info(json.dumps(data, indent=2))


def output_status(
    timestamp: str,
    pings: int,
    count_posts: int,
    time_to_now: timedelta = None,
    current_block_num: int ="",
) -> None:
    """Writes out a status update at with some count data"""
    if not Config.reports and Config.quiet:
        return None
    if time_to_now:
        logging.info(
            f"{timestamp} - Podpings: {pings:7} / {Pings.total_pings:10} - Count:"
            f" {count_posts:12} - BlockNum: {current_block_num} - Time Delta:"
            f" {time_to_now}"
        )
    else:
        logging.info(
            f"{timestamp} - Podpings: {pings:7} / {Pings.total_pings:10} - Count:"
            f" {count_posts:12} - BlockNum: {current_block_num}"
        )


def get_stream(block_num : int = None):
    """Open up a stream from Hive either live or history"""

    # If you want instant confirmation, you need to instantiate
    # class:beem.blockchain.Blockchain with mode="head",
    # otherwise, the call will wait until confirmed in an irreversible block.
    # noinspection PyTypeChecker

    if block_num:
        # History
        stream = Config.blockchain.stream(
            opNames=["custom_json"],
            start=block_num,
            max_batch_size=50,
            raw_ops=False,
            threading=False,
        )
    else:
        # Live
        stream = Config.blockchain.stream(
            opNames=["custom_json"], raw_ops=False, threading=False
        )
    return stream


def scan_chain(history: bool):
    """Either scans the old chain (history == True) or watches the live blockchain"""

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

    if history:
        report_period_start_time = Config.start_time
        current_block_num = Config.block_num
        stream = get_stream(Config.block_num)
        if Config.reports:
            logging.info("Started catching up")

    else:
        report_period_start_time = datetime.utcnow()
        current_block_num = Config.blockchain.get_current_block_num()
        stream = get_stream()
        if Config.reports:
            logging.info(f"Watching live from block_num: {current_block_num}")

    post = None
    try:
        for post in stream:
            post_time = post["timestamp"].replace(tzinfo=None)
            time_dif = post_time - report_period_start_time
            time_to_now = datetime.utcnow() - post_time
            count_posts += 1
            if Config.reports:
                if time_dif > report_timedelta:
                    timestamp = post["timestamp"]
                    current_block_num = post["block_num"]
                    if time_to_now.seconds < 1: time_to_now = timedelta(seconds=1)
                    output_status(
                        timestamp, pings, count_posts, time_to_now, current_block_num
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
                if post["id"] in list(Config.DIAGNOSTIC_OPERATION_IDS):
                    output_diagnostic(post)

            if history:
                if time_to_now < timedelta(seconds=2) or post_time > Config.stop_at:
                    timestamp = post["timestamp"]
                    current_block_num = post["block_num"]
                    if Config.show_reports:
                        output_status(
                            timestamp, pings, count_posts, time_to_now, current_block_num
                        )

                    if not (Config.urls_only):
                        logging.info(f"block_num: {post['block_num']}")
                    # Break out of the for loop we've caught up.
                    break
            else:
                if time_dif > timedelta(hours=1):
                    # Re-fetch the allowed_accounts every hour in case we add one.
                    allowed_accounts = get_allowed_accounts()


    except Exception as ex:
        logging.error(f"Exception: {ex}")
        logging.warning("Exception being handled - restarting")
        raise UnspecifiedHiveException(ex)

    if post and (not (Config.urls_only)):
        scan_time = datetime.utcnow() - scan_start_time
        logging.info(
            f"Finished catching up at block_num: {post['block_num']} in {scan_time}"
        )


def main() -> None:
    logging.basicConfig(
        level=logging.INFO,
        format=f"%(asctime)s - %(levelname)s %(name)s %(threadName)s : -  %(message)s",
        datefmt="%Y-%m-%dT%H:%M:%S%z",
    )
    Config.setup()

    """ do we want periodic reports? """
    if Config.show_reports:
        if Config.use_test_node:
            logging.info("---------------> Using Test Node " + Config.TEST_NODE[0])
        else:
            logging.info("---------------> Using Main Hive Chain ")

    # scan_history will look back over the last 1 hour reporting every 15 minute chunk
    if Config.history:
        scan_chain(history=True)

    if not Config.history_only or Config.stop_after:
        # scan_live will resume live scanning the chain and report every 5 minutes or
        # when a notification
        #
        scan_chain(history=False)
    else:
        logging.info("history_only is set. exiting")


if __name__ == "__main__":
    try:
        main()
    except Exception as ex:
        logging.error(f"Error: {ex}")
        logging.error("Restarting the watcher")
        Config.old = 1
        main()
