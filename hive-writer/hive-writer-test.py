# ----- A simple TCP client program in Python using send() function -----

# Changed the range number for how many times you want to hit the server
import json
import socket
import time
import uuid
from random import choice, randint, random
import zmq

context = zmq.Context()

# Socket to talk to server
print("Connecting to hello world server…")
zsocket = context.socket(zmq.REQ)
zsocket.connect("tcp://localhost:9999")


def old_socket():

    for n in range(20):
        # Create a client socket
        clientSocket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        # Connect to the server
        clientSocket.connect(("127.0.0.1", 9999))
        # Send data to server
        data = (
            f"https://www.brianoflondon.me/podcast2/brians-forest-talks-exp.xml?q={n}"
        )
        clientSocket.send(data.encode())
        # Receive data from server
        dataFromServer = clientSocket.recv(1024)
        # Print to the console
        print(dataFromServer.decode())

    clientSocket.close()


def loop_test():
    """Run a simple loop test on the hive-writer program"""
    start = time.perf_counter()
    #  Do 10 requests, waiting each time for a response
    for request in range(7):
        print(f"Sending request {request} …")
        data = f"https://www.brianoflondon.me/podcast2/brians-forest-talks-exp.xml?q={request}"
        zsocket.send(data.encode())
        #  Get the reply.
        message = zsocket.recv()
        print("Received reply %s [ %s ]" % (request, message))

    print("Time taken: " + str(time.perf_counter() - start))


def test_data(run_time: int = 0):
    """Use a generated unique URL"""
    n = 0
    test_start = time.perf_counter()
    while time.perf_counter() - test_start < (run_time * 60):
        burst = 0
        max_send = randint(4, 15) + choice(
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 20, 30]
        )
        while burst < max_send:
            url = f"https://example.com?n={n}&u={uuid.uuid4()}"
            start = time.perf_counter()
            zsocket.send(url.encode())
            message = zsocket.recv().decode()
            duration = time.perf_counter() - start
            print(f"{n:6} - Time taken: {duration}")
            burst += 1
            n += 1
            # time.sleep(random() / 2)
        # time.sleep(randint(2,6))




def old_data(start_line=0):
    """Run through old data and repeat it every few seconds"""
    urls = []
    line_num = 0
    with open(
        "/Users/gbishko/Documents/Python-iMac/PodcastIndex/podping.cloud/hive-writer/24hours.log"
    ) as f:
        while line_num <= start_line:
            line = f.readline()
            line_num += 1
        while line:
            burst = 0
            max_send = randint(4, 15) + choice(
                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 20, 30]
            )
            while burst < max_send and line:
                data = line.split(" - ")
                url = data[5].rstrip()
                start = time.perf_counter()
                zsocket.send(url.encode())
                message = zsocket.recv().decode()
                print("Time taken: " + str(time.perf_counter() - start))
                # print("Received reply: " + json.dumps(message,indent=2))
                print(f"Received Reply: {message}")
                time.sleep(random() / 5)
                line = f.readline()
                line_num += 1
                print(line_num)
                burst += 1
            time.sleep(randint(0, 4))


if __name__ == "__main__":
    test_data(3)
    # for _ in range(5):
    #     old_data(0)
    # loop_test()
    # old_socket()
