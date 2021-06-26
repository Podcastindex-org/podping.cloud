import json
import logging
from queue import Empty
import socketserver
import threading
import time
from random import randint
from sys import getsizeof

import zmq
from beem import Hive
from beem.account import Account
from beem.exceptions import AccountDoesNotExistsException, MissingKeyError
from beemapi.exceptions import UnhandledRPCError

from config import Config

logging.basicConfig(
    level=logging.INFO,
    format=f"%(asctime)s %(levelname)s %(name)s %(threadName)s : %(message)s",
)


def startup_sequence(ignore_errors=False, resource_test=True) -> bool:
    """Run though a startup sequence connect to Hive and check env variables
    Exit with error unless ignore_errors passed as True
    Defaults to sending two startup resource_test posts and checking resources"""
    global USE_TEST_NODE
    global hive
    error_messages = []
    # Set up Hive with error checking
    logging.info(
        "Podping startup sequence initiated, please stand by, full bozo checks in operation..."
    )
    if not Config.server_account:
        error_messages.append(
            "No Hive account passed: HIVE_SERVER_ACCOUNT environment var must be set."
        )
        logging.error(error_messages[-1])

    if not Config.wif:
        error_messages.append(
            "No Hive Posting Key passed: HIVE_POSTING_KEY environment var must be set."
        )
        logging.error(error_messages[-1])

    try:
        if USE_TEST_NODE:
            hive = Hive(keys=Config.wif, node=Config.TEST_NODE)
            logging.info("---------------> Using Test Node " + Config.TEST_NODE[0])
        else:
            hive = Hive(keys=Config.wif)
            hive.chain_params[
                "chain_id"
            ] = "beeab0de00000000000000000000000000000000000000000000000000000000"

            logging.info("---------------> Using Main Hive Chain ")

    except Exception as ex:
        error_messages.append(f"{ex} occurred {ex.__class__}")
        error_messages.append(f"Can not connect to Hive, probably bad key")
        logging.error(error_messages[-1])
        error_messages.append("I'm sorry, Dave, I'm afraid I can't do that")
        logging.error(error_messages[-1])
        exit_message = " - ".join(error_messages)
        raise SystemExit(exit_message)

    acc = None
    try:
        acc = Account(Config.server_account, blockchain_instance=hive, lazy=True)
        allowed = get_allowed_accounts()
        if not Config.server_account in allowed:
            error_messages.append(
                f"Account @{Config.server_account} not authorised to send Podpings"
            )
            logging.error(error_messages[-1])

    except AccountDoesNotExistsException:
        error_messages.append(
            f"Hive account @{Config.server_account} does not exist, check ENV vars and try again AccountDoesNotExistsException"
        )
        logging.error(error_messages[-1])
    except Exception as ex:
        error_messages.append(f"{ex} occurred {ex.__class__}")
        logging.error(error_messages[-1])

    if resource_test:
        if acc:
            try:  # Now post two custom json to test.
                manabar = acc.get_rc_manabar()
                logging.info(
                    f'Testing Account Resource Credits - before {manabar.get("current_pct"):.2f}%'
                )
                custom_json = {
                    "server_account": Config.server_account,
                    "USE_TEST_NODE": Config.test,
                    "message": "Podping startup initiated",
                }
                error_message, success = send_notification(
                    custom_json, "podping-startup"
                )

                if not success:
                    error_messages.append(error_message)
                logging.info("Testing Account Resource Credits.... 5s")
                time.sleep(2)
                manabar_after = acc.get_rc_manabar()
                logging.info(
                    f'Testing Account Resource Credits - after {manabar_after.get("current_pct"):.2f}%'
                )
                cost = manabar.get("current_mana") - manabar_after.get("current_mana")
                if cost == 0:  # skip this test if we're going to get ZeroDivision
                    capacity = 1000000
                else:
                    capacity = manabar_after.get("current_mana") / cost
                logging.info(f"Capacity for further podpings : {capacity:.1f}")
                custom_json["v"] = Config.CURRENT_PODPING_VERSION
                custom_json["capacity"] = f"{capacity:.1f}"
                custom_json["message"] = "Podping startup complete"
                error_message, success = send_notification(
                    custom_json, "podping-startup"
                )
                if not success:
                    error_messages.append(error_message)

            except Exception as ex:
                error_messages.append(f"{ex} occurred {ex.__class__}")
                logging.error(error_messages[-1])

    if error_messages:
        error_messages.append("I'm sorry, Dave, I'm afraid I can't do that")
        logging.error(
            "Startup of Podping status: I'm sorry, Dave, I'm afraid I can't do that."
        )
        exit_message = " - ".join(error_messages)
        if (not USE_TEST_NODE) or ignore_errors:
            raise SystemExit(exit_message)

    logging.info("Startup of Podping status: SUCCESS! Hit the BOOST Button.")
    return True

    # ---------------------------------------------------------------
    # END OF STARTUP SEQUENCE
    # ---------------------------------------------------------------


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
        if not success:
            logging.error(f"Result: {trx_id}")
        if success:
            self.request.sendall("OK".encode("utf-8"))
        else:
            self.request.sendall("ERR".encode("utf-8"))


