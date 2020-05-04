use super::conductor::BinaryElement;
use crate::common;
use crate::spec::types::hw::*;
use std::collections::HashMap;
use std::convert::TryFrom;

#[derive(Debug)]
pub enum Error {
    DuplicateLabel(String),
    UnknownLabel(String),
}

impl BinaryElement {
    fn words(&self) -> usize {
        match self {
            BinaryElement::LabelDef(_) => 0,
            BinaryElement::Data(raw) => raw.len(),
            BinaryElement::Inst(blob) => blob.words(),
        }
    }

    fn resolve<F>(self, resolver: F) -> Result<Vec<Word>, Error>
    where
        F: Fn(String) -> Result<Word, Error>,
    {
        match self {
            BinaryElement::LabelDef(_) => Ok(vec![]),
            BinaryElement::Data(raw) => Ok(raw),
            BinaryElement::Inst(blob) => blob.resolve(resolver),
        }
    }
}

fn build_label_map(elems: &Vec<BinaryElement>) -> Result<HashMap<String, Word>, Error> {
    let mut label_map = HashMap::new();

    let mut bs = 0;
    for e in elems {
        if let BinaryElement::LabelDef(label) = e {
            if label_map
                .insert(label.clone(), Word::try_from(bs).unwrap())
                .is_some()
            {
                return Err(Error::DuplicateLabel(label.clone()));
            }
        }

        bs += 2 * e.words();
    }

    Ok(label_map)
}

pub(super) fn resolve(elems: Vec<BinaryElement>) -> Result<Vec<Word>, Error> {
    let label_map = build_label_map(&elems)?;
    let label_resolver = |tag| {
        Ok(label_map.get(&tag).map(|v| *v))
            .transpose()
            .unwrap_or(Err(Error::UnknownLabel(tag)))
    };

    common::accumulate(elems.into_iter().map(|be| be.resolve(label_resolver)))
}
