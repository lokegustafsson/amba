#include <string>
#include <iostream>
#include <zmq.hpp>
#include "IPCMessage.h"

int main()
{
    // initialize the zmq context with a single IO thread
    zmq::context_t context{1};

    // construct a REQ (request) socket and connect to interface
    zmq::socket_t socket{context, zmq::socket_type::req};
    socket.connect("tcp://localhost:5555");

    // set up some static data to send
    const std::string data{"Hello"};

    IPCMessage ipcm;
    ipcm.send(socket, data);
    ipcm.recv(socket);
    return 0;
}
