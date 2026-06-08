use std::collections::BTreeMap;
use std::io::Cursor;

use gufo_common::xmp::{XML_NS_CC, XML_NS_RDF};
use xml::name::OwnedName;
use xml::reader::XmlEvent;
use xml::{EmitterConfig, ParserConfig, writer};

use crate::parsing::ReaderState::RdfTagUselessLevel;

use super::{Error, Tag, Xmp};

#[derive(Debug)]
enum ReaderState {
    Nothing,
    /// Inside a description where the XMP properties are defined
    RdfDescription,
    /// Inside an XMP property (also called XMP packet)
    Property(Tag),
    RdfTag,
    RdfTagUselessLevel,
    RdfTagData(Tag),
    /// Inside an rdf:Bag (unordered list with duplicates)
    RdfBag(Tag),
    RdfLi(Tag),
}

trait OwnedNameExt {
    fn is_rdf_description(&self) -> bool;
    fn is_rdf_open_tag(&self) -> bool;
    fn is_useless_level(&self) -> bool;
    fn is_rdf(&self, name: &str) -> bool;
}

impl OwnedNameExt for OwnedName {
    fn is_rdf(&self, name: &str) -> bool {
        self.local_name == name && self.namespace_ref() == Some(XML_NS_RDF)
    }

    fn is_rdf_description(&self) -> bool {
        self.local_name == "Description" && self.namespace_ref() == Some(XML_NS_RDF)
    }

    fn is_rdf_open_tag(&self) -> bool {
        self.local_name == "RDF" && self.namespace_ref() == Some(XML_NS_RDF)
    }

    fn is_useless_level(&self) -> bool {
        (self.local_name == "Work" || self.local_name == "Agent")
            && self.namespace_ref() == Some(XML_NS_CC)
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Generic(String),
    Bag(Vec<String>),
}

impl Xmp {
    pub(crate) fn lookup(data: &[u8]) -> Result<BTreeMap<Tag, Value>, Error> {
        Self::parse::<false>(data, Default::default()).map(|x| x.0)
    }

    pub(crate) fn lookup_and_update(
        data: &[u8],
        updates: BTreeMap<Tag, Value>,
    ) -> Result<(BTreeMap<Tag, Value>, Vec<u8>), Error> {
        Self::parse::<true>(data, updates)
    }

