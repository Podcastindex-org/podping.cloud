import logging
import os
import queue
from beemgraphenebase.types import Bool
import socketserver
import zmq
import threading
import time
import json
from random import randint

from beem import Hive
from beem.account import Account
from beem.exceptions import AccountDoesNotExistsException, MissingKeyError
from beemapi.exceptions import UnhandledRPCError

# Testnet instead of main Hive
# BOL: Switching off TestNet, we should test on Hive for now.
USE_TEST_NODE = os.getenv("USE_TEST_NODE", 'False').lower() in ('true', '1', 't')
TEST_NODE = ['http://testnet.openhive.network:8091']

# This is a global signal to shut down until RC's recover
# Stores the RC cost of each operation to calculate an average
HALT_THE_QUEUE = False
HALT_TIME = [0,30,60,120,240,480,960]


logging.basicConfig(level=logging.INFO,
                    format=f'%(asctime)s %(levelname)s %(name)s %(threadName)s : %(message)s')


# ---------------------------------------------------------------
# BASIC SOCKETS
# ---------------------------------------------------------------
class MyTCPHandler(socketserver.BaseRequestHandler):
    """
    The RequestHandler class for our server.

    It is instantiated once per connection to the server, and must
    override the handle() method to implement communication to the
    client.
    """

    def handle(self):
        # self.request is the TCP socket connected to the client
        self.data = self.request.recv(1024).strip()
        url = self.data.decode("utf-8")
        logging.info("Received from {}: {}".format(self.client_address[0], url))
        trx_id, success = url_in(url)
        if success:
            self.request.sendall("OK".encode("utf-8"))
        else:
            self.request.sendall("ERR".encode("utf-8"))

def url_in(url):
    """ Send a URL and I'll post it to Hive """
    custom_json = {'url': url}
    trx_id , success = send_notification(custom_json)
    return trx_id, success




def get_allowed_accounts(acc_name='podping') -> bool:
    """ get a list of all accounts allowed to post by acc_name (podping)
        and only react to these accounts """
    # Ignores test node.
    h = Hive(node='https://api.hive.blog')
    master_account = Account(acc_name, blockchain_instance=h, lazy=True)
    allowed = master_account.get_following()
    return allowed


def send_notification(custom_json, operation_id ='podping'):
    """ Sends a custom_json to Hive
        Expects two env variables, Hive account name and posting key
        HIVE_SERVER_ACCOUNT
        HIVE_POSTING_KEY
        """
    try:
        # Artifically create errors <-----------------------------------
        # if operation_id == 'podping':
        #     r = randint(1,5)
        #     if r == 1:
        #         raise Exception('What a mess')
        tx = hive.custom_json(id=operation_id, json_data= custom_json,
                            required_posting_auths=[server_account])
        trx_id = tx['trx_id']

        logging.info(f'Transaction sent: {trx_id}')
        return trx_id, True

    except MissingKeyError:
        error_message = f'The provided key for @{server_account} is not valid'
        logging.error(error_message)
        return error_message, False
    except UnhandledRPCError:
        error_message = f'Most likely resource credit depletion'
        logging.error(error_message)
        HALT_THE_QUEUE = True
        trx_id = error_message
        return trx_id, False

    except Exception as ex:
        error_message = f'{ex} occurred {ex.__class__}'
        logging.error(error_message)
        trx_id = error_message
        return trx_id, False

#Adding a Queue system to the Hive send_notification section

# hive_q = queue.Queue()


# def send_notification_worker():
#     """ Opens and watches a queue and sends notifications to Hive one by one """
#     while True:
#         items = hive_q.get()
#         func = items[0]
#         args = items[1:]
#         start = time.perf_counter()
#         trx_id, success = func(*args)
#         # Limit the rate to 1 post every 2 seconds, this will mostly avoid
#         # multiple updates in a single Hive block.
#         duration = time.perf_counter() - start
#         if duration < 2.0:
#             time.sleep(2.0-duration)
#         hive_q.task_done()
#         logging.info(f'Task time: {duration:0.2f} - Queue size: ' + str(hive_q.qsize()))
#         logging.info(f'Finished a task: {trx_id} - {success}')


# ---------------------------------------------------------------
# START OF STARTUP SEQUENCE RUNNING IN GLOBAL SCOPE
# ---------------------------------------------------------------

# threading.Thread(target=send_notification_worker, daemon=True).start()


error_messages = []
# Set up Hive with error checking
logging.info('Podping startup sequence initiated, please stand by, full bozo checks in operation...')
server_account = os.getenv('HIVE_SERVER_ACCOUNT')
if not server_account:
    error_messages.append('No Hive account passed: HIVE_SERVER_ACCOUNT environment var must be set.')
    logging.error(error_messages[-1])
wif = [os.getenv('HIVE_POSTING_KEY')]
if not wif:
    error_messages.append('No Hive Posting Key passed: HIVE_POSTING_KEY environment var must be set.')
    logging.error(error_messages[-1])

