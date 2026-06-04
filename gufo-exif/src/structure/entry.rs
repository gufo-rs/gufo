use gufo_common::exif::Tag;
use zerocopy::{BE, BigEndian, ByteOrder, FromBytes, IntoBytes, LE, LittleEndian, U16, U32, U64};

use super::type_::Type;
use super::util::{IndexType, UsizeConversion};
use crate::Error;

pub struct EntryImmutable {
    pub tag: Tag,
    pub type_: Type,
    pub count: usize,
    pub value_or_offset: usize,
}

#[derive(Debug)]
pub enum Entry<'a> {
    Be32(&'a mut EntryTyped<U32<BigEndian>, BigEndian>),
    Le32(&'a mut EntryTyped<U32<LittleEndian>, LittleEndian>),
    Be64(&'a mut EntryTyped<U64<BigEndian>, BigEndian>),
    Le64(&'a mut EntryTyped<U64<LittleEndian>, LittleEndian>),
}

impl<'a> Entry<'a> {
    pub fn value_or_offset(&mut self) -> Result<ValueOrOffset<'_>, Error> {
        crate::forall_formats!(self, entry, entry.value_or_offset())
    }

    pub fn ifd_pointer(&mut self) -> Result<u16, Error> {
        crate::forall_formats!(self, entry, entry.ifd_pointer())
    }

    pub fn update(
        &mut self,
        tag: Tag,
        type_: Type,
        count: usize,
        value: Vec<u8>,
    ) -> Result<(), Error> {
        crate::forall_formats!(self, entry, {
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
        crate::forall_formats!(self, entry, entry.type_())
    }

    pub fn count(&self) -> Result<usize, Error> {
        crate::forall_formats!(self, entry, entry.count())
    }

    pub fn to_immutable(&self) -> Result<EntryImmutable, Error> {
        let (tag_id, type_, count, value_or_offset) = match self {
            Self::Be32(x) => (
                x.tag_id.get(),
                x.type_.get(),
                x.count.try_to_usize()?,
                x.value_or_offset.try_to_usize()?,
            ),
            Self::Le32(x) => (
                x.tag_id.get(),
                x.type_.get(),
                x.count.try_to_usize()?,
                x.value_or_offset.try_to_usize()?,
            ),
            Self::Be64(x) => (
                x.tag_id.get(),
                x.type_.get(),
                x.count.try_to_usize()?,
                x.value_or_offset.try_to_usize()?,
            ),
            Self::Le64(x) => (
                x.tag_id.get(),
                x.type_.get(),
                x.count.try_to_usize()?,
                x.value_or_offset.try_to_usize()?,
            ),
        };

        Ok(EntryImmutable {
            tag: Tag(tag_id),
            type_: Type::from(type_),
            count,
            value_or_offset,
        })
    }
}

#[derive(
    Debug, zerocopy::FromBytes, zerocopy::KnownLayout, zerocopy::IntoBytes, zerocopy::Immutable,
)]
#[repr(C)]
pub struct EntryTyped<T: IndexType + zerocopy::Immutable, O: ByteOrder> {
    /// What data is stored (focal length etc)
    pub tag_id: U16<O>,
    pub type_: U16<O>,
    /// How often a value of the given type occurs
    pub count: T,
    pub value_or_offset: T,
}

impl<T: IndexType + zerocopy::Immutable, O: ByteOrder> EntryTyped<T, O> {
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
        if count == 1 && type_ == Type::Long {
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

impl EntryTyped<U32<BigEndian>, BigEndian> {
    pub fn as_entry(&mut self) -> Entry<'_> {
        Entry::Be32(self)
    }
}

impl EntryTyped<U32<LittleEndian>, LittleEndian> {
    pub fn as_entry(&mut self) -> Entry<'_> {
        Entry::Le32(self)
    }
}

impl EntryTyped<U64<BigEndian>, BigEndian> {
    pub fn as_entry(&mut self) -> Entry<'_> {
        Entry::Be64(self)
    }
}

impl EntryTyped<U64<LittleEndian>, LittleEndian> {
    pub fn as_entry(&mut self) -> Entry<'_> {
        Entry::Le64(self)
    }
}

#[derive(Debug)]
pub enum ValueOrOffset<'a> {
    Value(&'a mut [u8]),
    Offset(usize),
}
