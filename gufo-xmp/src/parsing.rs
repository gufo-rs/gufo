use std::collections::BTreeMap;
use std::io::Cursor;

use gufo_common::xmp::XML_NS_RDF;
use xml::name::OwnedName;
use xml::reader::XmlEvent;
use xml::{writer, EmitterConfig, ParserConfig};

use super::{get_namespace, local_name, Error, Tag, Xmp};

enum ReaderState {
    Nothing,
    /// Inside a description where the XMP properties are defined
    RdfDescription,
    /// Inside an XMP property (also called XMP packet)
    Property(Tag),
}

trait OwnedNameExt {
    fn is_rdf_description(&self) -> bool;
    fn tag_name(&self) -> &str;
    fn namespace(&self) -> Option<&str>;
}

impl OwnedNameExt for OwnedName {
    fn is_rdf_description(&self) -> bool {
        self.tag_name() == "Description" && self.namespace() == Some(XML_NS_RDF)
    }

    fn tag_name(&self) -> &str {
        &self.local_name.as_str()
    }

    fn namespace(&self) -> Option<&str> {
        self.namespace.as_ref().map(|x| x.as_str())
    }
}

impl Xmp {
    pub(crate) fn lookup(
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
        let mut found_properties = BTreeMap::new();

        for event in parser {
            match event? {
                ref event @ XmlEvent::StartElement {
                    ref name,
                    ref attributes,
                    ref namespace,
                } => {
                    let mut event = event.clone();

                    if name.is_rdf_description() {
                        // Start of a rdf:Description section with XMP elements
                        reader_state = ReaderState::RdfDescription;

                        let mut attributes = attributes.clone();

                        // The rdf:Description element can contain simple XMP properties directly as attributes according to Section 7.9.2.2 of Part 1
                        for attr in attributes.iter_mut() {
                            if let Some(tag) = Tag::from_name(&attr.name) {
                                // Apply updates
                                if let Some(value) = updates.get(&tag) {
                                    value.clone_into(&mut attr.value);
                                };
                                // Store property
                                found_properties.entry(tag).or_insert(attr.value.clone());
                            }
                        }

                        // Rewrite element with potentially updated properties
                        event = XmlEvent::StartElement {
                            name: name.to_owned(),
                            attributes,
                            namespace: namespace.to_owned(),
                        }
                    } else if matches!(reader_state, ReaderState::RdfDescription) {
                        // Inside rdf:Description, hence we are entering a property
                        if let Some(tag) = Tag::from_name(name) {
                            reader_state = ReaderState::Property(tag);
                        }
                    }

                    if let Some(event) = event.as_writer_event() {
                        writer.write(event)?;
                    }
                }
                XmlEvent::Characters(data) => {
                    let mut data = &data;
                    let mut event = writer::XmlEvent::Characters(data);

                    if let ReaderState::Property(ref tag) = reader_state {
                        // Apply update
                        if let Some(value) = updates.get(tag) {
                            event = writer::XmlEvent::Characters(value);
                            data = value;
                        };
                        // Store property
                        found_properties
                            .entry(tag.to_owned())
                            .or_insert(data.clone());
                    }

                    writer.write(event)?;
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

        Ok((found_properties, output))
    }
}
