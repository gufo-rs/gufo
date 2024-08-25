#![doc = include_str!("../README.md")]

mod parsing;
mod predefined;

use std::collections::BTreeMap;
use std::sync::Arc;

use gufo_common::xmp::Namespace;
use xml::name::OwnedName;

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct Tag {
    namespace: Namespace,
    name: String,
}

impl Tag {
    pub fn new(namespace: Namespace, name: String) -> Self {
        Self { namespace, name }
    }

    fn from_name(name: &OwnedName) -> Option<Self> {
        if let Some(namespace_url) = get_namespace(name) {
            let namespace = Namespace::from_url(namespace_url);

            let name = local_name(name).to_owned();
            Some(Self::new(namespace, name))
        } else {
            None
        }
    }
}

impl<T: gufo_common::xmp::Field> From<T> for Tag {
    fn from(_: T) -> Self {
        Self {
            name: T::NAME.to_string(),
            namespace: T::NAMESPACE,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Xmp {
    inner: Vec<u8>,
    entries: BTreeMap<Tag, String>,
}

#[derive(Debug, Clone, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    #[error("XmlReader: {0}")]
    XmlReader(xml::reader::Error),
    #[error("XmlWriter: {0}")]
    XmlWriter(Arc<xml::writer::Error>),
}

impl From<xml::reader::Error> for Error {
    fn from(value: xml::reader::Error) -> Self {
        Self::XmlReader(value)
    }
}

impl From<xml::writer::Error> for Error {
    fn from(value: xml::writer::Error) -> Self {
        Self::XmlWriter(Arc::new(value))
    }
}

impl Xmp {
    pub fn new(data: Vec<u8>) -> Result<Self, Error> {
        let (entries, _) = Self::lookup(&data, Default::default())?;

        Ok(Self {
            inner: data,
            entries,
        })
    }

    pub fn update(&mut self, updates: BTreeMap<Tag, String>) -> Result<(), Error> {
        let (entries, data) = Self::lookup(&self.inner, updates)?;
        self.entries = entries;
        self.inner = data;

        Ok(())
    }

    pub fn get(&self, tag: impl Into<Tag>) -> Option<&str> {
        self.entries.get(&tag.into()).map(|x| x.as_str())
    }

    pub fn get_frac(&self, tag: impl Into<Tag>) -> Option<(u32, u32)> {
        let (x, y) = self.get(tag)?.split_once('/')?;
        let x = x.parse().ok()?;
        let y = y.parse().ok()?;

        Some((x, y))
    }

    pub fn get_frac_f32(&self, tag: impl Into<Tag>) -> Option<f32> {
        let (x, y) = self.get_frac(tag)?;

        let res = x as f32 / y as f32;
        if res.is_finite() {
            Some(res)
        } else {
            None
        }
    }

    pub fn get_u16(&self, tag: impl Into<Tag>) -> Option<u16> {
        self.get(tag)?.parse().ok()
    }

    #[cfg(feature = "chrono")]
    pub fn get_date_time(&self, tag: impl Into<Tag>) -> Option<gufo_common::datetime::DateTime> {
        Some(gufo_common::datetime::DateTime::FixedOffset(
            self.get(tag)
                .and_then(|x| chrono::DateTime::parse_from_rfc3339(x).ok())?,
        ))
    }

    pub fn entries(&self) -> &BTreeMap<Tag, String> {
        &self.entries
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.inner
    }
}

fn local_name(OwnedName { local_name, .. }: &OwnedName) -> &str {
    local_name.as_str()
}

fn get_namespace(OwnedName { namespace, .. }: &OwnedName) -> Option<&str> {
    namespace.as_ref().map(|x| x.as_str())
}
