use std::collections::BTreeMap;
use std::io::Cursor;

use gufo_common::xmp::XML_NS_RDF;
use xml::name::OwnedName;
use xml::reader::XmlEvent;
use xml::{EmitterConfig, ParserConfig, writer};

use super::{Error, Tag, Xmp};

#[derive(Debug)]
enum ReaderState {
    Nothing,
    RdfTag,
    /// Inside an rdf:Description or similar where the XMP properties are
    /// defined
    TypedNode,
    /// Inside an XMP property (also called XMP packet)
    Property(Tag),
    /// Inside an rdf:Bag (unordered list with duplicates)
    RdfBag(Tag),
    RdfBagLi(Tag),
    /// Inside an rdf:Seq
    RdfSeq(Tag),
    RdfSeqLi(Tag),
}

trait OwnedNameExt {
    fn is_rdf(&self, name: &str) -> bool;
}

impl OwnedNameExt for OwnedName {
    fn is_rdf(&self, name: &str) -> bool {
        self.local_name == name && self.namespace_ref() == Some(XML_NS_RDF)
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Generic(String),
    Bag(Vec<String>),
    Seq(Vec<String>),
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

        let mut reader_state: ReaderState = ReaderState::Nothing;
        let mut level_below_property_node = 0;
        let mut found_properties = BTreeMap::new();

        for event in parser {
            match event? {
                ref event @ XmlEvent::StartElement {
                    ref name,
                    ref attributes,
                    ref namespace,
                } => {
                    let mut event = event.clone();

                    match &reader_state {
                        ReaderState::Nothing => {
                            if name.is_rdf("RDF") {
                                reader_state = ReaderState::RdfTag;
                            } else {
                                #[cfg(feature = "tracing")]
                                tracing::debug!("Unknown element in toplevel: {name:?}");
                            }
                        }
                        ReaderState::RdfTag => {
                            // Start of a rdf:Description section with XMP elements
                            reader_state = ReaderState::TypedNode;

                            let mut attributes = attributes.clone();

                            // The rdf:Description element can contain simple XMP properties
                            // directly as attributes according to
                            // Section 7.9.2.2 of Part 1
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
                        }
                        ReaderState::TypedNode => {
                            // Inside rdf:Description, hence we are entering a property
                            if let Some(tag) = Tag::from_name(name) {
                                reader_state = ReaderState::Property(tag);
                            }
                        }
                        ReaderState::Property(tag) => {
                            level_below_property_node += 1;

                            if name.is_rdf("Bag") {
                                reader_state = ReaderState::RdfBag(tag.clone());
                            } else if name.is_rdf("Seq") {
                                reader_state = ReaderState::RdfSeq(tag.clone());
                            }
                        }
                        ReaderState::RdfBag(tag) if name.is_rdf("li") => {
                            reader_state = ReaderState::RdfBagLi(tag.clone());
                        }
                        ReaderState::RdfSeq(tag) if name.is_rdf("li") => {
                            reader_state = ReaderState::RdfSeqLi(tag.clone());
                        }
                        _ => {}
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
                    } else if let ReaderState::RdfBagLi(tag) = &reader_state {
                        let value = found_properties
                            .entry(tag.to_owned())
                            .or_insert(Value::Bag(Vec::new()));

                        if let Value::Bag(bag) = value {
                            bag.push(data.clone());
                        } else {
                            #[cfg(feature = "tracing")]
                            tracing::debug!("Reader state is RdfBagLi but value is not Bag");
                        }
                    } else if let ReaderState::RdfSeqLi(tag) = &reader_state {
                        let value = found_properties
                            .entry(tag.to_owned())
                            .or_insert(Value::Seq(Vec::new()));

                        if let Value::Seq(seq) = value {
                            seq.push(data.clone());
                        } else {
                            #[cfg(feature = "tracing")]
                            tracing::debug!("Reader state is RdSeqfLi but value is not Seq");
                        }
                    }

                    if UPDATE {
                        writer.write(event)?;
                    }
                }
                ref event @ XmlEvent::EndElement { .. } => {
                    match reader_state {
                        ReaderState::RdfTag => {
                            // rdf:RDF closed
                            reader_state = ReaderState::Nothing;
                        }
                        ReaderState::TypedNode => {
                            // rdf:Description or similar closed
                            reader_state = ReaderState::RdfTag;
                        }
                        ReaderState::Property(_) => {
                            if level_below_property_node == 0 {
                                reader_state = ReaderState::TypedNode;
                            } else {
                                level_below_property_node -= 1;
                            }
                        }
                        ReaderState::RdfBagLi(tag) => reader_state = ReaderState::RdfBag(tag),
                        ReaderState::RdfSeqLi(tag) => reader_state = ReaderState::RdfSeq(tag),
                        ReaderState::RdfBag(_) | ReaderState::RdfSeq(_) => {
                            reader_state = ReaderState::RdfTag;
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
