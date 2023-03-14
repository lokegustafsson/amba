cmake_minimum_required(VERSION 3.10)
add_subdirectory(thirdparty/cppzmq)

find_package(Protobuf REQUIRED)
include_directories(${Protobuf_INCLUDE_DIRS})
include_directories(${CMAKE_CURRENT_BINARY_DIR})
protobuf_generate_cpp(PROTO_SRCS PROTO_HDRS message.proto)

include_directories(${CMAKE_SOURCE_DIR}/src)

add_executable(main src/main.cpp src/IPCMessage.cpp src/IPCMessage.h ${PROTO_SRCS} ${PROTO_HDRS})
target_link_libraries(main cppzmq)
target_link_libraries(main ${Protobuf_LIBRARIES})

add_executable(client src/client.cpp src/IPCMessage.cpp src/IPCMessage.h ${PROTO_SRCS} ${PROTO_HDRS})
target_link_libraries(client cppzmq)
target_link_libraries(client ${Protobuf_LIBRARIES})
