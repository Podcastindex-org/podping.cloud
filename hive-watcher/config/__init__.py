import argparse
import math
import os
import re
from socket import AF_INET, SOCK_STREAM, socket
from ipaddress import IPv4Address, IPv6Address, AddressValueError

import zmq
from lighthive.client import Client
import pendulum
from lighthive.exceptions import RPCNodeException

BLOCK_INTERVAL = 3


# Adapted from beem under the MIT license
# https://github.com/holgern/beem/blob/master/beem/blockchain.py#L307-L364
def get_estimated_block_num(client: Client, date, accurate=True):

    last_block_num = client.get_dynamic_global_properties()["head_block_number"]
    while True:
        try:
            last_block = client.get_block(last_block_num)
            break
        except RPCNodeException:
            pass
    time_diff = pendulum.parse(last_block['timestamp']) - date
    block_number = math.floor(last_block_num - time_diff.total_seconds() / BLOCK_INTERVAL)
    if block_number < 1:
        block_number = 1

    if accurate:
        if block_number > last_block_num:
            block_number = last_block_num
        block_time_diff = pendulum.duration(seconds=10)

        last_block_time_diff_seconds = 10
        second_last_block_time_diff_seconds = 10

        while block_time_diff.total_seconds() > BLOCK_INTERVAL or block_time_diff.total_seconds() < -BLOCK_INTERVAL:
            while True:
                try:
                    block = client.get_block(block_number)
                    break
                except RPCNodeException:
                    pass
            second_last_block_time_diff_seconds = last_block_time_diff_seconds
            last_block_time_diff_seconds = block_time_diff.total_seconds()
            block_time_diff = date - pendulum.parse(block['timestamp'])
            if second_last_block_time_diff_seconds == block_time_diff.total_seconds() and second_last_block_time_diff_seconds < 10:
                return int(block_number)
            delta = block_time_diff.total_seconds() // BLOCK_INTERVAL
            if delta == 0 and block_time_diff.total_seconds() < 0:
                delta = -1
            elif delta == 0 and block_time_diff.total_seconds() > 0:
                delta = 1
            block_number += delta
            if block_number < 1:
                break
            if block_number > last_block_num:
                break

    return int(block_number)


TEST_NODE = ["https://testnet.openhive.network"]

app_description = """PodPing - Watch the Hive Blockchain for notifications of new
Podcast Episodes


This code will run until terminated reporting every
notification of a new Podcast Episode sent to the Hive blockchain by any PodPing
servers.

With default arguments it will print to the StdOut a log of each new URL that has
updated interspersed with summary lines every 5 minutes that list the number of
PodPings and the number of other 'custom_json' operations seen on the blockchain.
This interval can be set with the --reports command line.
"""

my_parser = argparse.ArgumentParser(
    prog="hive-watcher",
    usage="%(prog)s [options]",
    description=app_description,
    epilog="",
)

block_history_argument_group = my_parser.add_argument_group()
block_history_argument_group.add_argument(
    "-b",
    "--block",
    action="store",
    type=int,
    required=False,
    metavar="",
    help="Hive Block number to start replay at or use:",
)

block_history_argument_group.add_argument(
    "-e",
    "--unix_epoch",
    action="store",
    type=int,
    required=False,
    metavar="",
    help="Unix timestamp to start the history",
)

block_history_argument_group.add_argument(
    "-o",
    "--old",
    action="store",
    type=int,
    required=False,
    metavar="",
    default=0,
    help="Time in HOURS to look back up the chain for old pings (default is 0)",
)


block_history_argument_group.add_argument(
    "-a",
    "--stop_after",
    action="store",
    type=int,
    required=False,
    metavar="",
    default=0,
    help=("Time in hours to replay for from the start point"),
)

block_history_argument_group.add_argument(
    "-y",
    "--start_date",
    action="store",
    type=str,
    required=False,
    metavar="",
    default=0,
    help=("ISO 8601/RFC 3339 Date/Time to start the history:  See https://pendulum.eustace.io/docs/#parsing"),
)


my_parser.add_argument(
    "-H",
    "--history_only",
    action="store_true",
    required=False,
    help="Report history only and exit",
)

my_parser.add_argument(
    "-d",
    "--diagnostic",
    action="store_true",
    required=False,
    help=("Show diagnostic posts written to the blockchain"),
)

my_parser.add_argument(
    "-u",
    "--urls_only",
    action="store_true",
    required=False,
    help=("Just output the urls on a single line, nothing else"),
)

my_parser.add_argument(
    "-j",
    "--json",
    action="store_true",
    required=False,
    help=("Just output the urls on a single line as json, nothing else"),
)

my_parser.add_argument(
    "-r",
    "--reports",
    action="store",
    type=int,
    required=False,
    metavar="",
    default=5,
    help=(
        "Time in MINUTES between periodic status reports, use 0 for no periodic reports"
    ),
)

group_zmq_socket = my_parser.add_mutually_exclusive_group()
group_zmq_socket.add_argument(
    "-s",
    "--socket",
    action="store",
    type=str,
    required=False,
    metavar="",
    default=None,
    help="<IP-Address>:<port> Socket to send each new url to",
)

group_zmq_socket.add_argument(
    "-z",
    "--zmq",
    action="store",
    type=str,
    required=False,
    metavar="",
    default=None,
    help="<IP-Address>:<port> for ZMQ to send each new url to (if no IP given, defaults to 127.0.0.1)",
)

my_parser.add_argument(
    "-t", "--test", action="store_true", required=False, help="Use a test net API"
)