def url_in(url):
    """Send a URL and I'll post it to Hive"""
    Config.url_q.put(url)
    return "Sent", True


def get_allowed_accounts(acc_name="podping") -> bool:
    """get a list of all accounts allowed to post by acc_name (podping)
    and only react to these accounts"""
    # Ignores test node.
    h = Hive(node="https://api.hive.blog")
    master_account = Account(acc_name, blockchain_instance=h, lazy=True)
    allowed = master_account.get_following()
    return allowed


def send_notification(data, operation_id="podping"):
    """Sends a custom_json to Hive
    Expects two env variables, Hive account name and posting key
    HIVE_SERVER_ACCOUNT
    HIVE_POSTING_KEY
    """
    num_urls = 0

    if type(data) == set:
        num_urls = len(data)
        size_of_urls = len("".join(data))
        custom_json = {
            "v": Config.CURRENT_PODPING_VERSION,
            "num_urls": num_urls,
            "r": Config.NOTIFICATION_REASONS["feed_update"],
            "urls": list(data),
        }
    elif type(data) == str:
        num_urls = 1
        size_of_urls = len(data)
        custom_json = {
            "v": Config.CURRENT_PODPING_VERSION,
            "num_urls": 1,
            "r": Config.NOTIFICATION_REASONS["feed_update"],
            "url": data,
        }
    elif type(data) == dict:
        size_of_urls = getsizeof(data)
        custom_json = data
    else:
        logging.error(f"Unknown data type: {data}")

    try:
        # Artificially create errors <-----------------------------------
        if operation_id == "podping" and Config.errors:
            r = randint(1, 100)
            if r <= Config.errors:
                raise Exception(
                    f"Infinite Improbability Error level of {r}% : Threshold set at {Config.errors}%"
                )

        # Assert Exception:o.json.length() <= HIVE_CUSTOM_OP_DATA_MAX_LENGTH: Operation JSON must be less than 8192 bytes.
        size_of_json = len(json.dumps(custom_json))
        tx = hive.custom_json(
            id=operation_id,
            json_data=custom_json,
            required_posting_auths=[Config.server_account],
        )
        trx_id = tx["trx_id"]
        logging.info(
            f"Transaction sent: {trx_id} - Num urls: {num_urls} - Size of Urls: {size_of_urls} - Json size: {size_of_json}"
        )
        logging.info(f"Overhead: {size_of_json - size_of_urls}")
        return trx_id, True

    except MissingKeyError:
        error_message = f"The provided key for @{Config.server_account} is not valid "
        logging.error(error_message)
        return error_message, False
    except UnhandledRPCError as ex:
        error_message = f"{ex} occurred: {ex.__class__}"
        logging.error(error_message)
        HALT_THE_QUEUE = True
        trx_id = error_message
        return trx_id, False

    except Exception as ex:
        error_message = f"{ex} occurred {ex.__class__}"
        logging.error(error_message)
        trx_id = error_message
        return trx_id, False


def send_notification_worker():
    """Opens and watches a queue and sends notifications to Hive one by one"""
    while True:
        items = Config.hive_q.get()
        func = items[0]
        args = items[1:]
        start = time.perf_counter()
        trx_id, success = func(*args)
        # Limit the rate to 1 post every 2 seconds, this will mostly avoid
        # multiple updates in a single Hive block.
        duration = time.perf_counter() - start
        # if duration < 2.0:
        #     time.sleep(2.0-duration)
        Config.hive_q.task_done()
        logging.info(
            f"Task time: {duration:0.2f} - Queue size: " + str(Config.hive_q.qsize())
        )
        logging.info(f'Finished a task: {trx_id["trx_id"]} - {success}')


