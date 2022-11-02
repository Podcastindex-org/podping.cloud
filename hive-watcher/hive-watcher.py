import itertools
import json
import logging
import sys
import time
from collections import deque
from datetime import timedelta
from timeit import default_timer as timer
from typing import Set

import backoff
import pendulum
from lighthive.client import Client
from lighthive.exceptions import RPCNodeException

from config import Config


class Pings:
    total_pings = 0


class UnspecifiedHiveException(Exception):
    pass


def get_client(
    connect_timeout=3,
    read_timeout=30,
    loglevel=logging.WARN,
    automatic_node_selection=False,
    api_type="condenser_api",
) -> Client:
    try:
        nodes = [
            "https://api.hive.blog",
            "https://api.deathwing.me",
            "https://hive-api.arcange.eu",
            "https://api.openhive.network",
        ]
        client = Client(
            connect_timeout=connect_timeout,
            nodes=nodes,
            read_timeout=read_timeout,
            loglevel=loglevel,
            automatic_node_selection=automatic_node_selection,
            backoff_mode=backoff.fibo,
            backoff_max_tries=3,
            load_balance_nodes=True,
            circuit_breaker=True,
        )
        return client(api_type)
    except Exception as ex:
        raise ex


def get_allowed_accounts(
    client: Client = None, account_name: str = "podping", num_retires = 3
) -> Set[str]:
    """get a list of all accounts allowed to post by acc_name (podping)
    and only react to these accounts"""

    if not client:
        client = get_client(connect_timeout=3, read_timeout=3)

    for _ in range(num_retires):
        try:
            master_account = client.account(account_name)
            return set(master_account.following())
        except (KeyError, RPCNodeException):
            logging.warning(f"Unable to get account followers - retrying")
        except Exception as e:
            logging.warning(f"Unable to get account followers: {e} - retrying")
        finally:
            client.next_node()

def allowed_op_id(operation_id: str) -> bool:
    """Checks if the operation_id is in the allowed list"""
    return Config.OPERATION_REGEX.match(operation_id) is not None


def output(post) -> int:
    """Prints out the post and extracts the custom_json"""

    data = json.loads(post["op"][1]['json'])

    if Config.json:
        if Config.hive_properties:
            data["hiveTxId"] = post["trx_id"]
            data["hiveBlockNum"] = post["block"]
        print(json.dumps(data))
        if "iris" in data:
            return len(data["iris"])
        if "urls" in data:
            return data["num_urls"]
        if "url" in data:
            return 1
        return -1

    data["medium_reason"] = "podcast update"

    # Check version of Podping and :
    if data.get("version") == "1.0":
        if data.get("iris"):
            data["urls"] = data.get("iris")
            data["num_urls"] = len(data["iris"])
            data["medium_reason"] = f"{data.get('medium')} {data.get('reason')}"

    if Config.quiet:
        if data.get("num_urls"):
            return data.get("num_urls")
        else:
            return 1

    if Config.urls_only or Config.json:
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

    data["trx_id"] = post["trx_id"]
    data["timestamp"] = post["timestamp"]

    count = 0
    if Config.use_test_node:
        data["test_node"] = True

    if data.get("url"):
        logging.info(
            f"Feed Updated | {data['timestamp']} | {data['trx_id']} "
            f"| {data.get('url')} | {post['op'][1]['required_posting_auths']}"
            f" | {data['medium_reason']}"
        )
        count = 1
    elif data.get("urls"):
        for url in data.get("urls"):
            count += 1
            logging.info(
                f"Feed Updated | {data['timestamp']} | {data['trx_id']}"
                f" | {url} | {post['op'][1]['required_posting_auths']}"
                f" | {data['medium_reason']}"
            )
    return count


def output_diagnostic(post: dict) -> None:
    """Just output Diagnostic messages recorded on the chain"""
    data = json.loads(post.get("json"))
    if Config.diagnostic:
        logging.info(
            f"Diagnostic | {post['timestamp']} "
            f"| {data.get('server_account')} | {post['trx_id']} | {data.get('message')}"
        )
        logging.info(json.dumps(data, indent=2))


