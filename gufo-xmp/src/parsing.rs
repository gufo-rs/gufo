use std::collections::BTreeMap;
use std::io::Cursor;

use gufo_common::xmp::XML_NS_RDF;
use xml::reader::XmlEvent;
use xml::{writer, EmitterConfig, ParserConfig};

use super::{get_namespace, local_name, Error, Tag, Xmp};

enum ReaderState {
    Nothing,
    RdfDescription,
    Tag(Tag),
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
}
