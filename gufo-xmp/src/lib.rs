#![doc = include_str!("../README.md")]

use std::collections::BTreeMap;
use std::io::Cursor;
use std::sync::Arc;

use gufo_common::field;
use gufo_common::xmp::{Namespace, XML_NS_RDF};
use xml::name::OwnedName;
use xml::reader::XmlEvent;
use xml::{writer, EmitterConfig, ParserConfig};

impl Tag {
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

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct Tag {
    namespace: Namespace,
    name: String,
}

impl Tag {
    pub fn new(namespace: Namespace, name: String) -> Self {
        Self { namespace, name }
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

enum ReaderState {
    Nothing,
    RdfDescription,
    Tag(Tag),
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

    pub fn get(&self, ref_: &Tag) -> Option<&str> {
        self.entries.get(ref_).map(|x| x.as_str())
    }

    pub fn model(&self) -> Option<String> {
        self.get(&gufo_common::field::Model.into())
            .map(ToString::to_string)
    }

    pub fn creator(&self) -> Option<String> {
        self.get(&field::Creator.into()).map(ToString::to_string)
    }

    pub fn entries(&self) -> &BTreeMap<Tag, String> {
        &self.entries
    }

    fn lookup(
        data: &[u8],
        updates: BTreeMap<Tag, String>,
    ) -> Result<(BTreeMap<Tag, String>, Vec<u8>), Error> {
        let parser = ParserConfig::default()
            .ignore_root_level_whitespace(false)
            .create_reader(Cursor::new(data));

        let mut output = Vec::new();
        let mut writer = EmitterConfig::default()
            .write_document_declaration(false)
            .pad_self_closing(false)
            .create_writer(&mut output);

        let mut reader_state = ReaderState::Nothing;
        let mut found_tags = BTreeMap::new();

        for e in parser {
            match e? {
                ref event @ XmlEvent::StartElement {
                    ref name,
                    ref attributes,
                    ref namespace,
                } => {
                    let mut event = event.clone();

                    if local_name(name) == "Description" && get_namespace(name) == Some(XML_NS_RDF)
                    {
                        let mut attributes = attributes.clone();

                        reader_state = ReaderState::RdfDescription;
                        for attr in attributes.iter_mut() {
                            if let Some(tag) = Tag::from_name(&attr.name) {
                                // Apply updates
                                if let Some(value) = updates.get(&tag) {
                                    value.clone_into(&mut attr.value);
                                };
                                // Store entry
                                found_tags.entry(tag).or_insert(attr.value.clone());
                            }
                        }
                        event = XmlEvent::StartElement {
                            name: name.to_owned(),
                            attributes,
                            namespace: namespace.to_owned(),
                        }
                    } else if matches!(reader_state, ReaderState::RdfDescription) {
                        if let Some(tag) = Tag::from_name(name) {
                            reader_state = ReaderState::Tag(tag);
                        }
                    }

                    if let Some(event) = event.as_writer_event() {
                        writer.write(event)?;
                    }
                }
                XmlEvent::Characters(data) => {
                    let mut data = &data;
                    let mut event = writer::XmlEvent::Characters(data);

                    if let ReaderState::Tag(ref tag) = reader_state {
                        // Apply update
                        if let Some(value) = updates.get(tag) {
                            event = writer::XmlEvent::Characters(value);
                            data = value;
                        };
                        // Store entry
                        found_tags.entry(tag.to_owned()).or_insert(data.clone());
                    }

                    writer.write(event)?;
                }
                ref event @ XmlEvent::EndElement { ref name } => {
                    match reader_state {
                        ReaderState::RdfDescription
                            if local_name(name) == "Description"
                                && get_namespace(name) == Some(XML_NS_RDF) =>
                        {
                            reader_state = ReaderState::Nothing;
                        }
                        ReaderState::Tag(_) => {
                            reader_state = ReaderState::RdfDescription;
                        }
                        _ => {}
                    }

                    if let Some(event) = event.as_writer_event() {
                        writer.write(event)?;
                    }
                }
                event => {
                    if let Some(event) = event.as_writer_event() {
                        writer.write(event)?;
                    }
                }
            }
        }

        Ok((found_tags, output))
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
