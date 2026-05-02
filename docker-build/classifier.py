import logging
import os
import socket
import struct

import birdnet
import numpy as np

logger = logging.getLogger(__name__)
logging.basicConfig(format='%(asctime)s \x1b[36;1m[PYTHON]\x1b[0m %(levelname)s: %(message)s', level=logging.DEBUG)


class BirdResult:
    def __init__(self, species_list):
        # species_list is a list of (name, confidence) tuples
        self.species_list = species_list

    def to_bytes(self):
        """
        convert a BirdResullt object to a bytestream for sending over the socket.
        """
        data = len(self.species_list).to_bytes(4, byteorder='little')
        for name, conf in self.species_list:
            name_bytes = name.encode('utf-8')
            data += len(name_bytes).to_bytes(4, byteorder='little')
            data += name_bytes
            data += struct.pack('<f', float(conf))
        return data


model = birdnet.load("acoustic", "2.4", "tf")
socket_path = "/tmp/birb_socket"

# remove an old socket in case there is a zombie from previous run.
try:
    os.unlink(socket_path)
except OSError:
    if os.path.exists(socket_path):
        raise

# create the socket.
server = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
server.bind(socket_path)
server.listen(1)

logger.info("Server is listening for incoming connections...")

# await connection
connection, client_address = server.accept()
logger.debug("Connection from: %s", str(connection).split(", ")[0][-4:])

# now the main loop where we listen for the socket.
while True:
    snippet = bytearray()
    # 48Khz * 3 seconds * 4 Bytes (f32)
    remaining = 144000 * 4

    # read until we have a full snippet.
    while remaining > 0:
        data = connection.recv(remaining)
        if not data:
            connection.close()
            os.unlink(socket_path)
            break

        remaining -= len(data)
        snippet.extend(data)

    # Convert the snippet to numpy data and smash it into the ML model.
    snippet = np.frombuffer(snippet, dtype=np.float32)
    predictions = model.predict_arrays((snippet, 48000))
    result = predictions.to_structured_array()
    result = result.view(np.recarray)
    # print if result is not empty (got a birb).
    if len(result) != 0:
        logger.debug("birb: %s | confidence: %s", result.species_name, result.confidence)

    # Encapsulate into our birb datatype
    species_list = list(zip(result.species_name, result.confidence))
    bird_result = BirdResult(species_list)

    # Convert to bytestream and send to rust.
    connection.sendall(bird_result.to_bytes())
