use yrs::{
    Any, Array, Doc, In, Out, ReadTxn, Transact, Update, XmlElementPrelim,
    types::{
        AsPrelim, Delta,
        xml::{XmlDeltaPrelim, XmlIn},
    },
    updates::decoder::Decode,
};

use crate::latex_render::{
    block::{Block, Marks},
    errors,
};

pub fn convert(blob: &[u8]) -> Result<Block, errors::Error> {
    let doc = convert_to_doc(blob)?;
    let txn = doc.transact();

    let Some(array) = txn.get_array("prosemirror") else {
        return Err(errors::Error::FormatError(
            "cannot get default array".into(),
        ));
    };

    let mut nodes = Vec::with_capacity(array.len(&txn) as usize);
    for node in array.iter(&txn) {
        let Out::YXmlElement(elem) = node else {
            continue;
        };
        let elem = elem.as_prelim(&txn);
        let node = yxml_element_convert_to_block(elem)?;
        nodes.push(node);
    }

    Ok(Block {
        content_type: "doc".into(),
        children: nodes,

        ..Default::default()
    })
}

fn convert_to_doc(blob: &[u8]) -> Result<Doc, errors::Error> {
    let doc = Doc::new();

    {
        let mut update_txn = doc.transact_mut();
        let update = match Update::decode_v1(blob) {
            Ok(update) => update,
            Err(err) => return Err(errors::Error::FormatError(err.to_string().into())),
        };
        if let Err(err) = update_txn.apply_update(update) {
            return Err(errors::Error::FormatError(err.to_string().into()));
        }
        update_txn.commit();
        update_txn.gc(None);
    }

    Ok(doc)
}

fn yxml_element_convert_to_block(elem: XmlElementPrelim) -> Result<Block, errors::Error> {
    let mut result = Block {
        content_type: elem.tag.into(),

        ..Default::default()
    };

    for (key, value) in elem.attributes {
        result.attrs.insert(key, value.into());
    }

    for node in elem.children {
        match node {
            XmlIn::Text(text) => {
                let mut contents = ydelta_convert_to_block_list(text)?;
                result.children.append(&mut contents);
            }
            XmlIn::Element(elem) => {
                let content = yxml_element_convert_to_block(elem)?;
                result.children.push(content);
            }
            XmlIn::Fragment(_frag) => {}
        }
    }

    Ok(result)
}

fn ydelta_convert_to_block_list(elem: XmlDeltaPrelim) -> Result<Vec<Block>, errors::Error> {
    let mut result = Vec::with_capacity(elem.delta.len());

    for delta in elem.delta.iter() {
        match delta {
            Delta::Inserted(In::Any(Any::String(value)), attrs) => {
                let mut content = Block {
                    content_type: "text".into(),
                    text: value.clone(),

                    ..Default::default()
                };

                if let Some(attrs) = attrs {
                    for key in attrs.keys() {
                        content.marks.push(Marks {
                            marks_type: key.clone(),

                            ..Default::default()
                        });
                    }
                }

                result.push(content);
            }
            _ => {}
        }
    }

    Ok(result)
}