my_parser.add_argument(
    "-l",
    "--livetest",
    action="store_true",
    required=False,
    help="Watch live Hive chain but looking for id=podping-livetest",
)


my_parser.set_defaults(history_only=False)

group = my_parser.add_mutually_exclusive_group()
group.add_argument("-q", "--quiet", action="store_true", help="Minimal output")
group.add_argument("-v", "--verbose", action="store_true", help="Lots of output")


args = my_parser.parse_args()
my_args = vars(args)


class Config:

    WATCHED_OPERATION_IDS = ["pp_", "podping"]
    OPERATION_REGEX = re.compile(r"^pp_(.*)_(.*)|podping$")
    DIAGNOSTIC_OPERATION_IDS = ["podping-startup", "pp_startup"]
    TEST_NODE = ["https://testnet.openhive.network"]

    test = my_args["test"]
    quiet = my_args["quiet"]
    reports = my_args["reports"]
    block_num = my_args["block"]
    unix_epoch = my_args["unix_epoch"]
    start_date = my_args["start_date"]
    history_only = my_args["history_only"]
    old = my_args["old"]
    diagnostic = my_args["diagnostic"]
    urls_only = my_args["urls_only"]
    json = my_args["json"]
    stop_after = my_args["stop_after"]
    use_socket = my_args["socket"]
    use_zmq = my_args["zmq"]
    livetest = my_args["livetest"]

    @classmethod
    def socket_connect(cls):
        """Connect to a socket"""
        cls.client_socket = socket(AF_INET, SOCK_STREAM)
        try:
            cls.client_socket.connect((cls.ip_address.compressed, cls.port))
        except Exception as ex:
            error_message = f"{ex} occurred {ex.__class__}"
            print(error_message)

    @classmethod
    def socket_send(cls, url):
        """Send a single URL to the socket specifie in startup"""
        if cls.client_socket:
            cls.socket_connect()
            cls.client_socket.send(url.encode())
            cls.client_socket.close

    @classmethod
    def zsocket_send(cls, url):
        """Send a single URL to the zsocket specified in startup"""
        if cls.zsocket:
            # cls.zsocket.RCV = 1000 # in milliseconds
            try:
                cls.zsocket.send_string(url, flags=zmq.NOBLOCK)
                msg = cls.zsocket.recv_string()
            except Exception as ex:
                print(f"Exception: {ex}")

    @classmethod
    def setup(cls):
        """Setup the config"""
        if cls.test:
            cls.use_test_node = True
        else:
            cls.use_test_node: bool = os.getenv("USE_TEST_NODE", "False").lower() in {
                "true",
                "1",
                "t",
            }

        # If reports is 0 no reports otherwise reports is report_minutes frequency
        if cls.reports == 0:
            cls.show_reports = False
            cls.report_minutes = 0
        else:
            cls.show_reports = True
            cls.report_minutes = cls.reports

        # If we have --old = use that or  if --start_date calculate
        # how many hours_ago that is
        if cls.start_date:
            start_date = pendulum.parse(cls.start_date)
            cls.hours_ago = pendulum.now() - start_date
        elif cls.unix_epoch:
            start_date = pendulum.from_timestamp(cls.unix_epoch)
            cls.hours_ago = pendulum.now() - start_date
        else:
            cls.hours_ago = pendulum.duration(hours=cls.old)

        # We are looking for some kind of history
        if cls.old or cls.block_num or cls.unix_epoch or cls.start_date:
            cls.history = True
            client = Client()
            if cls.block_num:
                while True:
                    try:
                        cls.start_time = pendulum.parse(client.get_block(cls.block_num)["timestamp"])
                        break
                    except RPCNodeException:
                        pass
            elif cls.hours_ago:
                cls.start_time = pendulum.now() - cls.hours_ago
                cls.block_num = get_estimated_block_num(client, cls.start_time)
            else:
                raise ValueError(
                    "scan_history: --old=<hours> required to scan history"
                )

            if cls.stop_after > 0:
                cls.stop_at = cls.start_time + pendulum.duration(hours=cls.stop_after)
            else:
                cls.stop_at = pendulum.datetime(year=3333, month=1, day=1)
        else:
            cls.history = False
            cls.start_time = pendulum.now()

        cls.client_socket = None
        if cls.use_socket:
            # TODO: Socket needs testing or conversion to zmq
            ip_port_params = cls.use_socket.split(":")
            try:
                cls.ip_address = IPv4Address(ip_port_params[0])
            except AddressValueError:
                cls.ip_address = IPv6Address(ip_port_params[0])
            cls.port = int(ip_port_params[1])

        cls.zsocket = None
        if cls.use_zmq:
            context = zmq.Context()
            ip_port_params = cls.use_zmq.split(":")
            if len(ip_port_params) == 1:
                cls.ip_address = IPv4Address("127.0.0.1")
                cls.ip_port = ip_port_params[0]
            else:
                cls.ip_port = ip_port_params[1]
                try:
                    cls.ip_address = IPv4Address(ip_port_params[0])
                except AddressValueError:
                    cls.ip_address = IPv6Address(cls.ip_port)

            cls.zsocket = context.socket(zmq.REQ)
            print(f"tcp://{cls.ip_address}:{cls.ip_port}")
            cls.zsocket.connect(f"tcp://{cls.ip_address}:{cls.ip_port}")

        if cls.livetest:
            cls.WATCHED_OPERATION_IDS = ["podping-livetest", "pplt_"]
            cls.OPERATION_REGEX = re.compile(r"^pplt_(.*)_(.*)|podping-livetest$")

        if cls.json:
            cls.urls_only = True

        if cls.urls_only:
            cls.show_reports = False
            cls.reports = 0
