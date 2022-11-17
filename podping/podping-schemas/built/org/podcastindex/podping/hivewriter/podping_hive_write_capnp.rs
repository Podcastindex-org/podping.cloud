// @generated by the capnpc-rust plugin to the Cap'n Proto schema compiler.
// DO NOT EDIT.
// source: org/podcastindex/podping/hivewriter/podping_hive_write.capnp


pub mod podping_hive_write {
  #[derive(Copy, Clone)]
  pub struct Owned(());
  impl <'a> ::capnp::traits::Owned<'a> for Owned { type Reader = Reader<'a>; type Builder = Builder<'a>; }
  impl <'a> ::capnp::traits::OwnedStruct<'a> for Owned { type Reader = Reader<'a>; type Builder = Builder<'a>; }
  impl ::capnp::traits::Pipelined for Owned { type Pipeline = Pipeline; }

  #[derive(Clone, Copy)]
  pub struct Reader<'a> { reader: ::capnp::private::layout::StructReader<'a> }

  impl <'a,> ::capnp::traits::HasTypeId for Reader<'a,>  {
    #[inline]
    fn type_id() -> u64 { _private::TYPE_ID }
  }
  impl <'a,> ::capnp::traits::FromStructReader<'a> for Reader<'a,>  {
    fn new(reader: ::capnp::private::layout::StructReader<'a>) -> Reader<'a,> {
      Reader { reader,  }
    }
  }

  impl <'a,> ::capnp::traits::FromPointerReader<'a> for Reader<'a,>  {
    fn get_from_pointer(reader: &::capnp::private::layout::PointerReader<'a>, default: ::core::option::Option<&'a [capnp::Word]>) -> ::capnp::Result<Reader<'a,>> {
      ::core::result::Result::Ok(::capnp::traits::FromStructReader::new(reader.get_struct(default)?))
    }
  }

  impl <'a,> ::capnp::traits::IntoInternalStructReader<'a> for Reader<'a,>  {
    fn into_internal_struct_reader(self) -> ::capnp::private::layout::StructReader<'a> {
      self.reader
    }
  }

  impl <'a,> ::capnp::traits::Imbue<'a> for Reader<'a,>  {
    fn imbue(&mut self, cap_table: &'a ::capnp::private::layout::CapTable) {
      self.reader.imbue(::capnp::private::layout::CapTableReader::Plain(cap_table))
    }
  }

  impl <'a,> Reader<'a,>  {
    pub fn reborrow(&self) -> Reader<'_,> {
      Reader { .. *self }
    }

    pub fn total_size(&self) -> ::capnp::Result<::capnp::MessageSize> {
      self.reader.total_size()
    }
    #[inline]
    pub fn get_podping_write(self) -> ::capnp::Result<crate::podping_write_capnp::podping_write::Reader<'a>> {
      ::capnp::traits::FromPointerReader::get_from_pointer(&self.reader.get_pointer_field(0), ::core::option::Option::None)
    }
    #[inline]
    pub fn has_podping_write(&self) -> bool {
      !self.reader.get_pointer_field(0).is_null()
    }
    #[inline]
    pub fn get_hive_account(self) -> ::capnp::Result<::capnp::text::Reader<'a>> {
      ::capnp::traits::FromPointerReader::get_from_pointer(&self.reader.get_pointer_field(1), ::core::option::Option::None)
    }
    #[inline]
    pub fn has_hive_account(&self) -> bool {
      !self.reader.get_pointer_field(1).is_null()
    }
  }

  pub struct Builder<'a> { builder: ::capnp::private::layout::StructBuilder<'a> }
  impl <'a,> ::capnp::traits::HasStructSize for Builder<'a,>  {
    #[inline]
    fn struct_size() -> ::capnp::private::layout::StructSize { _private::STRUCT_SIZE }
  }
  impl <'a,> ::capnp::traits::HasTypeId for Builder<'a,>  {
    #[inline]
    fn type_id() -> u64 { _private::TYPE_ID }
  }
  impl <'a,> ::capnp::traits::FromStructBuilder<'a> for Builder<'a,>  {
    fn new(builder: ::capnp::private::layout::StructBuilder<'a>) -> Builder<'a, > {
      Builder { builder,  }
    }
  }

  impl <'a,> ::capnp::traits::ImbueMut<'a> for Builder<'a,>  {
    fn imbue_mut(&mut self, cap_table: &'a mut ::capnp::private::layout::CapTable) {
      self.builder.imbue(::capnp::private::layout::CapTableBuilder::Plain(cap_table))
    }
  }

  impl <'a,> ::capnp::traits::FromPointerBuilder<'a> for Builder<'a,>  {
    fn init_pointer(builder: ::capnp::private::layout::PointerBuilder<'a>, _size: u32) -> Builder<'a,> {
      ::capnp::traits::FromStructBuilder::new(builder.init_struct(_private::STRUCT_SIZE))
    }
    fn get_from_pointer(builder: ::capnp::private::layout::PointerBuilder<'a>, default: ::core::option::Option<&'a [capnp::Word]>) -> ::capnp::Result<Builder<'a,>> {
      ::core::result::Result::Ok(::capnp::traits::FromStructBuilder::new(builder.get_struct(_private::STRUCT_SIZE, default)?))
    }
  }

  impl <'a,> ::capnp::traits::SetPointerBuilder for Reader<'a,>  {
    fn set_pointer_builder<'b>(pointer: ::capnp::private::layout::PointerBuilder<'b>, value: Reader<'a,>, canonicalize: bool) -> ::capnp::Result<()> { pointer.set_struct(&value.reader, canonicalize) }
  }

  impl <'a,> Builder<'a,>  {
    pub fn into_reader(self) -> Reader<'a,> {
      ::capnp::traits::FromStructReader::new(self.builder.into_reader())
    }
    pub fn reborrow(&mut self) -> Builder<'_,> {
      Builder { .. *self }
    }
    pub fn reborrow_as_reader(&self) -> Reader<'_,> {
      ::capnp::traits::FromStructReader::new(self.builder.into_reader())
    }

    pub fn total_size(&self) -> ::capnp::Result<::capnp::MessageSize> {
      self.builder.into_reader().total_size()
    }
    #[inline]
    pub fn get_podping_write(self) -> ::capnp::Result<crate::podping_write_capnp::podping_write::Builder<'a>> {
      ::capnp::traits::FromPointerBuilder::get_from_pointer(self.builder.get_pointer_field(0), ::core::option::Option::None)
    }
    #[inline]
    pub fn set_podping_write(&mut self, value: crate::podping_write_capnp::podping_write::Reader<'_>) -> ::capnp::Result<()> {
      ::capnp::traits::SetPointerBuilder::set_pointer_builder(self.builder.get_pointer_field(0), value, false)
    }
    #[inline]
    pub fn init_podping_write(self, ) -> crate::podping_write_capnp::podping_write::Builder<'a> {
      ::capnp::traits::FromPointerBuilder::init_pointer(self.builder.get_pointer_field(0), 0)
    }
    #[inline]
    pub fn has_podping_write(&self) -> bool {
      !self.builder.get_pointer_field(0).is_null()
    }
    #[inline]
    pub fn get_hive_account(self) -> ::capnp::Result<::capnp::text::Builder<'a>> {
      ::capnp::traits::FromPointerBuilder::get_from_pointer(self.builder.get_pointer_field(1), ::core::option::Option::None)
    }
    #[inline]
    pub fn set_hive_account(&mut self, value: ::capnp::text::Reader<'_>)  {
      self.builder.get_pointer_field(1).set_text(value);
    }
    #[inline]
    pub fn init_hive_account(self, size: u32) -> ::capnp::text::Builder<'a> {
      self.builder.get_pointer_field(1).init_text(size)
    }
    #[inline]
    pub fn has_hive_account(&self) -> bool {
      !self.builder.get_pointer_field(1).is_null()
    }
  }

  pub struct Pipeline { _typeless: ::capnp::any_pointer::Pipeline }
  impl ::capnp::capability::FromTypelessPipeline for Pipeline {
    fn new(typeless: ::capnp::any_pointer::Pipeline) -> Pipeline {
      Pipeline { _typeless: typeless,  }
    }
  }
  impl Pipeline  {
    pub fn get_podping_write(&self) -> crate::podping_write_capnp::podping_write::Pipeline {
      ::capnp::capability::FromTypelessPipeline::new(self._typeless.get_pointer_field(0))
    }
  }
  mod _private {
    use capnp::private::layout;
    pub const STRUCT_SIZE: layout::StructSize = layout::StructSize { data: 0, pointers: 2 };
    pub const TYPE_ID: u64 = 0xa113_2166_6a87_4f82;
  }
}
