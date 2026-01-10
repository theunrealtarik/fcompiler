use convert_case::{Case, Casing};
use std::str::FromStr;

pub mod signals;

#[derive(Debug, Clone, Copy)]
pub enum Signal {
    Item(signals::Item),
    Fluid(signals::Fluid),
    Virtual(signals::Virtual),
}

impl Signal {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(str: &str) -> Result<Self, std::io::ErrorKind> {
        if let Ok(signal) = signals::Item::from_str(str) {
            return Ok(Self::Item(signal));
        }

        if let Ok(signal) = signals::Fluid::from_str(str) {
            return Ok(Self::Fluid(signal));
        }

        if let Ok(signal) = signals::Virtual::from_str(str) {
            return Ok(Self::Virtual(signal));
        }

        Err(std::io::ErrorKind::NotFound)
    }

    pub fn name(&self) -> String {
        match self {
            Signal::Item(item) => item.to_string(),
            Signal::Fluid(fluid) => fluid.to_string(),
            Signal::Virtual(r#virtual) => r#virtual.to_string(),
        }
        .to_case(Case::Kebab)
    }

    pub fn category(&self) -> String {
        match self {
            Signal::Item(item) => item.category(),
            Signal::Fluid(fluid) => fluid.category(),
            Signal::Virtual(r#virtual) => r#virtual.category(),
        }
    }

    pub fn format(&self) -> String {
        format!("[{}={}]", self.category(), self.name())
    }
}
