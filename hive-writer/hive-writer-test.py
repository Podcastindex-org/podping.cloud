
#----- A simple TCP client program in Python using send() function -----

import socket









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