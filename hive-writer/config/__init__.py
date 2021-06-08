
import logging
import os
import argparse
import queue
import threading


# Testnet instead of main Hive
# BOL: Switching off TestNet, we should test on Hive for now.



# ---------------------------------------------------------------
# COMMAND LINE
# ---------------------------------------------------------------

app_description = """ PodPing - Runs as a server and writes a stream of URLs to the Hive Blockchain or sends a single URL to Hive (--url option) """


my_parser = argparse.ArgumentParser(prog='hive-writer',
                                    usage='%(prog)s [options]',
                                    description= app_description,
                                    epilog='')


group_noise = my_parser.add_mutually_exclusive_group()
group_noise.add_argument('-q', '--quiet', action='store_true', help='Minimal output')
group_noise.add_argument('-v', '--verbose', action='store_true', help='Lots of output')


group_action_type = my_parser.add_mutually_exclusive_group()
group_action_type.add_argument('-s', '--socket',
                       action='store', type=int, required=False,
                       metavar='',
                       default= None,
                       help='<port> Socket to listen on for each new url, returns either ')
group_action_type.add_argument('-z', '--zmq',
                       action='store', type=int, required=False,
                       metavar='',
                       default= None,
                       help='<port> for ZMQ to listen on for each new url, returns either ')

group_action_type.add_argument('-u', '--url',
                       action='store',
                       type=str,
                       required=False,
                       metavar='',
                       default=None,
                       help="<url> Takes in a single URL and sends a single podping to Hive, needs HIVE_SERVER_ACCOUNT and HIVE_POSTING_KEY ENV variables set")

my_parser.add_argument("-t",
                    "--test",
                    action="store_true",
                    required=False, help="Use a test net API"
)

my_parser.add_argument('-e', '--errors',
                       action='store', type=int, required=False,
                       metavar='',
                       default=None,
                       help='Deliberately force error rate of <int>%%')

args = my_parser.parse_args()
my_args = vars(args)





class Config():
    """ The Config Class """
    TEST_NODE = ['http://testnet.openhive.network:8091']
    CURRENT_PODPING_VERSION = 2
    NOTIFICATION_REASONS = {
        'feed_update' : 1,
        'new_feed' : 2,
        'host_change' : 3
    }


    HIVE_OPERATION_PERIOD = 3       # 1 Hive operation per this period in
    MAX_URL_PER_CUSTOM_JSON = 90   # total json size must be below 8192 bytes
    MAX_URL_LIST_BYTES = 7000

    # This is a global signal to shut down until RC's recover
    # Stores the RC cost of each operation to calculate an average
    HALT_THE_QUEUE = False
    # HALT_TIME = [1,2,3]
    HALT_TIME = [0,1,1,1,1,1,1,1,3,6,9,15,15,15,15,15,15,15]




    # ---------------------------------------------------------------
    # START OF STARTUP SEQUENCE
    # ---------------------------------------------------------------
    # GLOBAL:
    server_account = os.getenv('HIVE_SERVER_ACCOUNT')
    wif = [os.getenv('HIVE_POSTING_KEY')]

    # Adding a Queue system to the Hive send_notification section
    hive_q = queue.Queue()
    # Move the URL Q into a proper Q
    url_q = queue.Queue()

    socket = my_args["socket"]
    url = my_args["url"]
    zmq = my_args["zmq"]
    errors = my_args["errors"]

    # FROM ENV or from command line.
    test = os.getenv("USE_TEST_NODE", 'False').lower() in ('true', '1', 't')
    if my_args["test"]:
        test = True


    @classmethod
    def setup(cls):
        """ Setup the config """

        pass