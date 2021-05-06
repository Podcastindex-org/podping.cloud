import json
import logging
import os
from datetime import datetime, timedelta
from time import sleep

from beem import Hive
from beem.account import Account
from beem.blockchain import Blockchain

USE_TEST_NODE = os.getenv("USE_TEST_NODE", 'False').lower() in ('true', '1', 't')
TELEGRAM_ALERTS = True
WATCHED_OPERATION_IDS = ['podping','hive-hydra']
TEST_NODE = ['http://testnet.openhive.network:8091']

if USE_TEST_NODE:
    t_key = os.getenv('TELEGRAM_BOT_KEY_TEST')
else:
    t_key = os.getenv('TELEGRAM_BOT_KEY')

logging.basicConfig(level=logging.INFO,
                    format=f'%(asctime)s %(levelname)s %(name)s %(threadName)s : %(message)s')

if USE_TEST_NODE:
    logging.info('---------------> Using Test Node ' + TEST_NODE[0])
else:
    logging.info('---------------> Using Main Hive Chain ')

if USE_TEST_NODE:
    hive = Hive(node=TEST_NODE)
else:
    hive = Hive()


def get_allowed_accounts(acc_name) -> bool:
    """ get a list of all accounts allowed to post by acc_name (podcastindex)
        and only react to these accounts """

    if USE_TEST_NODE:
        return ['learn-to-code','hive-hydra','hivehydra','flyingboy']

    hiveaccount = Account(acc_name, blockchain_instance=hive, lazy=True)
    try:
        allowed = hiveaccount['posting']['account_auths']
        allowed = [x for (x,_) in allowed]

    except Exception as ex:
        allowed = []

    return allowed

def allowed_op_id(operation_id):
    """ Checks if the operation_id is in the allowed list """
    if operation_id in WATCHED_OPERATION_IDS:
        return True
    else:
        return False

def output(post) -> None:
    """ Prints out the post and extracts the custom_json """
    data = json.loads(post.get('json'))
    data['required_posting_auths'] = post.get('required_posting_auths')
    data['trx_id'] = post.get('trx_id')
    data['timestamp'] = post.get('timestamp')
    if USE_TEST_NODE:
        data['test_node'] = True
    logging.info('Feed Updated - ' + str(data.get('timestamp')) + ' - ' + data.get('trx_id') + ' - ' + data.get('url'))


def scan_live(report_freq = None):
    """ watches the stream from the Hive blockchain """

    if not report_freq:
        report_freq = timedelta(minutes=5)

    if type(report_freq) == int:
        report_freq = timedelta(minutes=report_freq)
    allowed_accounts = get_allowed_accounts('podcastindex')

    blockchain = Blockchain(mode="head", blockchain_instance=hive)
    current_block_num = blockchain.get_current_block_num()
    logging.info('Watching live from block_num: ' + str(current_block_num))

    # If you want instant confirmation, you need to instantiate
    # class:beem.blockchain.Blockchain with mode="head",
    # otherwise, the call will wait until confirmed in an irreversible block.
    stream = blockchain.stream(opNames=['custom_json'], raw_ops=False, threading=False, thread_num=4)

    start_time = datetime.utcnow()
    count_posts = 0

    for post in stream:
        count_posts +=1
        time_dif = post['timestamp'].replace(tzinfo=None) - start_time
        if time_dif > report_freq:
            current_block_num = blockchain.get_current_block_num()
            logging.info(str(post['timestamp']) + " Count: " + str(count_posts) + " block_num: " + str(current_block_num))
            start_time =post['timestamp'].replace(tzinfo=None)
            count_posts = 0

        if allowed_op_id(post['id']):
            if  (set(post['required_posting_auths']) & set(allowed_accounts)):
                output(post)

        if time_dif > timedelta(hours=1):
            # Refetch the allowed_accounts every hour in case we add one.
            allowed_accounts = get_allowed_accounts('podcastindex')

def scan_history(timed= None, report_freq = None):
    """ Scans back in history timed time delta ago, reporting with report_freq
        if timed is an int, treat it as hours, if report_freq is int, treat as min """
    scan_start_time = datetime.utcnow()

    if not report_freq:
        report_freq = timedelta(minutes=5)

    if not timed:
        timed = timedelta(hours=1)

    if type(timed) == int:
        timed = timedelta(hours=timed)

    if type(report_freq) == int:
        report_freq = timedelta(minutes=report_freq)

    allowed_accounts = get_allowed_accounts('podcastindex')

    blockchain = Blockchain(mode="head", blockchain_instance=hive)
    start_time = datetime.utcnow() - timed
    count_posts = 0
    block_num = blockchain.get_estimated_block_num(start_time)

    logging.info('Started catching up')
    stream = blockchain.stream(opNames=['custom_json'], start = block_num,
                               max_batch_size = 50,
                               raw_ops=False, threading=False)
    for post in stream:
        post_time = post['timestamp'].replace(tzinfo=None)
        time_dif = post_time - start_time
        time_to_now = datetime.utcnow() - post_time
        count_posts += 1
        if time_dif > report_freq:
            logging.info(str(post['timestamp']) + " Count: " + str(count_posts) + " Time Delta: " + str(time_to_now))
            start_time =post['timestamp'].replace(tzinfo=None)
            count_posts = 0

        if allowed_op_id(post['id']):
            if  (set(post['required_posting_auths']) & set(allowed_accounts)):
                output(post)

        if time_to_now < timedelta(seconds=2):
            logging.info('block_num: ' + str(post['block_num']))
            # Break out of the for loop we've caught up.
            break

    scan_time = datetime.utcnow() - scan_start_time
    logging.info('Finished catching up at block_num: ' + str(post['block_num']) + ' in '+ str(scan_time))


def main() -> None:
    """ Main file """
    """ scan_history will look back over the last 1 hour reporting every 15 minute chunk """
    scan_history(1, 15)
    """ scan_live will resume live scanning the chain and report every 5 minutes or when
        a notification arrives """
    scan_live(5)



if __name__ == "__main__":
    main()
