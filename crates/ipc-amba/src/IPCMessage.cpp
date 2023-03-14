#include "IPCMessage.h"

void IPCMessage::send(zmq::socket_t& socket, const std::string& msg) {
    amba::IPCMessage message;
    message.set_content(msg);

    std::string data_to_send;
    message.SerializeToString(&data_to_send);

    socket.send(zmq::buffer(data_to_send), zmq::send_flags::none);
}

void IPCMessage::recv(zmq::socket_t& socket) {
    zmq::message_t reply{};
    socket.recv(reply, zmq::recv_flags::none);

    amba::IPCMessage message;
    message.ParseFromArray(reply.data(), reply.size());
    std::cout << "Received " << message.content() << std::endl;
}

