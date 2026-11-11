//! A compiled schema (Jing's `SchemaImpl`: the expanded start pattern and its
//! builder) and its binary form, so a schema can be parsed at build time and
//! embedded in the binary.

use crate::datatype::{DatatypeId, DatatypeRef, ValueId};
use crate::name::{Name, Namespace};
use crate::name_class::{NameClass, NameClassId};
use crate::pattern::*;
use crate::pattern_builder::*;
use crate::schema_builder_impl::{ResourceLoader, SchemaBuilderImpl};

const MAGIC: &[u8; 8] = b"VNURNG01";
/// The patterns every [`PatternBuilder`] starts with, at fixed ids.
const FIXED: u32 = 5;

#[derive(Debug, Clone)]
pub struct Schema {
    pub pb: PatternBuilder,
    pub start: PatternId,
}

impl Schema {
    /// Parses a compact-syntax schema and expands it.
    pub fn parse(uri: &str, loader: &ResourceLoader<'_>) -> Result<Schema, String> {
        let mut sb = SchemaBuilderImpl::new(loader);
        let start = sb.parse(uri)?;
        let pb = std::mem::take(&mut sb.spb.pb);
        Ok(Schema { pb, start })
    }

    /// Serializes the patterns reachable from the start pattern.
    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        let pb = &self.pb;
        // Number reachable patterns; the fixed ones keep their ids.
        let mut map: Vec<u32> = vec![u32::MAX; pb.nodes.len()];
        for i in 0..FIXED {
            map[i as usize] = i;
        }
        let mut order: Vec<PatternId> = Vec::new();
        let mut stack = vec![self.start];
        while let Some(p) = stack.pop() {
            if map[p.0 as usize] != u32::MAX {
                continue;
            }
            map[p.0 as usize] = FIXED + order.len() as u32;
            order.push(p);
            match pb.pattern(p) {
                Pattern::Choice(a, b)
                | Pattern::Group(a, b)
                | Pattern::Interleave(a, b)
                | Pattern::After(a, b) => {
                    stack.push(b);
                    stack.push(a);
                }
                Pattern::OneOrMore(a)
                | Pattern::Element(_, a)
                | Pattern::Attribute(_, a)
                | Pattern::DataExcept(_, a)
                | Pattern::List(a) => stack.push(a),
                Pattern::Ref(_) => return Err("unexpanded reference in the schema".into()),
                _ => {}
            }
        }
        let id = |p: PatternId| map[p.0 as usize];

