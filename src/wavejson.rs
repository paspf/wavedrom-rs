use serde::{Deserialize, Serialize};

use crate::{Figure, Wave, WaveLine, WaveLineGroup};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaveJson {
    pub signal: Vec<Signal>,
}

impl Into<Figure> for WaveJson {
    fn into(self) -> Figure {
        Figure::from_lines(
            self.signal
                .into_iter()
                .map(WaveLine::from)
                .collect::<Vec<WaveLine>>(),
        )
    }
}

impl From<Signal> for WaveLine {
    fn from(signal: Signal) -> Self {
        match signal {
            Signal::Group(items) => {
                let mut label = None;

                let items = items
                    .into_iter()
                    .filter_map(|item| match item {
                        SignalGroupItem::String(s) => {
                            if label.is_none() {
                                label = Some(s);
                            }

                            None
                        }
                        SignalGroupItem::Item(line) => Some(WaveLine::from(line)),
                    })
                    .collect::<Vec<WaveLine>>();

                WaveLine::Group(WaveLineGroup(label, items))
            }
            Signal::Item(item) => WaveLine::Wave(Wave::from(item)),
        }
    }
}

impl From<SignalItem> for Wave {
    fn from(item: SignalItem) -> Wave {
        Wave {
            name: item.name.unwrap_or_default(),
            cycles: item.wave.unwrap_or_default().parse().unwrap(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Signal {
    Group(Vec<SignalGroupItem>),
    Item(SignalItem),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SignalGroupItem {
    String(String),
    Item(Signal),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalItem {
    pub name: Option<String>,
    pub wave: Option<String>,
    pub data: Option<SignalData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SignalData {
    One(String),
    Multiple(Vec<String>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_basic() {
        let data = r#"
        {
            "signal": [
                { "name": "abc", "wave": "123" }
            ]
        }
        "#;

        let _wavejson: WaveJson = serde_json::from_str(data).unwrap();

        // dbg!(wavejson);
        // assert!(false);
    }

    #[test]
    fn to_svg() {
        use crate::svg::ToSvg;

        let data = r#"
        {
            "signal": [
                { "name": "abc", "wave": "120..." },
                [
                    "xyz",
                    { "name": "def", "wave": "00112200" },
                    [
                        "XYZ",
                        { "name": "def", "wave": "00112200" },
                        { "name": "def", "wave": "00112200" }
                    ]
                ]
            ]
        }
        "#;

        let wavejson: WaveJson = serde_json::from_str(data).unwrap();
        let figure: Figure = wavejson.into();

        let rendered = figure.assemble().unwrap();

        let mut file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(true)
            .create(true)
            .open("test.svg")
            .unwrap();

        rendered.write_svg(&mut file).unwrap();
    }
}
