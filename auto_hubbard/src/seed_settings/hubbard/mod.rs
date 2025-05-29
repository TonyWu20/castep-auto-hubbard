use std::{fmt::Display, str::FromStr};

use castep_periodic_table::element::ElementSymbol;
use derive_builder::Builder;

use castep_cell_parser::{BlockBuilder, BlockIO, CELLParser, CellParseError, Rule};
use pest::Parser;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Orbital {
    S,
    P,
    D,
    F,
}

impl Display for Orbital {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Orbital::S => "s",
                Orbital::P => "p",
                Orbital::D => "d",
                Orbital::F => "f",
            }
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HubbardType {
    U,
    Alpha,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Builder)]
#[builder()]
pub struct HubbardItem {
    element: ElementSymbol,
    atom_id: Option<usize>,
    orbital: Orbital,
    hub_value: f64,
}

impl Display for HubbardItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:>4} {} {}: {:>20.15}",
            format!("{:?}", self.element),
            self.atom_id.map_or(String::new(), |v| format!("{v}")),
            self.orbital,
            self.hub_value
        )
    }
}

impl HubbardItem {
    pub fn element(&self) -> ElementSymbol {
        self.element
    }

    pub fn element_mut(&mut self) -> &mut ElementSymbol {
        &mut self.element
    }

    pub fn set_element(&mut self, element: ElementSymbol) {
        self.element = element;
    }

    pub fn atom_id(&self) -> Option<usize> {
        self.atom_id
    }

    pub fn atom_id_mut(&mut self) -> &mut Option<usize> {
        &mut self.atom_id
    }

    pub fn set_atom_id(&mut self, atom_id: Option<usize>) {
        self.atom_id = atom_id;
    }

    pub fn orbital(&self) -> Orbital {
        self.orbital
    }

    pub fn orbital_mut(&mut self) -> &mut Orbital {
        &mut self.orbital
    }

    pub fn set_orbital(&mut self, orbital: Orbital) {
        self.orbital = orbital;
    }

    pub fn hub_value(&self) -> f64 {
        self.hub_value
    }

    pub fn hub_value_mut(&mut self) -> &mut f64 {
        &mut self.hub_value
    }

    pub fn set_hub_value(&mut self, u_value: f64) {
        self.hub_value = u_value;
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct HubbardBlock {
    // unit: EnergyUnit
    hubbard_type: HubbardType,
    settings: Vec<HubbardItem>,
}

impl HubbardBlock {
    pub fn new(hubbard_type: HubbardType, settings: Vec<HubbardItem>) -> Self {
        Self {
            hubbard_type,
            settings,
        }
    }

    pub fn settings(&self) -> &[HubbardItem] {
        &self.settings
    }

    pub fn settings_mut(&mut self) -> &mut Vec<HubbardItem> {
        &mut self.settings
    }

    pub fn set_settings(&mut self, settings: Vec<HubbardItem>) {
        self.settings = settings;
    }
    pub fn filter_item_mut<F: FnMut(&&mut HubbardItem) -> bool>(
        &mut self,
        item_condition: F,
    ) -> std::iter::Filter<std::slice::IterMut<'_, HubbardItem>, F> {
        self.settings_mut().iter_mut().filter(item_condition)
    }

    pub fn hubbard_type(&self) -> HubbardType {
        self.hubbard_type
    }
}

impl BlockIO for HubbardBlock {
    type Item = HubbardBlock;

