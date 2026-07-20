// Polyhymnia — C++ "Mathematical Engine / Randomizer".
//
// Its entire job is to pick one int64 out of a list of int64s. It does
// this with far more ceremony than necessary: hardware entropy via
// std::random_device, a manual bit-mixing step, and raw pointer
// arithmetic to walk to the chosen element — because overengineering
// is the point of this project.

#include <cstdint>
#include <iostream>
#include <memory>
#include <random>
#include <string>

#include <grpcpp/ext/proto_server_reflection_plugin.h>
#include <grpcpp/grpcpp.h>

#include "quote.grpc.pb.h"

using grpc::Server;
using grpc::ServerBuilder;
using grpc::ServerContext;
using grpc::Status;
using grpc::StatusCode;

using polyhymnia::IdList;
using polyhymnia::Randomizer;
using polyhymnia::SelectedId;

namespace {

constexpr char kListenAddress[] = "0.0.0.0:50052";

// Combines two independent std::random_device draws into a single 64-bit
// seed via bitwise shifting, then uses that seed to uniformly pick an
// index in [0, count). This is needlessly elaborate for picking one
// index, which is exactly the brief.
int64_t PickRandomIndex(std::size_t count) {
    std::random_device rd;

    const uint64_t hi = static_cast<uint64_t>(rd());
    const uint64_t lo = static_cast<uint64_t>(rd());
    const uint64_t seed = (hi << 32) | lo;

    std::mt19937_64 engine(seed);
    std::uniform_int_distribution<std::size_t> dist(0, count - 1);

    return static_cast<int64_t>(dist(engine));
}

} // namespace

class RandomizerServiceImpl final : public Randomizer::Service {
  public:
    Status SelectRandomId(ServerContext* /*context*/, const IdList* request,
                          SelectedId* response) override {
        const int size = request->ids_size();

        if (size <= 0) {
            return Status(StatusCode::INVALID_ARGUMENT,
                          "IdList must contain at least one id");
        }

        const int64_t index = PickRandomIndex(static_cast<std::size_t>(size));

        // Overengineering, as requested: walk to the chosen element with
        // raw pointer arithmetic instead of calling ids(index) directly.
        const int64_t* base = request->ids().data();
        const int64_t* target = base + index;
        const int64_t selected_id = *target;

        std::cout << "[cpp-engine] picked id=" << selected_id << " (index "
                  << index << " of " << size << " candidates)\n";

        response->set_id(selected_id);
        return Status::OK;
    }
};

void RunServer() {
    grpc::reflection::InitProtoReflectionServerBuilderPlugin();

    RandomizerServiceImpl service;

    ServerBuilder builder;
    builder.AddListeningPort(kListenAddress, grpc::InsecureServerCredentials());
    builder.RegisterService(&service);

    std::unique_ptr<Server> server(builder.BuildAndStart());
    std::cout << "[cpp-engine] Randomizer service listening on "
              << kListenAddress << std::endl;
    server->Wait();
}

int main() {
    RunServer();
    return 0;
}