    fn parse<const UPDATE: bool>(
        data: &[u8],
        updates: BTreeMap<Tag, Value>,
    ) -> Result<(BTreeMap<Tag, Value>, Vec<u8>), Error> {
        let parser = ParserConfig::default()
            .ignore_root_level_whitespace(false)
            .create_reader(Cursor::new(data));

        let mut output = Vec::new();

        let mut writer = EmitterConfig::default()
            .write_document_declaration(false)
            .pad_self_closing(false)
            .create_writer(&mut output);

        let mut reader_state = ReaderState::Nothing;
        let mut useless_level = 0;
        let mut found_properties = BTreeMap::new();

        for event in parser {
            match event? {
                ref event @ XmlEvent::StartElement {
                    ref name,
                    ref attributes,
                    ref namespace,
                } => {
                    let mut event = event.clone();
                    if let ReaderState::Property(tag) | ReaderState::RdfTagData(tag) = &reader_state
                        && name.is_rdf("Bag")
                    {
                        reader_state = ReaderState::RdfBag(tag.clone());
                    } else if let ReaderState::RdfBag(tag) = &reader_state
                        && name.is_rdf("li")
                    {
                        reader_state = ReaderState::RdfLi(tag.clone());
                    } else if matches!(reader_state, ReaderState::RdfTag) {
                        if !name.is_useless_level() {
                            if let Some(tag) = Tag::from_name(name) {
                                reader_state = ReaderState::RdfTagData(tag);
                                useless_level += 1;
                            }
                        }
                    } else if name.is_rdf_open_tag() {
                        reader_state = ReaderState::RdfTag;
                    } else if name.is_rdf_description() {
                        // Start of a rdf:Description section with XMP elements
                        reader_state = ReaderState::RdfDescription;

                        let mut attributes = attributes.clone();

                        // The rdf:Description element can contain simple XMP properties directly as
                        // attributes according to Section 7.9.2.2 of Part 1
                        for attr in attributes.iter_mut() {
                            if let Some(tag) = Tag::from_name(&attr.name) {
                                if UPDATE {
                                    // Apply updates
                                    if let Some(value) = updates.get(&tag) {
                                        if let Value::Generic(s) = value {
                                            s.clone_into(&mut attr.value);
                                        } else {
                                            return Err(Error::other(
                                                "Writing other values than generic into attributes is not supported.",
                                            ));
                                        }
                                    };
                                }
                                // Store property
                                found_properties
                                    .entry(tag)
                                    .or_insert(Value::Generic(attr.value.clone()));
                            }
                        }

                        if UPDATE {
                            // Rewrite element with potentially updated properties
                            event = XmlEvent::StartElement {
                                name: name.to_owned(),
                                attributes,
                                namespace: namespace.to_owned(),
                            }
                        }
                    } else if matches!(reader_state, ReaderState::RdfDescription) {
                        // Inside rdf:Description, hence we are entering a property
                        if let Some(tag) = Tag::from_name(name) {
                            reader_state = ReaderState::Property(tag);
                        }
                    }

                    if UPDATE {
                        if let Some(event) = event.as_writer_event() {
                            writer.write(event)?;
                        }
                    }
                }
                XmlEvent::Characters(data) => {
                    let mut data = &data;
                    let mut event = writer::XmlEvent::Characters(data);

                    if let ReaderState::Property(ref tag) = reader_state {
                        // Apply update
                        if let Some(value) = updates.get(tag) {
                            if let Value::Generic(s) = value {
                                event = writer::XmlEvent::Characters(s);
                                data = s;
                            } else {
                                return Err(Error::other(
                                    "Writing other values than generic is not supported yet.",
                                ));
                            }
                        };
                        // Store property
                        found_properties
                            .entry(tag.to_owned())
                            .or_insert(Value::Generic(data.clone()));
                    } else if let ReaderState::RdfTagData(tag) = &reader_state {
                        found_properties
                            .entry(tag.to_owned())
                            .or_insert(Value::Generic(data.clone()));
                    } else if let ReaderState::RdfLi(tag) = &reader_state {
                        let x = found_properties
                            .entry(tag.to_owned())
                            .or_insert(Value::Bag(Vec::new()));

                        if let Value::Bag(bag) = x {
                            bag.push(data.clone());
                        } else {
                            #[cfg(feature = "tracing")]
                            tracing::debug!("Reader state is RdfLi but value is not Bag");
                        }
                    }

                    if UPDATE {
                        writer.write(event)?;
                    }
                }
                ref event @ XmlEvent::EndElement { ref name } => {
                    match reader_state {
                        ReaderState::RdfDescription if name.is_rdf_description() => {
                            // rdf:Description closed
                            reader_state = ReaderState::Nothing;
                        }
                        ReaderState::Property(_) => {
                            // Property closed
                            reader_state = ReaderState::RdfDescription;
                        }
                        ReaderState::RdfTagData(_) => {
                            if useless_level > 0 {
                                reader_state = RdfTagUselessLevel;
                            } else {
                                reader_state = ReaderState::RdfTag
                            }
                        }
                        ReaderState::RdfTagUselessLevel => {
                            if useless_level > 0 {
                                reader_state = ReaderState::RdfTagUselessLevel;
                                useless_level -= 1;
                            } else {
                                reader_state = ReaderState::RdfTag;
                            }
                        }
                        _ => {}
                    }

                    if UPDATE {
                        if let Some(event) = event.as_writer_event() {
                            writer.write(event)?;
                        }
                    }
                }
                event => {
                    if UPDATE {
                        if let Some(event) = event.as_writer_event() {
                            writer.write(event)?;
                        }
                    }
                }
            }
        }

        Ok((found_properties, output))
    }
}
