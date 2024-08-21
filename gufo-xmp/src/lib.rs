use std::collections::BTreeMap;
use std::io::Cursor;
use std::sync::Arc;

use xml::name::OwnedName;
use xml::reader::XmlEvent;
use xml::{writer, EmitterConfig, ParserConfig};

const EXIF_EARLY_XML_NS: &str = "http://ns.adobe.com/exif/1.0/";
const EXIF_LATER_XML_NS: &str = "http://cipa.jp/exif/";
const RDF_XML_NS: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";
const TIFF_XML_NS: &str = "http://ns.adobe.com/tiff/1.0/";
const PS_XML_NS: &str = "http://ns.adobe.com/photoshop/1.0/";
const DC_XML_NS: &str = "http://purl.org/dc/elements/1.1/";
const XMP_XML_NS: &str = "http://ns.adobe.com/xap/1.0/";
const XMP_RIGHTS_XML_NS: &str = "http://ns.adobe.com/xap/1.0/rights/";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tag {
    Tiff,
    Exif,
    Ps,
    Dc,
    Xmp,
    XmpRights,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct Ref {
    tag: Tag,
    name: String,
}

impl Ref {
    pub fn new(tag: Tag, name: String) -> Self {
        Self { tag, name }
    }
}

impl<T: gufo_common::xmp::Field> From<T> for Ref {
    fn from(_: T) -> Self {
        let tag = if T::EX { Tag::Exif } else { Tag::Exif };

        Self {
            name: T::NAME.to_string(),
            tag,
        }
    }
}

enum ReaderState {
    Nothing,
    RdfDescription,
    Tag(Ref),
}

#[derive(Debug, Clone)]
pub struct Xmp {
    inner: Vec<u8>,
    entries: BTreeMap<Ref, String>,
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

    pub fn update(&mut self, updates: BTreeMap<Ref, String>) -> Result<(), Error> {
        let (entries, data) = Self::lookup(&self.inner, updates)?;
        self.entries = entries;
        self.inner = data;

        Ok(())
    }

    pub fn get(&self, ref_: &Ref) -> Option<&str> {
        self.entries.get(ref_).map(|x| x.as_str())
    }

    pub fn model(&self) -> Option<String> {
        self.get(&gufo_common::field::Model.into())
            .map(ToString::to_string)
    }

    pub fn entries(&self) -> &BTreeMap<Ref, String> {
        &self.entries
    }

    fn lookup(
        data: &[u8],
        updates: BTreeMap<Ref, String>,
    ) -> Result<(BTreeMap<Ref, String>, Vec<u8>), Error> {
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

                    if local_name(name) == "Description" && get_namespace(name) == Some(RDF_XML_NS)
                    {
                        let mut attributes = attributes.clone();

                        reader_state = ReaderState::RdfDescription;
                        for attr in attributes.iter_mut() {
                            if let Some(tag) = name_to_tag(&attr.name) {
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
                        if let Some(tag) = name_to_tag(name) {
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
                                && get_namespace(name) == Some(RDF_XML_NS) =>
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

fn name_to_tag(name: &OwnedName) -> Option<Ref> {
    if let Some(namespace) = get_namespace(name) {
        let tag = if namespace.starts_with(EXIF_LATER_XML_NS) {
            Tag::Exif
        } else {
            match namespace {
                EXIF_EARLY_XML_NS => Tag::Exif,
                TIFF_XML_NS => Tag::Tiff,
                PS_XML_NS => Tag::Ps,
                DC_XML_NS => Tag::Dc,
                XMP_XML_NS => Tag::Xmp,
                XMP_RIGHTS_XML_NS => Tag::XmpRights,
                _ => return None,
            }
        };

        let name = local_name(name).to_owned();
        Some(Ref { name, tag })
    } else {
        None
    }
}
