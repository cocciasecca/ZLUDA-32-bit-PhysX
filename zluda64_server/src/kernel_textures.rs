//! Diagnostic candidates only: absence of Tex is NOT proof of absent raw
//! descriptor access/coherence effects. Never use this classification to skip a barrier.
use ptx_parser::{Directive, Instruction as I, ParsedOperand, Statement};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Usage {
    NoTextureInstructions,
    TextureInstructions,
    Unknown,
}
#[derive(Default)]
struct Node {
    texture: bool,
    unknown: bool,
    calls: BTreeSet<String>,
}
fn scan(body: &[Statement<ParsedOperand<&str>>], node: &mut Node) {
    for statement in body {
        match statement {
            Statement::Block(body) => scan(body, node),
            Statement::Instruction(_, instruction) => match instruction {
                I::Tex { .. } => node.texture = true,
                I::Call { arguments, .. } => {
                    node.calls.insert(arguments.func.to_owned());
                }
                I::Abs { .. }
                | I::Add { .. }
                | I::Sub { .. }
                | I::Mul { .. }
                | I::Mad { .. }
                | I::Div { .. }
                | I::Fma { .. }
                | I::Min { .. }
                | I::Max { .. }
                | I::Mov { .. }
                | I::Ld { .. }
                | I::St { .. }
                | I::Cvt { .. }
                | I::Cvta { .. }
                | I::Bra { .. }
                | I::Ret { .. }
                | I::Exit { .. }
                | I::Set { .. }
                | I::Setp { .. }
                | I::SetBool { .. }
                | I::SetpBool { .. }
                | I::Selp { .. }
                | I::And { .. }
                | I::Or { .. }
                | I::Xor { .. }
                | I::Not { .. }
                | I::Shl { .. }
                | I::Shr { .. }
                | I::Neg { .. }
                | I::Rcp { .. }
                | I::Rsqrt { .. }
                | I::Sqrt { .. }
                | I::Sin { .. }
                | I::Cos { .. }
                | I::Lg2 { .. }
                | I::Ex2 { .. }
                | I::Rem { .. }
                | I::Mul24 { .. }
                | I::Atom { .. }
                | I::AtomCas { .. }
                | I::Bar { .. }
                | I::Membar { .. }
                | I::Vote { .. }
                | I::Popc { .. }
                | I::Clz { .. }
                | I::Bfe { .. }
                | I::Bfi { .. }
                | I::Brev { .. }
                | I::Prmt { .. } => (),
                _ => node.unknown = true,
            },
            Statement::Label(_) | Statement::Variable(_) => (),
        }
    }
}
fn resolve(
    name: &str,
    nodes: &BTreeMap<String, Node>,
    visiting: &mut BTreeSet<String>,
    depth: usize,
) -> Usage {
    if depth > 128 || !visiting.insert(name.to_owned()) {
        return Usage::Unknown;
    }
    let result = match nodes.get(name) {
        None => Usage::Unknown,
        Some(n) if n.texture => Usage::TextureInstructions,
        Some(n) => {
            let mut result = if n.unknown {
                Usage::Unknown
            } else {
                Usage::NoTextureInstructions
            };
            for callee in &n.calls {
                match resolve(callee, nodes, visiting, depth + 1) {
                    Usage::TextureInstructions => {
                        result = Usage::TextureInstructions;
                        break;
                    }
                    Usage::Unknown => result = Usage::Unknown,
                    Usage::NoTextureInstructions => (),
                }
            }
            result
        }
    };
    visiting.remove(name);
    result
}
pub fn analyze(image: &[u8]) -> BTreeMap<String, Usage> {
    if image.len() > 16 * 1024 * 1024 {
        return BTreeMap::new();
    }
    let image = image.strip_suffix(&[0]).unwrap_or(image);
    let Ok(text) = std::str::from_utf8(image) else {
        return BTreeMap::new();
    };
    let Ok(module) = ptx_parser::parse_module_checked(text) else {
        return BTreeMap::new();
    };
    let mut nodes = BTreeMap::new();
    let mut kernels = BTreeSet::new();
    for directive in module.directives {
        if let Directive::Method(linking, f) = directive {
            let name = f.func_directive.name().to_owned();
            if f.func_directive.name.is_kernel() {
                kernels.insert(name.clone());
            }
            let mut node = Node::default();
            if let Some(body) = f.body {
                scan(&body, &mut node)
            } else {
                node.unknown = true;
            }
            if linking.contains(ptx_parser::LinkingDirective::WEAK)
                || linking.contains(ptx_parser::LinkingDirective::EXTERN)
            {
                node.unknown = true;
            }
            if nodes.contains_key(&name) {
                node.unknown = true;
            }
            nodes.insert(name, node);
        }
    }
    kernels
        .into_iter()
        .map(|k| {
            let usage = resolve(&k, &nodes, &mut BTreeSet::new(), 0);
            (k, usage)
        })
        .collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parsed_leaf_and_malformed_input() {
        let text =
            b".version 6.0\n.target sm_30\n.address_size 32\n.visible .entry leaf() { ret; }\0";
        assert_eq!(
            analyze(text).get("leaf"),
            Some(&Usage::NoTextureInstructions)
        );
        assert!(analyze(b"not PTX").is_empty());
        assert!(analyze(&[255]).is_empty());
    }
    #[test]
    fn call_graph_propagates_texture_unknown_and_recursion() {
        let mut n = BTreeMap::new();
        n.insert("leaf".into(), Node::default());
        n.insert(
            "root".into(),
            Node {
                calls: BTreeSet::from(["leaf".into()]),
                ..Default::default()
            },
        );
        assert_eq!(
            resolve("root", &n, &mut BTreeSet::new(), 0),
            Usage::NoTextureInstructions
        );
        n.get_mut("leaf").unwrap().texture = true;
        assert_eq!(
            resolve("root", &n, &mut BTreeSet::new(), 0),
            Usage::TextureInstructions
        );
        n.remove("leaf");
        assert_eq!(resolve("root", &n, &mut BTreeSet::new(), 0), Usage::Unknown);
        n.get_mut("root").unwrap().calls = BTreeSet::from(["root".into()]);
        assert_eq!(resolve("root", &n, &mut BTreeSet::new(), 0), Usage::Unknown);
    }
    #[test]
    fn parsed_direct_call_and_unresolved_call() {
        let text=b".version 6.0\n.target sm_30\n.address_size 32\n.func helper() { ret; }\n.visible .entry root() { call helper, (); ret; }";
        assert_eq!(
            analyze(text).get("root"),
            Some(&Usage::NoTextureInstructions)
        );
        let text=b".version 6.0\n.target sm_30\n.address_size 32\n.extern .func helper();\n.visible .entry root() { call helper, (); ret; }";
        assert_eq!(analyze(text).get("root"), Some(&Usage::Unknown));
    }
}
