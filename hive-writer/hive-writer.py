import logging
import socket
from beem import Hive
import os
import threading
import socketserver

# Testnet instead of main Hive
# BOL: Switching off TestNet, we should test on Hive for now.
USE_TEST_NODE = False
TEST_NODE = ['http://testnet.openhive.network:8091']

logging.basicConfig(level=logging.INFO,
                    format=f'%(asctime)s %(levelname)s %(name)s %(threadName)s : %(message)s')

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
        print("{}: {}".format(self.client_address[0], url))
        url_in(url)
        self.request.sendall("OK".encode("utf-8"))


def url_in(url):
    """ Send a URL and I'll post it to Hive """
    custom_json = {'url': url}
    trx_id, success = send_notification(custom_json=custom_json)
    custom_json['trx_id'] = trx_id
    custom_json['success'] = success
    #emit('response', {'data': custom_json})


def send_notification(custom_json, server_account='', wif=''):
    """ Sends a custom_json to Hive
        Expects two env variables, Hive account name and posting key
        HIVE_SERVER_ACCOUNT
        HIVE_POSTING_KEY
        """

    operation_id = 'hive-hydra'

    try:
        if server_account == '':
            server_account = os.getenv('HIVE_SERVER_ACCOUNT')
            pass
        if wif == '':
            wif = [os.getenv('HIVE_POSTING_KEY')]
            pass

        if USE_TEST_NODE:
            h = Hive(keys=wif,node=TEST_NODE)
        else:
            h = Hive(keys=wif)


        tx = h.custom_json(id=operation_id, json_data= custom_json,
                            required_posting_auths=[server_account])

        trx_id = tx['trx_id']
        logging.info(f'Transaction sent: {trx_id}')
        return trx_id, True

    except Exception as ex:
        error_message = f'{ex} occurred {ex.__class__}'
        logging.error(error_message)
        trx_id = error_message
        return trx_id, False


if __name__ == "__main__":
    HOST, PORT = "localhost", 5001

    # Create the server, binding to localhost on port 9999
    server = socketserver.TCPServer((HOST, PORT), MyTCPHandler)

    # Activate the server; this will keep running until you
    # interrupt the program with Ctrl-C
    server.serve_forever()



# if __name__ == '__main__':
#     main()