    fn from_block(block: &castep_cell_parser::Block) -> Result<Self::Item, CellParseError> {
        let hub_type = match block.name().to_lowercase().as_str() {
            "hubbard_u" => Ok(HubbardType::U),
            "hubbard_alpha" => Ok(HubbardType::Alpha),
            _ => Err(CellParseError::UnexpectedBlockType((
                "HUBBARD_U or HUBBARD_ALPHA".to_string(),
                block.name().to_string(),
            ))),
        }?;
        let settings = block
            .values()
            .iter()
            .flat_map(|line| {
                let parsed = CELLParser::parse(Rule::hubbard_line, line).unwrap();
                let atom_id = parsed
                    .find_first_tagged("atom_id")
                    .and_then(|p| p.as_str().parse::<usize>().ok());
                let element_symbol = parsed
                    .find_first_tagged("symbol")
                    .and_then(|p| ElementSymbol::from_str(p.as_str()).ok())
                    .unwrap();
                // Collect hubbard settings from each line
                parsed
                    .filter_map(|pair| {
                        if matches!(pair.as_rule(), Rule::hubbard_value) {
                            let mut inner = pair.into_inner();
                            let orbital = inner
                                .next()
                                .and_then(|orbital| {
                                    match orbital.as_str().to_lowercase().as_str() {
                                        "s" => Some(Orbital::S),
                                        "p" => Some(Orbital::P),
                                        "d" => Some(Orbital::D),
                                        "f" => Some(Orbital::F),
                                        _ => None,
                                    }
                                })
                                .unwrap();
                            let value = inner
                                .next()
                                .and_then(|value| value.as_str().parse::<f64>().ok())
                                .unwrap();
                            Some(
                                HubbardItemBuilder::default()
                                    .element(element_symbol)
                                    .orbital(orbital)
                                    .atom_id(atom_id)
                                    .hub_value(value)
                                    .build()
                                    .unwrap(),
                            )
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<HubbardItem>>()
            })
            .collect::<Vec<HubbardItem>>();
        Ok(Self::new(hub_type, settings))
    }

    fn to_block(&self, order: usize) -> castep_cell_parser::Block {
        let name = match self.hubbard_type {
            HubbardType::U => "HUBBARD_U",
            HubbardType::Alpha => "HUBBARD_ALPHA",
        };
        BlockBuilder::default()
            .order(order)
            .name(name.to_string())
            .values(
                self.settings()
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>(),
            )
            .build()
            .unwrap()
    }
}
#[cfg(test)]
mod test {
    use std::str::FromStr;

    use castep_cell_parser::{BlockIO, CELLParser, Rule};
    use castep_periodic_table::element::ElementSymbol;
    use pest::Parser;

    use crate::seed_settings::hubbard::{HubbardItem, HubbardItemBuilder, HubbardType, Orbital};

    use super::HubbardBlock;

    #[test]
    fn test_hubbard_u() {
        let hubbard_u_line = "    U 2  d: 1.2 f: 2.1";
        let parsed = CELLParser::parse(Rule::hubbard_line, hubbard_u_line).unwrap();
        let atom_id = parsed
            .find_first_tagged("atom_id")
            .and_then(|p| p.as_str().parse::<usize>().ok());
        let element_symbol = parsed
            .find_first_tagged("symbol")
            .and_then(|p| ElementSymbol::from_str(p.as_str()).ok())
            .unwrap();
        let values = parsed
            .filter_map(|pair| {
                if matches!(pair.as_rule(), Rule::hubbard_value) {
                    let mut inner = pair.into_inner();
                    let orbital = inner
                        .next()
                        .and_then(|orbital| match orbital.as_str().to_lowercase().as_str() {
                            "s" => Some(Orbital::S),
                            "p" => Some(Orbital::P),
                            "d" => Some(Orbital::D),
                            "f" => Some(Orbital::F),
                            _ => None,
                        })
                        .unwrap();
                    let value = inner
                        .next()
                        .and_then(|value| value.as_str().parse::<f64>().ok())
                        .unwrap();
                    Some(
                        HubbardItemBuilder::default()
                            .element(element_symbol)
                            .orbital(orbital)
                            .atom_id(atom_id)
                            .hub_value(value)
                            .build()
                            .unwrap(),
                    )
                } else {
                    None
                }
            })
            .collect::<Vec<HubbardItem>>();
        values.iter().for_each(|item| println!("{item}"));
        let mut hubbard_u_block = HubbardBlock::new(HubbardType::U, values);
        println!("{}", hubbard_u_block.to_block(1));
        hubbard_u_block
            .filter_item_mut(|item| item.element() == ElementSymbol::U)
            .for_each(|item| item.set_hub_value(3.2));
        println!("{}", hubbard_u_block.to_block(1));
    }
}
