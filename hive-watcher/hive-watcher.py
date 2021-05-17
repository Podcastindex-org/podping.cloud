import argparse
import json
import logging
import os
from datetime import datetime, timedelta
from socket import AF_INET, SOCK_STREAM, socket
from typing import Set

from beem import Hive
from beem.account import Account
from beem.block import Block
from beem.blockchain import Blockchain
from beem.nodelist import NodeList

USE_TEST_NODE = os.getenv("USE_TEST_NODE", 'False').lower() in ('true', '1', 't')
WATCHED_OPERATION_IDS = ['podping','hive-hydra']
TEST_NODE = ['http://testnet.openhive.network:8091']
total_pings = 0

logging.basicConfig(level=logging.INFO,
                    format=f'%(asctime)s - %(levelname)s %(name)s %(threadName)s : -  %(message)s')

if USE_TEST_NODE:
    hive = Hive(node=TEST_NODE)
else:
    nodelist = NodeList()
    nodelist.update_nodes()
    # hive = Hive(node = nodelist.get_hive_nodes())
    hive = Hive()

app_description = """PodPing - Watch the Hive Blockchain for notifications of new Podcast Episodes
\n\n
This code will run until terminated reporting every notification of a new Podcast Episode sent to the Hive blockchain by any PodPing servers.

With default arguments it will print to the StdOut a log of each new URL that has updated interspersed with summary lines every 5 minutes that list the number of PodPings and the number of other 'custom_json' operations seen on the blockchain. This interval can be set with the --reports command line."""

my_parser = argparse.ArgumentParser(prog='hive-watcher',
                                    usage='%(prog)s [options]',
                                    description= app_description,
                                    epilog='')


# my_parser.add_argument('-q', '--quiet',
#                        action=)

group_old = my_parser.add_argument_group()
group_old.add_argument('-b', '--block',
                       action='store', type=int, required=False,
                       metavar='',
                       help='Hive Block number to start replay at or use:')

group_old.add_argument('-o',
                       '--old',
                       action='store', type=int, required=False,
                       metavar='',
                       default=0,
                       help='Time in HOURS to look back up the chain for old pings (default is 0)')



my_parser.add_argument('-r',
                       '--reports',
                       action='store', type=int, required=False,
                       metavar='',
                       default=5,
                       help='Time in MINUTES between periodic status reports, use 0 for no periodic reports')

my_parser.add_argument('-s', '--socket',
                       action='store', type=str, required=False,
                       metavar='',
                       default= None,
                       help='<IP-Address>:<port> Socket to send each new url to')

my_parser.add_argument('-t', '--test',
                       action='store_true', required=False,
                       help="Use a test net API")

group = my_parser.add_mutually_exclusive_group()
group.add_argument('-q', '--quiet', action='store_true', help='Minimal output')
group.add_argument('-v', '--verbose', action='store_true', help='Lots of output')

def get_allowed_accounts(acc_name='podping') -> Set[Account]:
    """ get a list of all accounts allowed to post by acc_name (podping)
        and only react to these accounts """

    # Switching to a simpler authentication system. Only podpings from accounts which
    # the PODPING Hive account FOLLOWS will be watched.

    # This is giving an error if I don't specify api server exactly.
    #TODO reported as Issue on Beem library https://github.com/holgern/beem/issues/301
    h = Hive(node='https://api.hive.blog')

    master_account = Account(acc_name, blockchain_instance=h, lazy=True)

    return set(master_account.get_following())

def allowed_op_id(operation_id):
    """ Checks if the operation_id is in the allowed list """
    if operation_id in WATCHED_OPERATION_IDS:
        return True
    else:
        return False


def output(post) -> int:
    """ Prints out the post and extracts the custom_json """

    data = json.loads(post.get('json'))
    if myArgs.get('quiet'):
        if data.get('num_urls'):
            return data.get('num_urls')
        else:
            return 1
    data['required_posting_auths'] = post.get('required_posting_auths')
    data['trx_id'] = post.get('trx_id')
    data['timestamp'] = post.get('timestamp')

    count = 0
    if USE_TEST_NODE:
        data['test_node'] = True
    if data.get('url'):
        logging.info('Feed Updated - ' + str(data.get('timestamp')) + ' - ' + data.get('trx_id') + ' - ' + data.get('url'))
        count = 1
    elif data.get('urls'):
        for url in data.get('urls'):
            count += 1
            logging.info('Feed Updated - ' + str(data.get('timestamp')) + ' - ' + data.get('trx_id') + ' - ' + url )
    return count

def output_status(timestamp, pings, count_posts, time_to_now='', current_block_num='') -> None:
    """ Writes out a status update at with some count data """
    if (not myArgs.get('reports')) and myArgs.get('quiet'):
        return None
    if time_to_now:
        logging.info(f'{timestamp} - Podpings: {pings:7,} / {total_pings:10,} - Count: {count_posts} - BlockNum: {current_block_num} - Time Delta: {time_to_now}')
    else:
        logging.info(f'{timestamp} - Podpings: {pings:7,} / {total_pings:10,} - Count: {count_posts} - BlockNum: {current_block_num}')


def output_to_socket(post, clientSocket) -> None:
    """ Take in a post and a socket and send the url to a socket """
    if not(myArgs['socket']):
        return None
    data = json.loads(post.get('json'))
    url = data.get('url')
    if url:
        try:
            clientSocket.send((url).encode())
        except Exception as ex:
            error_message = f'{ex} occurred {ex.__class__}'
            logging.error(error_message)
            open_socket()


    # Do we need to receive from the socket?