def output_status(
    timestamp: str,
    pings: int,
    count_posts: int,
    time_to_now: timedelta = None,
    current_block_num: int = "",
) -> None:
    """Writes out a status update at with some count data"""
    if not Config.reports and Config.quiet:
        return None
    if time_to_now:
        logging.info(
            f"{timestamp} | Podpings: {pings:7} / {Pings.total_pings:10} | Count:"
            f" {count_posts:12} | BlockNum: {current_block_num} | Time Delta:"
            f" {time_to_now}"
        )
    else:
        logging.info(
            f"{timestamp} | Podpings: {pings:7} / {Pings.total_pings:10} | Count:"
            f" {count_posts:12} | BlockNum: {current_block_num}"
        )


def historical_block_stream_generator(client, start_block, end_block):
    batch_size = 50
    num_in_batch = 0

    current_batch = deque()
    for block_num in range(start_block, end_block):
        client.get_ops_in_block(block_num, batch=True)
        current_batch.append(block_num)
        num_in_batch += 1
        if num_in_batch == batch_size or block_num == end_block:
            while True:
                try:
                    batch = client.process_batch()
                    current_batch.clear()
                    break
                except RPCNodeException:
                    for b in current_batch:
                        client.get_ops_in_block(b, batch=True)
            for ops in batch:
                for post in ops:
                    if post['op'][0] == 'custom_json':
                        yield post
            num_in_batch = 0


def listen_for_custom_json_operations(condenser_api_client, start_block):
    current_block = start_block
    if not current_block:
        current_block = condenser_api_client.get_dynamic_global_properties()["head_block_number"]
    block_client = get_client(connect_timeout=3, read_timeout=3, automatic_node_selection=True, api_type="block_api")
    while True:
        start_time = timer()
        while True:
            try:
                head_block = condenser_api_client.get_dynamic_global_properties()["head_block_number"]
                break
            except (KeyError, RPCNodeException):
                pass
        while (head_block - current_block) > 0:
            while True:
                try:
                    block = block_client.get_block({"block_num": current_block})
                    break
                except RPCNodeException:
                    pass
            try:
                for op in [(trx_id, op) for trx_id, transaction in enumerate(block['block']['transactions']) for op in transaction['operations']]:
                    if op[1]['type'] == 'custom_json_operation':
                        yield {
                            "block": current_block,
                            "timestamp": block['block']['timestamp'],
                            "trx_id": block['block']['transaction_ids'][op[0]],
                            "op": [
                                'custom_json',
                                op[1]['value'],
                            ]
                        }
            except KeyError:
                logging.warning(f"Block {current_block} is invalid")
            current_block += 1
            while True:
                try:
                    head_block = condenser_api_client.get_dynamic_global_properties()["head_block_number"]
                    break
                except (KeyError, RPCNodeException):
                    pass
        end_time = timer()
        sleep_time = 3 - (end_time - start_time)
        if sleep_time > 0 and (head_block - current_block) <= 0:
            time.sleep(sleep_time)



