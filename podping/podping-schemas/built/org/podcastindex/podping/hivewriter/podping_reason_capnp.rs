// @generated by the capnpc-rust plugin to the Cap'n Proto schema compiler.
// DO NOT EDIT.
// source: org/podcastindex/podping/hivewriter/podping_reason.capnp


#[repr(u16)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PodpingReason {
  Update = 0,
  Live = 1,
  LiveEnd = 2,
  NewIRI = 3,
}
impl ::capnp::traits::FromU16 for PodpingReason {
  #[inline]
  fn from_u16(value: u16) -> ::core::result::Result<PodpingReason, ::capnp::NotInSchema> {
    match value {
      0 => ::core::result::Result::Ok(PodpingReason::Update),
      1 => ::core::result::Result::Ok(PodpingReason::Live),
      2 => ::core::result::Result::Ok(PodpingReason::LiveEnd),
      3 => ::core::result::Result::Ok(PodpingReason::NewIRI),
      n => ::core::result::Result::Err(::capnp::NotInSchema(n)),
    }
  }
}
impl ::capnp::traits::ToU16 for PodpingReason {
  #[inline]
  fn to_u16(self) -> u16 { self as u16 }
}
impl ::capnp::traits::HasTypeId for PodpingReason {
  #[inline]
  fn type_id() -> u64 { 0xd99a_1fdf_acec_bc89u64 }
}
