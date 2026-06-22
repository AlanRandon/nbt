use crate::{Compound, List, ListVariant, Variant};
use std::fmt::Display;

impl Display for Variant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_indented(f, 0)
    }
}

trait IndentedDisplay {
    fn fmt_indented(&self, f: &mut std::fmt::Formatter<'_>, indent: usize) -> std::fmt::Result;
}

impl IndentedDisplay for Variant {
    fn fmt_indented(&self, f: &mut std::fmt::Formatter<'_>, indent: usize) -> std::fmt::Result {
        match self {
            Self::Int8(value) => PrimitiveDisplay::fmt(value, f),
            Self::Int16(value) => PrimitiveDisplay::fmt(value, f),
            Self::Int32(value) => PrimitiveDisplay::fmt(value, f),
            Self::Int64(value) => PrimitiveDisplay::fmt(value, f),
            Self::Float32(value) => PrimitiveDisplay::fmt(value, f),
            Self::Float64(value) => PrimitiveDisplay::fmt(value, f),
            Self::String(value) => PrimitiveDisplay::fmt(value, f),
            Self::List(list) => list.fmt_indented(f, indent),
            Self::Compound(compound) => compound.fmt_indented(f, indent),
            Self::Int8List(list_variant) => list_variant.fmt_typed_list(f),
            Self::Int32List(list_variant) => list_variant.fmt_typed_list(f),
            Self::Int64List(list_variant) => list_variant.fmt_typed_list(f),
        }
    }
}

impl IndentedDisplay for List {
    fn fmt_indented(&self, f: &mut std::fmt::Formatter<'_>, indent: usize) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "[]"),
            Self::Int8(list_variant) => list_variant.fmt_list(f),
            Self::Int16(list_variant) => list_variant.fmt_list(f),
            Self::Int32(list_variant) => list_variant.fmt_list(f),
            Self::Int64(list_variant) => list_variant.fmt_list(f),
            Self::Float32(list_variant) => list_variant.fmt_list(f),
            Self::Float64(list_variant) => list_variant.fmt_list(f),
            Self::String(list_variant) => list_variant.fmt_list(f),
            Self::List(list_variant) => list_variant.fmt_indented(f, indent),
            Self::Compound(list_variant) => list_variant.fmt_indented(f, indent),
            Self::Int8List(list_variant) => list_variant.fmt_indented(f, indent),
            Self::Int32List(list_variant) => list_variant.fmt_indented(f, indent),
            Self::Int64List(list_variant) => list_variant.fmt_indented(f, indent),
        }
    }
}

impl IndentedDisplay for Compound {
    fn fmt_indented(&self, f: &mut std::fmt::Formatter<'_>, indent: usize) -> std::fmt::Result {
        write!(f, "{{\n")?;
        for (index, (key, value)) in self.0.iter().enumerate() {
            write!(
                f,
                "{empty:\t>indent$}\"{key}\": ",
                empty = "",
                indent = indent + 1
            )?;

            value.fmt_indented(f, indent + 1)?;

            if index < self.0.len().saturating_sub(1) {
                write!(f, ",\n")?;
            } else {
                write!(f, "\n")?;
            }
        }
        write!(f, "{empty:\t>indent$}}}", empty = "", indent = indent)?;
        Ok(())
    }
}

pub trait TypeSymbol {
    const TYPE_SYMBOL: &str;
    const ARRAY_SYMBOL: char;
}

macro_rules! impl_type_symbol {
    ($T:ty, $symbol:expr, $array_symbol:expr) => {
        impl TypeSymbol for $T {
            const TYPE_SYMBOL: &str = $symbol;
            const ARRAY_SYMBOL: char = $array_symbol;
        }
    };
}

impl_type_symbol!(u8, "uB", 'B');
impl_type_symbol!(u16, "uS", 'S');
impl_type_symbol!(u32, "uI", 'I');
impl_type_symbol!(u64, "uL", 'L');
impl_type_symbol!(f32, "f", 'F');
impl_type_symbol!(f64, "d", 'D');

pub trait PrimitiveDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}

impl<T: TypeSymbol + Display> PrimitiveDisplay for T {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self, T::TYPE_SYMBOL)
    }
}

impl PrimitiveDisplay for String {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{self}\"")
    }
}

