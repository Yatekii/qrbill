use crate::billing_infos::swico::{StructuredSet, SwicoComponent, Version};
use std::{collections::BTreeMap, sync::Arc};

#[derive(Debug, thiserror::Error)]
pub enum SyntaxParserError {
    #[error("Invalid Swico beacon/group, found : {0:?}")]
    InvalidBeacons(String),
    #[error("Could not find index during parsing, this is as bug")]
    IndexError,
}
type Err = SyntaxParserError;

pub fn s1_parser(s: &str) -> Result<Version, Err> {
    invalid_beacons(s)?;
    let (mut msg, mut stru) = (String::new(), String::new());
    if let Some((uns, st)) = s.split_once("//S1") {
        msg.push_str(uns);
        stru.push_str(st);
    } else {
        return Err(Err::IndexError);
    };
    let uns = msg.trim();
    let s = stru.as_str();
    let mut structured_set = StructuredSet::new();
    structured_set.insert(SwicoComponent::Unstructured, Arc::from(uns));
    let mut indexes: BTreeMap<u8, &SwicoComponent> = BTreeMap::new();
    let components = SwicoComponent::for_parsing();
    components.iter().for_each(|c| {
        let to_find = c.to_string();
        if let Some(x) = s.find(&to_find) {
            indexes.insert(x as u8, c);
        };
    });
    let indexes: Vec<(u8, &SwicoComponent)> = indexes.into_iter().collect();
    indexes.windows(2).try_for_each(|slice| -> Result<(), Err> {
        let (i1, c1) = slice.first().ok_or(Err::IndexError)?;
        let (i2, _) = slice.last().ok_or(Err::IndexError)?;
        let val = s[*i1 as usize..*i2 as usize].to_string();
        let val = val.replace(c1.to_string().as_str(), "");
        if !val.is_empty() {
            structured_set.insert(**c1, Arc::from(val));
        };
        Ok(())
    })?;
    let (lastu, lastc) = indexes.last().ok_or(Err::IndexError)?;
    let val = s[*lastu as usize..].to_string();
    let val = val.replace(lastc.to_string().as_str(), "");
    structured_set.insert(**lastc, Arc::from(val));
    structured_set.insert(SwicoComponent::Prefix, Arc::from("S1"));
    Ok(Version::S1(structured_set.clone()))
}

fn invalid_beacons(s: &str) -> Result<(), Err> {
    for t in SwicoComponent::invalids().iter() {
        if s.contains(t.as_str()) {
            return Err(SyntaxParserError::InvalidBeacons(t.into()));
        }
    }
    Ok(())
}
