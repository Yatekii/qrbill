use crate::billing_infos::{BillingInfoParagrah, DataType};
use std::collections::HashMap;

pub type RawData = HashMap<DataType, Vec<String>>;
impl TotalLength for RawData {
    fn tot_len(&self) -> usize {
        self.iter()
            .filter(|c| !matches!(c.0, DataType::Unstructured))
            .map(|(_, l)| l.iter().map(|c| c.chars().count()).sum::<usize>())
            .sum()
    }
}
impl Fold for RawData {
    fn fold_from(&self, d: &DataType) -> Option<String> {
        self.get(d)
            .map(|x| x.iter().fold(String::new(), |cur, nxt| cur + nxt))
    }
}

pub trait Fold {
    fn fold_from(&self, d: &DataType) -> Option<String>;
}
pub trait TotalLength {
    fn tot_len(&self) -> usize;
}
pub trait RawDataKind {
    fn raw_data(&self) -> Option<RawData>;
}

pub fn make_paragraph_from_raw(r: &RawData) -> Option<BillingInfoParagrah> {
    let i = match r.tot_len() {
        101..=140 => 3,
        49..=100 => 2,
        _ => 1,
    };
    let mut data = BillingInfoParagrah::new();
    if let Some(uns) = r.get(&DataType::Unstructured) {
        if let Some(unstructured) = uns.first() {
            if !unstructured.is_empty() {
                data.extend(split_unstructured(unstructured));
            }
        }
    }
    if let Some(structured) = r.get(&DataType::Structured) {
        let tot = structured.len();
        structured.chunks(tot.div_ceil(i)).for_each(|slice| {
            let s = slice
                .iter()
                .fold(String::new(), |cur: String, nxt| cur.to_owned() + nxt.as_str());
            if !s.is_empty() {
                data.push(s.clone());
            };
        });
    }
    if !data.is_empty() {
        Some(data)
    } else {
        None
    }
}
fn split_unstructured(s: &str) -> Vec<String> {
    let i = s.len();
    if i < 70 {
        return vec![s.to_string()];
    }
    let m = i / 2;
    let c = (i - m) / 2;
    let upper_bound = m + c;
    let lower_bound = m - c;
    let split_chars = [";", "/", "\\", ",", ".", " "];
    let index = split_chars
        .iter()
        .filter_map(|c| s.find(c))
        .filter(|c| *c > lower_bound && *c < upper_bound)
        .min();
    if let Some(split_i) = index {
        let (a, b) = s.split_at(split_i + 1);
        return vec![a.trim().into(), b.trim().into()];
    }
    vec!["".into()]
}
