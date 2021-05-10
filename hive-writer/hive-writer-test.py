
#----- A simple TCP client program in Python using send() function -----

# Changed the range number for how many times you want to hit the server
import socket
import time

start = time.perf_counter()

for n in range(2):

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

print('Time taken: ' + str(time.perf_counter() - start) )