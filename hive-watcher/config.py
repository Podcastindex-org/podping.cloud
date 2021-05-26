import argparse
import os


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
    help=("Time in hours to replay for from the start point")
)

block_history_argument_group.add_argument(
    "-y",
    "--startdate",
    action="store",
    type=str,
    required=False,
    metavar="",
    default=0,
    help=("<%%Y-%%m-%%d %%H:%%M:%%S> Date/Time to start the history"),
)


my_parser.add_argument(
    "-H",
    "--history-only",
    action="store_true",
    required=False,
    help="Report history only and exit",
)

my_parser.add_argument(
    "-d",
    "--diagnostic",
    action="store_true",
    required=False,
    help=("Show diagnostic posts written to the blockchain")
)

my_parser.add_argument(
    "-u",
    "--urls_only",
    action="store_true",
    required=False,
    help=("Just output the urls on a single line, nothing else")
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

my_parser.add_argument(
    "-s",
    "--socket",
    action="store",
    type=str,
    required=False,
    metavar="",
    default=None,
    help="<IP-Address>:<port> Socket to send each new url to",
)

my_parser.add_argument(
    "-t", "--test", action="store_true", required=False, help="Use a test net API"
)

my_parser.set_defaults(history_only=False)

group = my_parser.add_mutually_exclusive_group()
group.add_argument("-q", "--quiet", action="store_true", help="Minimal output")
group.add_argument("-v", "--verbose", action="store_true", help="Lots of output")



args = my_parser.parse_args()
my_args = vars(args)


class Config():
    test = my_args["test"]
    quiet = my_args["quiet"]
    reports = my_args["reports"]
    old = my_args["old"]
    diagnostic = my_args["diagnostic"]
    urls_only = my_args["urls_only"]
    stop_after = my_args["stop_after"]
    use_socket = my_args["socket"]


    @classmethod
    def setup(cls):
        """ Setup the config """
        if cls.test:
            cls.use_test_node = True
        else:
            cls.use_test_node: bool = os.getenv("USE_TEST_NODE", "False").lower() in {
                "true",
                "1",
                "t",
            }

        if cls.reports == 0:
            cls.show_reports = False
        else:
            cls.show_reports = True
            cls.report_minutes = cls.reports
