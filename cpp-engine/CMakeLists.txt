cmake_minimum_required(VERSION 3.16)
project(cpp-engine CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

if(NOT CMAKE_BUILD_TYPE)
    set(CMAKE_BUILD_TYPE Release)
endif()

find_package(Threads REQUIRED)

# --- Locate Protobuf + gRPC -------------------------------------------
# Homebrew / vcpkg / from-source installs ship proper CMake config
# packages (ProtobufConfig.cmake, gRPCConfig.cmake). Distro packages
# from apt/dnf usually do NOT ship these and only provide pkg-config
# .pc files instead, so we try CONFIG mode first and fall back to
# pkg-config if that fails.

find_package(Protobuf CONFIG QUIET)
find_package(gRPC CONFIG QUIET)

if(Protobuf_FOUND AND gRPC_FOUND)
    message(STATUS "Using CMake config packages for Protobuf/gRPC")

    set(PROTOC_EXE protobuf::protoc)
    get_target_property(GRPC_CPP_PLUGIN_EXE gRPC::grpc_cpp_plugin LOCATION)
    set(POLYHYMNIA_GRPC_LIBS gRPC::grpc++ gRPC::grpc++_reflection)
    set(POLYHYMNIA_PROTOBUF_LIBS protobuf::libprotobuf)
    set(POLYHYMNIA_INCLUDE_DIRS "")
else()
    message(STATUS "CMake config packages not found — falling back to pkg-config "
                    "(typical for apt/dnf-installed gRPC on Linux)")

    find_package(PkgConfig REQUIRED)
    pkg_check_modules(GRPC REQUIRED IMPORTED_TARGET grpc++)
    pkg_check_modules(PROTOBUF REQUIRED IMPORTED_TARGET protobuf)

    find_program(PROTOC_BIN protoc REQUIRED)
    find_program(GRPC_CPP_PLUGIN_EXE grpc_cpp_plugin REQUIRED)
    find_library(GRPC_CPP_REFLECTION_LIB grpc++_reflection REQUIRED)

    set(PROTOC_EXE ${PROTOC_BIN})
    set(POLYHYMNIA_GRPC_LIBS PkgConfig::GRPC ${GRPC_CPP_REFLECTION_LIB})
    set(POLYHYMNIA_PROTOBUF_LIBS PkgConfig::PROTOBUF)
    set(POLYHYMNIA_INCLUDE_DIRS ${GRPC_INCLUDE_DIRS} ${PROTOBUF_INCLUDE_DIRS})
endif()

# --- Codegen -------------------------------------------------------------

set(PROTO_FILE "${CMAKE_SOURCE_DIR}/../proto/quote.proto")
set(GENERATED_DIR "${CMAKE_BINARY_DIR}/generated")
file(MAKE_DIRECTORY "${GENERATED_DIR}")

set(PROTO_SRCS "${GENERATED_DIR}/quote.pb.cc")
set(PROTO_HDRS "${GENERATED_DIR}/quote.pb.h")
set(GRPC_SRCS  "${GENERATED_DIR}/quote.grpc.pb.cc")
set(GRPC_HDRS  "${GENERATED_DIR}/quote.grpc.pb.h")

add_custom_command(
    OUTPUT ${PROTO_SRCS} ${PROTO_HDRS} ${GRPC_SRCS} ${GRPC_HDRS}
    COMMAND ${PROTOC_EXE}
    ARGS --cpp_out="${GENERATED_DIR}"
         --grpc_out="${GENERATED_DIR}"
         --plugin=protoc-gen-grpc="${GRPC_CPP_PLUGIN_EXE}"
         -I "${CMAKE_SOURCE_DIR}/../proto"
         "${PROTO_FILE}"
    DEPENDS "${PROTO_FILE}"
    COMMENT "Generating C++ protobuf/gRPC code from quote.proto"
)

add_executable(cpp-engine
    src/main.cpp
    ${PROTO_SRCS}
    ${GRPC_SRCS}
)

target_include_directories(cpp-engine PRIVATE "${GENERATED_DIR}" ${POLYHYMNIA_INCLUDE_DIRS})

target_link_libraries(cpp-engine
    PRIVATE
        ${POLYHYMNIA_GRPC_LIBS}
        ${POLYHYMNIA_PROTOBUF_LIBS}
        Threads::Threads
)

target_compile_options(cpp-engine PRIVATE -Wall -Wextra -Wpedantic)
