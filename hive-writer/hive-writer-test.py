
#----- A simple TCP client program in Python using send() function -----

# Changed the range number for how many times you want to hit the server
import socket
import time


import zmq

context = zmq.Context()

#  Socket to talk to server
print("Connecting to hello world server…")
socket = context.socket(zmq.REQ)
socket.connect("tcp://localhost:5555")

start = time.perf_counter()
#  Do 10 requests, waiting each time for a response
for request in range(3):
    print(f"Sending request {request} …")
    data = f"https://www.brianoflondon.me/{request}/podcast2/brians-forest-talks-exp.xml"
    socket.send(data.encode())
    #  Get the reply.
    message = socket.recv()
    print("Received reply %s [ %s ]" % (request, message))


print('Time taken: ' + str(time.perf_counter() - start) )