def url_q_worker():
    while True:
        url_set = set()
        start = time.perf_counter()
        duration = 0
        url_set_bytes = 0
        while (duration < Config.HIVE_OPERATION_PERIOD) and (
            url_set_bytes < Config.MAX_URL_LIST_BYTES
        ):
            #  get next URL from Q
            logging.info(f"Duration: {duration:.3f} - WAITING - Queue: {len(url_set)}")
            try:
                url = Config.url_q.get(timeout=Config.HIVE_OPERATION_PERIOD)
            except Empty:
                break
            except Exception as ex:
                logging.error(f"{ex} occurred")
            url_set.add(url)
            duration = time.perf_counter() - start
            logging.info(
                f"Duration: {duration:.3f} - URL in queue: {url} - URL List: {len(url_set)}"
            )
            Config.url_q.task_done()
            url_set_bytes = len("".join(url_set))
        if len(url_set):
            Config.hive_q.put((failure_retry, url_set))
            logging.info(f"Size of Urls: {url_set_bytes}")


# def url_in(url):
#     """ Send a URL and I'll post it to Hive """
#     custom_json = {'url': url}
#     hive_q.put( (send_notification, custom_json ))

# Global used for tracking the number of failures we get in recursive retry
peak_fail_count = 0


def failure_retry(url_set, failure_count=0):
    """Recursion... see recursion"""
    global peak_fail_count
    if failure_count > 0:
        logging.error(f"Waiting {Config.HALT_TIME[failure_count]}s")
        time.sleep(Config.HALT_TIME[failure_count])
        logging.info(f"RETRYING num_urls: {len(url_set)}")
    else:
        if type(url_set) == set:
            logging.info(f"Received num_urls: {len(url_set)}")
        elif type(url_set) == str:
            logging.info(f"One URL Received: {url_set}")
        else:
            logging.info(f"{url_set}")

    trx_id, success = send_notification(url_set)
    #  Send reply back to client
    answer = {"url": url_set, "trx_id": trx_id}
    if success:
        answer["message"] = "success"
        if peak_fail_count > 0:
            answer["retries"] = peak_fail_count
        failure_count = 0
        peak_fail_count = 0
        return answer, failure_count
    else:
        failure_count += 1
        peak_fail_count += 1
        answer["message"] = "failure - server will retry"
        if failure_count >= len(Config.HALT_TIME):
            # Give up.
            error_message = f"I'm sorry Dave, I'm afraid I can't do that. Too many tries {failure_count}"
            logging.error(error_message)
            raise SystemExit(error_message)
        answer, failure_count = failure_retry(url_set, failure_count)
        # Walk back up the recursion tree:
        return answer, failure_count


# Adding a Queue system to the Hive send_notification section
threading.Thread(target=send_notification_worker, daemon=True).start()

# Adding a Queue system for holding URLs and sending them out
threading.Thread(target=url_q_worker, daemon=True).start()


def main() -> None:
    """Main man what counts..."""
    Config.setup()
    global USE_TEST_NODE
    if Config.test:
        USE_TEST_NODE = True
    else:
        USE_TEST_NODE = False

    if Config.url:
        url = Config.url
        if startup_sequence(resource_test=False):
            answer, failure_count = failure_retry(url)
            return
        else:
            raise (SystemExit)
        return

    startup_sequence(resource_test=True)
    if Config.socket:
        HOST, PORT = "localhost", Config.socket
        # Create the server, binding to localhost on port 9999
        server = socketserver.TCPServer((HOST, PORT), MyTCPHandler)
        # Activate the server; this will keep running until you
        # interrupt the program with Ctrl-C
        server.serve_forever()
    elif Config.zmq:
        context = zmq.Context()
        socket = context.socket(zmq.REP)
        socket.bind(f"tcp://*:{Config.zmq}")
        while True:
            url = socket.recv().decode("utf-8")
            Config.url_q.put(url)
            ans = "OK"
            socket.send(ans.encode("utf-8"))

    else:
        logging.error(
            "You've got to specify --socket or --zmq otherwise I can't listen!"
        )


if __name__ == "__main__":
    """Hit it!"""
    main()
