use gufo_common::exif::Tag;
use zerocopy::{BE, BigEndian, ByteOrder, FromBytes, IntoBytes, LE, LittleEndian, U16, U32, U64};

use super::type_::Type;
use super::util::IndexType;
use crate::Error;

#[derive(Debug)]
pub enum Entry<'a> {
    Be32(&'a mut EntryGeneric<U32<BigEndian>, BigEndian>),
    Le32(&'a mut EntryGeneric<U32<LittleEndian>, LittleEndian>),
    Be64(&'a mut EntryGeneric<U64<BigEndian>, BigEndian>),
    Le64(&'a mut EntryGeneric<U64<LittleEndian>, LittleEndian>),
}

impl<'a> Entry<'a> {
    pub fn tag(&self) -> Tag {
        crate::forall_formats_self!(self, entry, entry.tag())
    }

    pub fn value_or_offset(&mut self) -> Result<ValueOrOffset<'_>, Error> {
        crate::forall_formats_self!(self, entry, entry.value_or_offset())
    }

    pub fn ifd_pointer(&mut self) -> Result<u16, Error> {
        crate::forall_formats_self!(self, entry, entry.ifd_pointer())
    }

    pub fn update(
        &mut self,
        tag: Tag,
        type_: Type,
        count: usize,
        value: Vec<u8>,
    ) -> Result<(), Error> {
        crate::forall_formats_self!(self, entry, {
            entry.check_count_fits(count)?;
            entry.set_value(value)?;
            entry.set_count(count)?;
            entry.set_tag_id(tag);
            entry.set_type(type_);
        });

        Ok(())
    }

    pub fn update_offset(
        &mut self,
        tag: Tag,
        type_: Type,
        count: usize,
        value: usize,
    ) -> Result<(), Error> {
        let data = match self {
            Self::Be32(_) => U32::<BE>::new(value.try_into()?).as_bytes().to_vec(),
            Self::Le32(_) => U32::<LE>::new(value.try_into()?).as_bytes().to_vec(),
            Self::Be64(_) => U64::<BE>::new(value.try_into()?).as_bytes().to_vec(),
            Self::Le64(_) => U64::<LE>::new(value.try_into()?).as_bytes().to_vec(),
        };

        self.update(tag, type_, count, data)
    }

    pub fn type_(&self) -> Type {
        crate::forall_formats_self!(self, entry, entry.type_())
    }

    pub fn count(&self) -> Result<usize, Error> {
        crate::forall_formats_self!(self, entry, entry.count())
    }
}

#[derive(
    Debug, zerocopy::FromBytes, zerocopy::KnownLayout, zerocopy::IntoBytes, zerocopy::Immutable,
)]
#[repr(C)]
pub struct EntryGeneric<T: IndexType + zerocopy::Immutable, O: ByteOrder> {
    /// What data is stored (focal length etc)
    pub tag_id: U16<O>,
    pub type_: U16<O>,
    /// How often a value of the given type occurs
    pub count: T,
    pub value_or_offset: T,
}

impl<T: IndexType + zerocopy::Immutable, O: ByteOrder> EntryGeneric<T, O> {
    pub fn tag(&self) -> Tag {
        Tag(self.tag_id.get())
    }

    pub fn type_(&self) -> Type {
        Type::from(self.type_.get())
    }

    pub fn count(&self) -> Result<usize, Error> {
        self.count.try_to_usize()
    }

    pub fn serialize(&self) -> &[u8] {
        self.as_bytes()
    }

    pub fn set_tag_id(&mut self, tag_id: Tag) {
        self.tag_id = U16::<O>::new(tag_id.0);
    }

    pub fn set_type(&mut self, type_: Type) {
        self.type_ = U16::<O>::new(type_.u16());
    }

    pub fn check_count_fits(&self, count: usize) -> Result<(), Error> {
        T::try_from_usize(count).map(|_| ())
    }

    pub fn set_count(&mut self, count: usize) -> Result<(), Error> {
        self.count = T::try_from_usize(count)?;
        Ok(())
    }

    pub fn set_value(&mut self, mut value: Vec<u8>) -> Result<(), Error> {
        let target_len = std::mem::size_of::<T>();
        if value.len() < target_len {
            value.resize(target_len, 0);
        }

        self.value_or_offset = T::read_from_bytes(&value).map_err(|_| Error::TryFromSlice)?;

        Ok(())
    }

    pub fn value_or_offset(&mut self) -> Result<ValueOrOffset<'_>, Error> {
        let data_size = self.type_().size() * self.count.try_to_usize()?;
        if data_size > std::mem::size_of::<T>() {
            Ok(ValueOrOffset::Offset(self.value_or_offset.try_to_usize()?))
        } else {
            Ok(ValueOrOffset::Value(self.value_or_offset.as_mut_bytes()))
        }
    }

    pub fn ifd_pointer(&mut self) -> Result<u16, Error> {
        let count = self.count()?;
        let type_ = self.type_();
        if (count == 1 && type_ == Type::Long) || type_ == Type::Undefined {
            Ok(
                U16::<O>::read_from_prefix(self.value_or_offset.as_mut_bytes())
                    .map_err(|_| Error::TryFromSlice)?
                    .0
                    .get(),
            )
        } else {
            Err(Error::other(format!(
                "Invalid type/count {count}x{type_:?} for ifd pointer entry {}",
                self.tag_id.get()
            )))
        }
    }
}

impl EntryGeneric<U32<BigEndian>, BigEndian> {
    pub fn as_entry(&mut self) -> Entry<'_> {
        Entry::Be32(self)
    }
}

impl EntryGeneric<U32<LittleEndian>, LittleEndian> {
    pub fn as_entry(&mut self) -> Entry<'_> {
        Entry::Le32(self)
    }
}

impl EntryGeneric<U64<BigEndian>, BigEndian> {
    pub fn as_entry(&mut self) -> Entry<'_> {
        Entry::Be64(self)
    }
}

impl EntryGeneric<U64<LittleEndian>, LittleEndian> {
    pub fn as_entry(&mut self) -> Entry<'_> {
        Entry::Le64(self)
    }
}

#[derive(Debug)]
pub enum ValueOrOffset<'a> {
    Value(&'a mut [u8]),
    Offset(usize),
}