try:
    if USE_TEST_NODE:
        hive = Hive(keys=wif,node=TEST_NODE)
    else:
        hive = Hive(keys=wif)

except Exception as ex:
    error_messages.append(f'{ex} occurred {ex.__class__}')
    error_messages.append(f'Can not connect to Hive, probably bad key')
    logging.error(error_messages[-1])
    error_messages.append("I'm sorry, Dave, I'm affraid I can't do that")
    logging.error(error_messages[-1])
    exit_message = ' - '.join(error_messages)
    raise SystemExit(exit_message)


acc = None
try:
    acc = Account(server_account, blockchain_instance=hive, lazy=True)
    allowed = get_allowed_accounts()
    if not server_account in allowed:
        error_messages.append(f'Account @{server_account} not authorised to send Podpings')
        logging.error(error_messages[-1])

except AccountDoesNotExistsException:
    error_messages.append(f'Hive account @{server_account} does not exist, check ENV vars and try again AccountDoesNotExistsException')
    logging.error(error_messages[-1])
except Exception as ex:
    error_messages.append(f'{ex} occurred {ex.__class__}')
    logging.error(error_messages[-1])

if acc:
    try:    # Now post two custom json to test.
        manabar = acc.get_rc_manabar()
        logging.info(f'Testing Account Resource Credits - before {manabar.get("current_pct"):.2f}%')
        custom_json = {
            "server_account" : server_account,
            "USE_TEST_NODE" : USE_TEST_NODE
        }
        error_message , success = send_notification(custom_json, 'podping-startup')

        if not success:
            error_messages.append(error_message)
        logging.info('Testing Account Resource Credits.... 5s')
        time.sleep(5)
        manabar_after = acc.get_rc_manabar()
        logging.info(f'Testing Account Resource Credits - after {manabar_after.get("current_pct"):.2f}%')
        cost = manabar.get('current_mana') - manabar_after.get('current_mana')
        capacity = manabar_after.get('current_mana') / cost
        logging.info(f'Capacity for further podpings : {capacity:.1f}')
        custom_json['capacity'] = f'{capacity:.1f}'
        custom_json['message'] = 'Podping startup complete'
        error_message , success = send_notification(custom_json, 'podping-startup')
        if not success:
            error_messages.append(error_message)

    except Exception as ex:
        error_messages.append(f'{ex} occurred {ex.__class__}')
        logging.error(error_messages[-1])


if error_messages:
    error_messages.append("I'm sorry, Dave, I'm affraid I can't do that")
    logging.error("Startup of Podping status: I'm sorry, Dave, I'm affraid I can't do that.")
    exit_message = ' - '.join(error_messages)
    if not USE_TEST_NODE:
        raise SystemExit(exit_message)


logging.info("Startup of Podping status: SUCCESS! Hit the BOOST Button.")

# ---------------------------------------------------------------
# END OF STARTUP SEQUENCE RUNNING IN GLOBAL SCOPE
# ---------------------------------------------------------------

# context = zmq.Context()
# socket = context.socket(zmq.REP)
# socket.bind("tcp://*:5555")


# def url_in(url):
#     """ Send a URL and I'll post it to Hive """
#     custom_json = {'url': url}
#     hive_q.put( (send_notification, custom_json ))

# Global used for tracking the number of failures we get in recursive retry
peak_fail_count = 0

def failure_retry(url, failure_count = 0):
    """ Recursion... see recursion """
    global peak_fail_count
    if failure_count > 0:
        logging.error(f"Waiting {HALT_TIME[failure_count]}")
        time.sleep(HALT_TIME[failure_count])
        logging.info(f"RETRYING url: {url}")
    else:
        logging.info(f"Received url: {url}")

    custom_json = {'url': url}
    trx_id, success = send_notification(custom_json)
    #  Send reply back to client
    answer ={
        'url':url,
        'trx_id':trx_id
    }
    if success:
        answer['message'] = 'success'
        if peak_fail_count > 0:
            answer['retries'] = peak_fail_count
        failure_count = 0
        peak_fail_count = 0
        return answer, failure_count
    else:
        failure_count += 1
        peak_fail_count += 1
        answer['message'] = 'failure - server will retry'
        if failure_count > len(HALT_TIME):
            # Give up.
            error_message = f"I'm sorry Dave, I'm affraid I can't do that. Too many tries {failure_count}"
            logging.error(error_message)
            raise SystemExit(error_message)
        answer, failure_count = failure_retry(url, failure_count)
        # Walk back up the recursion tree:
        return answer, failure_count



if __name__ == "__main__":
    HOST, PORT = "localhost", 9999

    # Create the server, binding to localhost on port 9999
    server = socketserver.TCPServer((HOST, PORT), MyTCPHandler)

    # Activate the server; this will keep running until you
    # interrupt the program with Ctrl-C
    server.serve_forever()




    # failure_count = 0
    # while True :
    #     #  Wait for next request from client
    #     url = socket.recv().decode('utf-8')
    #     answer, failure_count = failure_retry(url)
    #     ans = json.dumps(answer, indent=2)
    #     socket.send(ans.encode('utf-8'))