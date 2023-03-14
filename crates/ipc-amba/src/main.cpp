#include <string>
#include <iostream>
#include <zmq.hpp>
#include "IPCMessage.h"

int main() 
{
    // initialize the zmq context with a single IO thread
    zmq::context_t context{1};

    // construct a REP (reply) socket and bind to interface
    zmq::socket_t socket{context, zmq::socket_type::rep};
    socket.bind("tcp://*:5555");

    // prepare some static data for responses
    const std::string data{"World"};

    IPCMessage ipcm;
    for (;;) {
        ipcm.recv(socket);
        ipcm.send(socket, data);
    }
    return 0;
}