def scan_live(report_freq = None, reports = True):
    """ watches the stream from the Hive blockchain """
    global total_pings
    if type(report_freq) == int:
        report_freq = timedelta(minutes=report_freq)

    allowed_accounts = get_allowed_accounts()

    blockchain = Blockchain(mode="head", blockchain_instance=hive)
    current_block_num = blockchain.get_current_block_num()
    if reports:
        logging.info('Watching live from block_num: ' + str(current_block_num))

    # If you want instant confirmation, you need to instantiate
    # class:beem.blockchain.Blockchain with mode="head",
    # otherwise, the call will wait until confirmed in an irreversible block.
    stream = blockchain.stream(opNames=['custom_json'], raw_ops=False, threading=False, thread_num=4)

    start_time = datetime.utcnow()
    count_posts = 0
    pings = 0

    for post in stream:
        count_posts +=1
        time_dif = post['timestamp'].replace(tzinfo=None) - start_time
        if reports:
            if time_dif > report_freq:
                current_block_num = str(blockchain.get_current_block_num())
                timestamp = str(post['timestamp'])
                output_status(timestamp, pings, count_posts, current_block_num=current_block_num)
                start_time =post['timestamp'].replace(tzinfo=None)
                count_posts = 0
                pings = 0

        if allowed_op_id(post['id']):
            if set(post['required_posting_auths']) & allowed_accounts:
                count = output(post)
                if myArgs['socket']:
                    output_to_socket(post, clientSocket)
                pings += count
                total_pings += count

        if time_dif > timedelta(hours=1):
            # Refetch the allowed_accounts every hour in case we add one.
            allowed_accounts = get_allowed_accounts()

def scan_history(param= None, report_freq = None, reports = True):
    """ Scans back in history timed time delta ago, reporting with report_freq
        if timed is an int, treat it as hours, if report_freq is int, treat as min """
    global total_pings
    # Very first transaction from Dave Testing:
    # 2021-05-10 13:51:58,353 INFO root MainThread : Feed Updated - 2021-05-07 20:58:33+00:00 - f0affd194524a6e0171d65d29d5c501865f0bd72 - https://feeds.transistor.fm/retail-remix

    scan_start_time = datetime.utcnow()

    if not report_freq:
        report_freq = timedelta(minutes=5)

    if not param:
        timed = timedelta(hours=1)

    blockchain = Blockchain(mode="head", blockchain_instance=hive)
    if type(param) == int:
        block_num = param
        start_time = Block(block_num)['timestamp'].replace(tzinfo=None)
    else:
        start_time = datetime.utcnow() - param
        block_num = blockchain.get_estimated_block_num(start_time)

    if type(report_freq) == int:
        report_freq = timedelta(minutes=report_freq)

    allowed_accounts = get_allowed_accounts()

    count_posts = 0
    pings = 0

    if reports:
        logging.info('Started catching up')
    stream = blockchain.stream(opNames=['custom_json'], start = block_num,
                               max_batch_size = 50,
                               raw_ops=False, threading=False)

    for post in stream:
        post_time = post['timestamp'].replace(tzinfo=None)
        time_dif = post_time - start_time
        time_to_now = datetime.utcnow() - post_time
        count_posts += 1
        if reports:
            if time_dif > report_freq:
                timestamp = str(post['timestamp'])
                current_block_num = post['block_num']
                output_status(timestamp, pings, count_posts, time_to_now, current_block_num=current_block_num)
                start_time =post['timestamp'].replace(tzinfo=None)
                count_posts = 0
                pings = 0

        if allowed_op_id(post['id']):
            if set(post['required_posting_auths']) & allowed_accounts:
                count = output(post)
                pings += count
                total_pings += count

        if time_to_now < timedelta(seconds=2):
            timestamp = str(post['timestamp'])
            current_block_num = post['block_num']
            output_status(timestamp, pings, count_posts, time_to_now, current_block_num=current_block_num)
            logging.info('block_num: ' + str(post['block_num']))
            # Break out of the for loop we've caught up.
            break

    scan_time = datetime.utcnow() - scan_start_time
    logging.info('Finished catching up at block_num: ' + str(post['block_num']) + ' in '+ str(scan_time))

def open_socket():
    """ If a socket errors out and will try to reopen it """
    try:
        clientSocket.connect((ip_address,port))
    except Exception as ex:
        error_message = f'{ex} occurred {ex.__class__}'
        logging.error(error_message)



args = my_parser.parse_args()
myArgs = vars(args)

if myArgs['socket']:
    ip_port = myArgs['socket'].split(':')
    ip_address = ip_port[0]
    port = int(ip_port[1])
    clientSocket = socket(AF_INET, SOCK_STREAM)
    open_socket()



def main() -> None:
    """ Main file """
    global hive
    global USE_TEST_NODE
    if myArgs['test']:
        USE_TEST_NODE = True
        hive = Hive(node=TEST_NODE)

    """ do we want periodic reports? """
    if myArgs['reports'] == 0:
        reports = False
    else:
        reports = True
        if USE_TEST_NODE:
            logging.info('---------------> Using Test Node ' + TEST_NODE[0])
        else:
            logging.info('---------------> Using Main Hive Chain ')

    """ scan_history will look back over the last 1 hour reporting every 15 minute chunk """
    if myArgs['old'] or myArgs['block']:
        if myArgs['block']:
            param = myArgs['block']
        else:
            param = timedelta(hours = myArgs['old'])

        scan_history(param, myArgs['reports'], reports)

    """ scan_live will resume live scanning the chain and report every 5 minutes or when
        a notification arrives """
    scan_live(myArgs['reports'],reports)



if __name__ == "__main__":

    main()
