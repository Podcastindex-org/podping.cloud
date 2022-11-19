@0xe034fb00fceb07b4;

using import "podping_medium.capnp".PodpingMedium;
using import "podping_reason.capnp".PodpingReason;

struct Podping {
    medium @0 :PodpingMedium;
    reason @1 :PodpingReason;
    iris @2 :List(Text);
    timestampNs @3 :UInt64;
    sessionId @4 :UInt64;
}
