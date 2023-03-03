use proc_macro2::{Ident, Span};
use syn::{*};
use syn::spanned::Spanned;

// Taken from https://github.com/serenity-rs/serenity/blob/current/command_attr/src/attributes.rs
// The library is under the ISC license, which is compatible with GPL based licenses
// TODO rewrite this with our own code maybe?
#[derive(Debug)]
pub struct Values {
    pub name: Ident,
    pub literals: Vec<Lit>,
    pub kind: ValueKind,
    pub span: Span,
}

impl Values {
    #[inline]
    pub fn new(name: Ident, kind: ValueKind, literals: Vec<Lit>, span: Span) -> Self {
        Values {
            name,
            literals,
            kind,
            span,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ValueKind {
    // #[<name>]
    Name,

    // #[<name> = <value>]
    Equals,

    // #[<name>([<value>, <value>, <value>, ...])]
    List,

    // #[<name>(<value>)]
    SingleList,
}

pub fn parse_values(attr: &Attribute) -> Result<Values> {
    let meta = attr.parse_meta()?;

    match meta {
        Meta::Path(path) => {
            let name = to_ident(&path)?;

            Ok(Values::new(name, ValueKind::Name, Vec::new(), attr.span()))
        },
        Meta::List(meta) => {
            let name = to_ident(&meta.path)?;
            let nested = meta.nested;

            if nested.is_empty() {
                return Err(Error::new(attr.span(), "list cannot be empty"));
            }

            let mut lits = Vec::with_capacity(nested.len());

            for meta in nested {
                match meta {
                    NestedMeta::Lit(l) => lits.push(l),
                    NestedMeta::Meta(m) => match m {
                        Meta::Path(path) => {
                            let i = to_ident(&path)?;
                            lits.push(Lit::Str(LitStr::new(&i.to_string(), i.span())));
                        }
                        Meta::List(_) | Meta::NameValue(_) => {
                            return Err(Error::new(attr.span(), "cannot nest a list; only accept literals and identifiers at this level"))
                        }
                    },
                }
            }

            let kind = if lits.len() == 1 { ValueKind::SingleList } else { ValueKind::List };

            Ok(Values::new(name, kind, lits, attr.span()))
        },
        Meta::NameValue(meta) => {
            let name = to_ident(&meta.path)?;
            let lit = meta.lit;

            Ok(Values::new(name, ValueKind::Equals, vec![lit], attr.span()))
        },
    }
}

fn to_ident(p: &Path) -> Result<Ident> {
    if p.segments.is_empty() {
        return Err(Error::new(p.span(), "cannot convert an empty path to an identifier"));
    }

    if p.segments.len() > 1 {
        return Err(Error::new(p.span(), "the path must not have more than one segment"));
    }

    if !p.segments[0].arguments.is_empty() {
        return Err(Error::new(p.span(), "the singular path segment must not have any arguments"));
    }

    Ok(p.segments[0].ident.clone())
}