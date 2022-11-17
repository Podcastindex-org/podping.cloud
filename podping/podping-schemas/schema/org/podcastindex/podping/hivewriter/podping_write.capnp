@0x9b917b91f85f5cc2;

using import "podping_medium.capnp".PodpingMedium;
using import "podping_reason.capnp".PodpingReason;

struct PodpingWrite {
    medium @0 :PodpingMedium;
    reason @1 :PodpingReason;
    iri @2 :Text;
}