        let mut w = Writer(Vec::with_capacity(1 << 20));
        w.0.extend_from_slice(MAGIC);
        let namespaces = pb.names.namespaces();
        w.uint(namespaces.len() as u32);
        for ns in namespaces {
            w.str(ns);
        }
        let names = pb.names.names();
        w.uint(names.len() as u32);
        for (ns, local) in names {
            w.uint(ns.0);
            w.str(local);
        }
        let classes = pb.name_classes.all();
        w.uint(classes.len() as u32);
        for nc in classes {
            match *nc {
                NameClass::Simple(n) => {
                    w.uint(0);
                    w.uint(n.0);
                }
                NameClass::AnyName => w.uint(1),
                NameClass::AnyNameExcept(e) => {
                    w.uint(2);
                    w.uint(e.0);
                }
                NameClass::NsName(ns) => {
                    w.uint(3);
                    w.uint(ns.0);
                }
                NameClass::NsNameExcept(ns, e) => {
                    w.uint(4);
                    w.uint(ns.0);
                    w.uint(e.0);
                }
                NameClass::Choice(a, b) => {
                    w.uint(5);
                    w.uint(a.0);
                    w.uint(b.0);
                }
                NameClass::Null => w.uint(6),
            }
        }
        w.uint(pb.datatypes.len() as u32);
        for dt in &pb.datatypes {
            w.str(&dt.library);
            w.str(&dt.name);
            w.uint(dt.params.len() as u32);
            for (k, v) in &dt.params {
                w.str(k);
                w.str(v);
            }
        }
        w.uint(pb.values.len() as u32);
        for (dt, lexical) in &pb.values {
            w.uint(dt.0);
            w.str(lexical);
        }
        w.uint(order.len() as u32);
        for &p in &order {
            let node = pb.node(p);
            let (tag, a, b) = match node.pattern {
                Pattern::Choice(x, y) => (0, id(x), id(y)),
                Pattern::Group(x, y) => (1, id(x), id(y)),
                Pattern::Interleave(x, y) => (2, id(x), id(y)),
                Pattern::After(x, y) => (3, id(x), id(y)),
                Pattern::OneOrMore(x) => (4, id(x), 0),
                Pattern::Element(nc, x) => (5, nc.0, id(x)),
                Pattern::Attribute(nc, x) => (6, nc.0, id(x)),
                Pattern::Data(dt) => (7, dt.0, 0),
                Pattern::DataExcept(dt, x) => (8, dt.0, id(x)),
                Pattern::Value(dt, v) => (9, dt.0, v.0),
                Pattern::List(x) => (10, id(x), 0),
                other => return Err(format!("unexpected pattern in the schema: {other:?}")),
            };
            w.uint(tag);
            w.uint(a);
            w.uint(b);
            w.uint(node.nullable as u32);
            w.uint(node.content_type as u32);
        }
        w.uint(id(self.start));
        Ok(w.0)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Schema, String> {
        let mut r = Reader { bytes, pos: 0 };
        if bytes.len() < MAGIC.len() || &bytes[..MAGIC.len()] != MAGIC {
            return Err("not a compiled schema".into());
        }
        r.pos = MAGIC.len();
        let mut pb = PatternBuilder::default();
        for _ in 0..r.uint()? {
            let uri = r.str()?;
            pb.names.namespace(uri);
        }
        for _ in 0..r.uint()? {
            let ns = Namespace(r.uint()?);
            let local = r.str()?;
            pb.names.name(ns, local);
        }
        for _ in 0..r.uint()? {
            let nc = match r.uint()? {
                0 => NameClass::Simple(Name(r.uint()?)),
                1 => NameClass::AnyName,
                2 => NameClass::AnyNameExcept(NameClassId(r.uint()?)),
                3 => NameClass::NsName(Namespace(r.uint()?)),
                4 => NameClass::NsNameExcept(Namespace(r.uint()?), NameClassId(r.uint()?)),
                5 => NameClass::Choice(NameClassId(r.uint()?), NameClassId(r.uint()?)),
                6 => NameClass::Null,
                t => return Err(format!("bad name class tag {t}")),
            };
            pb.name_classes.intern(nc);
        }
        for _ in 0..r.uint()? {
            let library = r.str()?.to_string();
            let name = r.str()?.to_string();
            let mut params = Vec::new();
            for _ in 0..r.uint()? {
                params.push((r.str()?.to_string(), r.str()?.to_string()));
            }
            pb.datatype(DatatypeRef {
                library,
                name,
                params,
            });
        }
        for _ in 0..r.uint()? {
            let dt = DatatypeId(r.uint()?);
            let lexical = r.str()?;
            pb.value(dt, lexical);
        }
        let count = r.uint()?;
        let mut nodes = Vec::with_capacity(count as usize);
        for _ in 0..count {
            let (tag, a, b) = (r.uint()?, r.uint()?, r.uint()?);
            let (p1, p2) = (PatternId(a), PatternId(b));
            let pattern = match tag {
                0 => Pattern::Choice(p1, p2),
                1 => Pattern::Group(p1, p2),
                2 => Pattern::Interleave(p1, p2),
                3 => Pattern::After(p1, p2),
                4 => Pattern::OneOrMore(p1),
                5 => Pattern::Element(NameClassId(a), p2),
                6 => Pattern::Attribute(NameClassId(a), p2),
                7 => Pattern::Data(DatatypeId(a)),
                8 => Pattern::DataExcept(DatatypeId(a), p2),
                9 => Pattern::Value(DatatypeId(a), ValueId(b)),
                10 => Pattern::List(p1),
                t => return Err(format!("bad pattern tag {t}")),
            };
            let nullable = r.uint()? != 0;
            let content_type = r.uint()? as u8;
            nodes.push(PatternNode {
                pattern,
                nullable,
                content_type,
            });
        }
        pb.load_nodes(nodes);
        let start = PatternId(r.uint()?);
        Ok(Schema { pb, start })
    }
}

impl PatternBuilder {
    /// Appends deserialized nodes; the patterns Java keeps in the builder's
    /// `PatternInterner` are interned again.
    fn load_nodes(&mut self, nodes: Vec<PatternNode>) {
        for node in nodes {
            let id = PatternId(self.nodes.len() as u32);
            self.nodes.push(node);
            if matches!(
                node.pattern,
                Pattern::Choice(..)
                    | Pattern::Group(..)
                    | Pattern::Interleave(..)
                    | Pattern::OneOrMore(_)
                    | Pattern::After(..)
            ) {
                self.register_interned(node.pattern, id);
            }
        }
    }
}

struct Writer(Vec<u8>);

impl Writer {
    fn uint(&mut self, mut v: u32) {
        loop {
            let byte = (v & 0x7f) as u8;
            v >>= 7;
            if v == 0 {
                self.0.push(byte);
                return;
            }
            self.0.push(byte | 0x80);
        }
    }

    fn str(&mut self, s: &str) {
        self.uint(s.len() as u32);
        self.0.extend_from_slice(s.as_bytes());
    }
}

struct Reader<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn uint(&mut self) -> Result<u32, String> {
        let mut v: u32 = 0;
        let mut shift = 0;
        loop {
            let byte = *self.bytes.get(self.pos).ok_or("truncated schema")?;
            self.pos += 1;
            v |= ((byte & 0x7f) as u32) << shift;
            if byte & 0x80 == 0 {
                return Ok(v);
            }
            shift += 7;
        }
    }

    fn str(&mut self) -> Result<&'a str, String> {
        let len = self.uint()? as usize;
        let end = self.pos + len;
        let s = self.bytes.get(self.pos..end).ok_or("truncated schema")?;
        self.pos = end;
        std::str::from_utf8(s).map_err(|e| e.to_string())
    }
}
