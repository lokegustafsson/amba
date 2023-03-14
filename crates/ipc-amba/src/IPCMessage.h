#pragma once
#include <zmq.hpp>
#include "message.pb.h"

struct IPCMessage {
    void send(zmq::socket_t& socket, const std::string& msg);
    void recv(zmq::socket_t& socket);
};
