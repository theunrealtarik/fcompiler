use convert_case::{Case, Casing};
use std::str::FromStr;

mod signals;

#[derive(Debug, Clone, Copy, strum_macros::Display)]
pub enum SignalId {
    Item(signals::Item),
    Fluid(signals::Fluid),
    Virtual(signals::Virtual),
}

impl SignalId {
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
            SignalId::Item(item) => item.to_string(),
            SignalId::Fluid(fluid) => fluid.to_string(),
            SignalId::Virtual(r#virtual) => r#virtual.to_string(),
        }
        .to_case(Case::Kebab)
    }

    pub fn category(&self) -> String {
        match self {
            SignalId::Item(item) => item.category(),
            SignalId::Fluid(fluid) => fluid.category(),
            SignalId::Virtual(r#virtual) => r#virtual.category(),
        }
    }

    pub fn format(&self) -> String {
        format!("[{}={}]", self.category(), self.name())
    }
}