impl<T: TypeSymbol + Display> ListVariant<T> {
    fn fmt_typed_list(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}; ", T::ARRAY_SYMBOL)?;
        for (index, item) in self.0.iter().enumerate() {
            item.fmt(f)?;
            if index < self.0.len().saturating_sub(1) {
                write!(f, ", ")?;
            }
        }
        write!(f, "]")?;
        Ok(())
    }
}

impl<T: PrimitiveDisplay> ListVariant<T> {
    fn fmt_list(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for (index, item) in self.0.iter().enumerate() {
            item.fmt(f)?;
            if index < self.0.len().saturating_sub(1) {
                write!(f, ", ")?;
            }
        }
        write!(f, "]")?;
        Ok(())
    }
}

impl<T: IndentedDisplay> IndentedDisplay for ListVariant<T> {
    fn fmt_indented(&self, f: &mut std::fmt::Formatter<'_>, indent: usize) -> std::fmt::Result {
        write!(f, "[\n")?;
        for (index, item) in self.0.iter().enumerate() {
            write!(f, "{empty:\t>indent$}", empty = "", indent = indent + 1)?;
            item.fmt_indented(f, indent + 1)?;
            if index < self.0.len().saturating_sub(1) {
                write!(f, ",\n")?;
            } else {
                write!(f, "\n")?;
            }
        }
        write!(f, "{empty:\t>indent$}]", empty = "", indent = indent)?;
        Ok(())
    }
}

impl IndentedDisplay for ListVariant<ListVariant<u8>> {
    fn fmt_indented(&self, f: &mut std::fmt::Formatter<'_>, indent: usize) -> std::fmt::Result {
        write!(f, "[\n")?;
        for (index, item) in self.0.iter().enumerate() {
            write!(f, "{empty:\t>indent$}", empty = "", indent = indent + 1)?;
            item.fmt_typed_list(f)?;
            if index < self.0.len().saturating_sub(1) {
                write!(f, ",\n")?;
            } else {
                write!(f, "\n")?;
            }
        }
        write!(f, "{empty:\t>indent$}]", empty = "", indent = indent)?;
        Ok(())
    }
}

impl IndentedDisplay for ListVariant<ListVariant<u16>> {
    fn fmt_indented(&self, f: &mut std::fmt::Formatter<'_>, indent: usize) -> std::fmt::Result {
        write!(f, "[\n")?;
        for (index, item) in self.0.iter().enumerate() {
            write!(f, "{empty:\t>indent$}", empty = "", indent = indent + 1)?;
            item.fmt_typed_list(f)?;
            if index < self.0.len().saturating_sub(1) {
                write!(f, ",\n")?;
            } else {
                write!(f, "\n")?;
            }
        }
        write!(f, "{empty:\t>indent$}]", empty = "", indent = indent)?;
        Ok(())
    }
}

impl IndentedDisplay for ListVariant<ListVariant<u32>> {
    fn fmt_indented(&self, f: &mut std::fmt::Formatter<'_>, indent: usize) -> std::fmt::Result {
        write!(f, "[\n")?;
        for (index, item) in self.0.iter().enumerate() {
            write!(f, "{empty:\t>indent$}", empty = "", indent = indent + 1)?;
            item.fmt_typed_list(f)?;
            if index < self.0.len().saturating_sub(1) {
                write!(f, ",\n")?;
            } else {
                write!(f, "\n")?;
            }
        }
        write!(f, "{empty:\t>indent$}]", empty = "", indent = indent)?;
        Ok(())
    }
}

impl IndentedDisplay for ListVariant<ListVariant<u64>> {
    fn fmt_indented(&self, f: &mut std::fmt::Formatter<'_>, indent: usize) -> std::fmt::Result {
        write!(f, "[\n")?;
        for (index, item) in self.0.iter().enumerate() {
            write!(f, "{empty:\t>indent$}", empty = "", indent = indent + 1)?;
            item.fmt_typed_list(f)?;
            if index < self.0.len().saturating_sub(1) {
                write!(f, ",\n")?;
            } else {
                write!(f, "\n")?;
            }
        }
        write!(f, "{empty:\t>indent$}]", empty = "", indent = indent)?;
        Ok(())
    }
}
