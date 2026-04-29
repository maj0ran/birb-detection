import os
import socket
import struct

import birdnet
import numpy as np


class BirdResult:
    def __init__(self, species_list):
        # species_list is a list of (name, confidence) tuples
        self.species_list = species_list

    def to_bytes(self):
        data = len(self.species_list).to_bytes(4, byteorder='little')
        for name, conf in self.species_list:
            name_bytes = name.encode('utf-8')
            data += len(name_bytes).to_bytes(4, byteorder='little')
            data += name_bytes
            data += struct.pack('<f', float(conf))
        return data


model = birdnet.load("acoustic", "2.4", "tf")

socket_path = "/tmp/birb_socket"
try:
    os.unlink(socket_path)
except OSError:
    if os.path.exists(socket_path):
        raise

server = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
server.bind(socket_path)
server.listen(1)

print("Server is listening for incoming connections...")
connection, client_address = server.accept()

print("Connection from", str(connection).split(", ")[0][-4:])
while True:
    snippet = bytearray()
    remaining = 144000 * 4
    while remaining > 0:
        data = connection.recv(remaining)
        if not data:
            break
        remaining -= len(data)
        snippet.extend(data)

    snippet = np.frombuffer(snippet, dtype=np.float32)
    predictions = model.predict_arrays((snippet, 48000))

    result = predictions.to_structured_array()
    result = result.view(np.recarray)

    print("birb:", result.species_name)
    print("confidence:", result.confidence)

    # Encapsulate into custom datatype
    species_list = list(zip(result.species_name, result.confidence))
    bird_result = BirdResult(species_list)

    # Convert to bytestream and send
    connection.sendall(bird_result.to_bytes())

#   close the connection
connection.close()
#  remove the socket file
os.unlink(socket_path)