def scan_chain(client: Client, history: bool, start_block=None):
    """Either scans the old chain (history == True) or watches the live blockchain"""

    # Very first transaction from Dave Testing:
    """2021-05-10 13:51:58,353 INFO root MainThread
     : Feed Updated - 2021-05-07 20:58:33+00:00
     - f0affd194524a6e0171d65d29d5c501865f0bd72
     - https://feeds.transistor.fm/retail-remix"""

    scan_start_time = pendulum.now()
    report_timedelta = pendulum.duration(minutes=Config.report_minutes)

    #allowed_accounts = get_allowed_accounts(client)
    #allowed_accounts_start_time = pendulum.now()

    count_posts = 0
    pings = 0

    if history:
        report_period_start_time = Config.start_time
        end_block = client.get_dynamic_global_properties()["head_block_number"]
        stream = historical_block_stream_generator(client, start_block, end_block + 1)
        if Config.reports:
            logging.info(f"Started catching up from block_num: {start_block}")

    else:
        report_period_start_time = pendulum.now()
        #event_listener = EventListener(client, "head", start_block=start_block)
        #stream = event_listener.on("custom_json")
        stream = listen_for_custom_json_operations(client, start_block)
        if Config.reports:
            logging.info(f"Watching live from block_num: {start_block}")

    post = None
    try:
        for post in stream:
            post_time = pendulum.parse(post["timestamp"])
            time_dif = post_time - report_period_start_time
            time_to_now = pendulum.now() - post_time
            count_posts += 1
            if Config.reports:
                if time_dif > report_timedelta:
                    timestamp = post["timestamp"]
                    current_block_num = post["block"]
                    if time_to_now.seconds < 1:
                        time_to_now = pendulum.duration(seconds=1)
                    output_status(
                        timestamp, pings, count_posts, time_to_now, current_block_num
                    )
                    report_period_start_time = pendulum.parse(post["timestamp"])
                    count_posts = 0
                    pings = 0

            if allowed_op_id(post["op"][1]["id"]):
                #if set(post["op"][1]["required_posting_auths"]) & allowed_accounts:
                count = output(post)
                pings += count
                Pings.total_pings += count

            if Config.diagnostic:
                if post["op"][1]["id"] in list(Config.DIAGNOSTIC_OPERATION_IDS):
                    output_diagnostic(post)

            if history:
                if time_to_now < pendulum.duration(seconds=2) or post_time > Config.stop_at:
                    timestamp = post["timestamp"]
                    current_block_num = post["block"]
                    if Config.show_reports and not Config.urls_only:
                        output_status(
                            timestamp,
                            pings,
                            count_posts,
                            time_to_now,
                            current_block_num,
                        )

                    if not (Config.urls_only):
                        logging.info(f"block_num: {post['block']}")
                    # Break out of the for loop we've caught up.
                    break
            #else:
            #    allowed_accounts_time_diff = pendulum.now() - allowed_accounts_start_time
            #    if allowed_accounts_time_diff > pendulum.duration(hours=1):
            #        # Re-fetch the allowed_accounts every hour in case we add one.
            #        allowed_accounts = get_allowed_accounts()
            #        allowed_accounts_start_time = pendulum.now()


    except Exception as ex:
        logging.exception(ex)
        logging.error(f"Exception: {ex}")
        logging.error(f"Error with node {client.current_node}")
        logging.warning("Exception being handled | restarting")
        raise UnspecifiedHiveException(ex)

    if post and (not (Config.urls_only)):
        scan_time = pendulum.now() - scan_start_time
        logging.info(
            f"Finished catching up at block_num: {post['block']} in {scan_time}"
        )

    if post:
        return post['block']


def main() -> None:
    logging.getLogger("lighthive.client").setLevel(logging.INFO)
    logging.basicConfig(
        level=logging.INFO,
        format=f"%(asctime)s | %(levelname)s %(name)s %(threadName)s : |  %(message)s",
        datefmt="%Y-%m-%dT%H:%M:%S%z",
    )
    Config.setup()

    """ do we want periodic reports? """
    if Config.show_reports:
        if Config.use_test_node:
            logging.info("---------------> Using Test Node " + Config.TEST_NODE[0])
        else:
            logging.info("---------------> Using Main Hive Chain ")

    client = get_client(connect_timeout=3, read_timeout=3, automatic_node_selection=False)
    start_block = None

    # scan_history will look back over the last 1 hour reporting every 15 minute chunk
    if Config.history:
        start_block = scan_chain(client, history=True, start_block=Config.block_num)

    if start_block is None:
        while True:
            try:
                start_block = client.get_dynamic_global_properties()["head_block_number"]
                break
            except RPCNodeException:
                pass
    else:
        start_block += 1

    if not Config.history_only or Config.stop_after:
        # scan_live will resume live scanning the chain and report every 5 minutes or
        # when a notification
        #
        scan_chain(client, history=False, start_block=start_block)
    else:
        logging.info("history_only is set. exiting")
        sys.exit(0)


if __name__ == "__main__":
    while True:
        try:
            main()
        except KeyboardInterrupt:
            logging.info("Terminated with Ctrl-C")
            sys.exit(1)
        except Exception as ex:
            logging.error(f"Error: {ex}", exc_info=True)
            logging.error("Restarting the watcher")
            Config.old = 1
            main()
