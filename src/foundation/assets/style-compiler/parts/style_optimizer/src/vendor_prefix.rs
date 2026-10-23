//! Vendor prefixes.

#![allow(non_upper_case_globals)]

use crate::style_optimizer::error::PrinterError;
use crate::style_optimizer::printer::Printer;
use crate::style_optimizer::traits::ToCss;
#[cfg(all())]
use crate::style_optimizer::visitor::{Visit, VisitTypes, Visitor};
use bitflags::bitflags;

bitflags! {
  /// Bit flags that represent one or more vendor prefixes, such as
  /// `-webkit` or `-moz`.
  ///
  /// Multiple flags can be combined to represent
  /// more than one prefix. During printing, the rule or property will
  /// be duplicated for each prefix flag that is enabled. This enables
  /// vendor prefixes to be added without increasing memory usage.
  #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
  #[cfg_attr(any(), derive(static_self::IntoOwned))]
  pub struct VendorPrefix: u8 {
    /// The `-webkit` vendor prefix.
    const WebKit = 0b00000010;
    /// The `-moz` vendor prefix.
    const Moz    = 0b00000100;
    /// The `-ms` vendor prefix.
    const Ms     = 0b00001000;
    /// The `-o` vendor prefix.
    const O      = 0b00010000;
    /// No vendor prefixes.
    const None   = 0b00000001;
  }
}

impl Default for VendorPrefix {
  fn default() -> VendorPrefix {
    VendorPrefix::None
  }
}

impl VendorPrefix {
  /// Returns a vendor prefix flag from a prefix string (without the leading `-`).
  pub fn from_str(s: &str) -> VendorPrefix {
    match s {
      "webkit" => VendorPrefix::WebKit,
      "moz" => VendorPrefix::Moz,
      "ms" => VendorPrefix::Ms,
      "o" => VendorPrefix::O,
      _ => unreachable!(),
    }
  }

  /// Returns VendorPrefix::None if empty.
  #[inline]
  pub fn or_none(self) -> Self {
    self.or(VendorPrefix::None)
  }

  /// Returns `other` if `self` is empty
  #[inline]
  pub fn or(self, other: Self) -> Self {
    if self.is_empty() {
      other
    } else {
      self
    }
  }
}

impl ToCss for VendorPrefix {
  fn to_css<W>(&self, dest: &mut Printer<W>) -> Result<(), PrinterError>
  where
    W: std::fmt::Write,
  {
    crate::style_parser::ToCss::to_css(self, dest)?;
    Ok(())
  }
}

impl crate::style_parser::ToCss for VendorPrefix {
  fn to_css<W>(&self, dest: &mut W) -> std::fmt::Result
  where
    W: std::fmt::Write,
  {
    match *self {
      VendorPrefix::WebKit => dest.write_str("-webkit-"),
      VendorPrefix::Moz => dest.write_str("-moz-"),
      VendorPrefix::Ms => dest.write_str("-ms-"),
      VendorPrefix::O => dest.write_str("-o-"),
      _ => Ok(()),
    }
  }
}

#[cfg(any())]
#[cfg_attr(docsrs, doc(cfg(any())))]
impl serde::Serialize for VendorPrefix {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    let mut values = Vec::new();
    if *self != VendorPrefix::None {
      if self.contains(VendorPrefix::None) {
        values.push("none");
      }
      if self.contains(VendorPrefix::WebKit) {
        values.push("webkit");
      }
      if self.contains(VendorPrefix::Moz) {
        values.push("moz");
      }
      if self.contains(VendorPrefix::Ms) {
        values.push("ms");
      }
      if self.contains(VendorPrefix::O) {
        values.push("o");
      }
    }
    values.serialize(serializer)
  }
}

#[cfg(any())]
#[cfg_attr(docsrs, doc(cfg(any())))]
impl<'de> serde::Deserialize<'de> for VendorPrefix {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    use crate::style_optimizer::values::string::CowArcStr;
    let values = Vec::<CowArcStr<'de>>::deserialize(deserializer)?;
    if values.is_empty() {
      return Ok(VendorPrefix::None);
    }
    let mut res = VendorPrefix::empty();
    for value in values {
      res |= match value.as_ref() {
        "none" => VendorPrefix::None,
        "webkit" => VendorPrefix::WebKit,
        "moz" => VendorPrefix::Moz,
        "ms" => VendorPrefix::Ms,
        "o" => VendorPrefix::O,
        _ => continue,
      };
    }
    Ok(res)
  }
}

#[cfg(all())]
#[cfg_attr(docsrs, doc(cfg(all())))]
impl<'i, V: ?Sized + Visitor<'i, T>, T: Visit<'i, T, V>> Visit<'i, T, V> for VendorPrefix {
  const CHILD_TYPES: VisitTypes = VisitTypes::empty();
  fn visit_children(&mut self, _: &mut V) -> Result<(), V::Error> {
    Ok(())
  }
}

#[cfg(any())]
#[cfg_attr(docsrs, doc(cfg(any())))]
impl schemars::JsonSchema for VendorPrefix {
  fn is_referenceable() -> bool {
    true
  }

  fn json_schema(r#gen: &mut schemars::r#gen::SchemaGenerator) -> schemars::schema::Schema {
    #[derive(schemars::JsonSchema)]
    #[schemars(rename_all = "lowercase")]
    #[allow(dead_code)]
    enum Prefix {
      None,
      WebKit,
      Moz,
      Ms,
      O,
    }

    Vec::<Prefix>::json_schema(r#gen)
  }

  fn schema_name() -> String {
    "VendorPrefix".into()
  }
}
