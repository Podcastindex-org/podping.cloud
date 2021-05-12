
#----- A simple TCP client program in Python using send() function -----

# Changed the range number for how many times you want to hit the server
import socket
import time
import json


import zmq

# context = zmq.Context()

#  Socket to talk to server
# print("Connecting to hello world server…")
# socket = context.socket(zmq.REQ)
# socket.connect("tcp://192.168.0.245:5555")

def old_socket():

    for n in range(10):
            # Create a client socket
        clientSocket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        # Connect to the server
        clientSocket.connect(("127.0.0.1",9999))
        # Send data to server
        data = f"https://www.brianoflondon.me/{n}/podcast2/brians-forest-talks-exp.xml"
        clientSocket.send(data.encode())
        # Receive data from server
        dataFromServer = clientSocket.recv(1024)
        # Print to the console
        print(dataFromServer.decode())

    clientSocket.close()




def loop_test():
    """ Run a simple loop test on the hive-writer program """
    start = time.perf_counter()
    #  Do 10 requests, waiting each time for a response
    for request in range(2):
        print(f"Sending request {request} …")
        data = f"https://www.brianoflondon.me/podcast2/brians-forest-talks-exp.xml?q={request}"
        socket.send(data.encode())
        #  Get the reply.
        message = socket.recv()
        print("Received reply %s [ %s ]" % (request, message))


    print('Time taken: ' + str(time.perf_counter() - start) )


def old_data(start_line=0):
    """ Run through old data and repeat it every few seconds """
    urls = []
    line_num = 0
    with open('/Users/gbishko/Documents/Python-iMac/PodcastIndex/podping.cloud/hive-writer/24hours.log') as f:
        while start_line > 0:
            line = f.readline()
            start_line -= 1
            line_num +=1
        while line:
            data = line.split(' - ')
            url = data[5].rstrip()
            start = time.perf_counter()
            socket.send(url.encode())
            message = socket.recv_json()
            print('Time taken: ' + str(time.perf_counter() - start) )
            print("Received reply: " + json.dumps(message,indent=2))
            time.sleep(30)
            line = f.readline()
            line_num +=1
            print(line_num)



if __name__ == "__main__":
    # old_data(720)
    # loop_test()
    old_socket()
