use wn::ast::{Expr, OpBin, OpUn, Stmt};

pub fn ast_to_dot(stmts: &[Stmt]) -> String {
    let mut graph = DotGraph::new();
    let root = graph.add_node("Program");

    for stmt in stmts {
        let stmt_id = graph.stmt(stmt);
        graph.add_edge(root, stmt_id, None);
    }

    graph.finish()
}

struct DotGraph {
    next_id: usize,
    nodes: Vec<String>,
    edges: Vec<String>,
}

impl DotGraph {
    fn new() -> Self {
        Self {
            next_id: 0,
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    fn finish(self) -> String {
        let mut out = String::from("digraph wn_ast {\n");
        out.push_str("  rankdir=TB;\n");
        out.push_str("  node [shape=box, fontname=\"Iosevka\"];\n");
        out.push_str("  edge [fontname=\"Iosevka\"];\n");

        for node in self.nodes {
            out.push_str(&node);
            out.push('\n');
        }
        for edge in self.edges {
            out.push_str(&edge);
            out.push('\n');
        }

        out.push_str("}\n");
        out
    }

    fn add_node(&mut self, label: impl AsRef<str>) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        self.nodes.push(format!(
            "  n{id} [label=\"{}\"];",
            escape_dot(label.as_ref())
        ));
        id
    }

    fn add_edge(&mut self, from: usize, to: usize, label: Option<&str>) {
        match label {
            Some(label) => self.edges.push(format!(
                "  n{from} -> n{to} [label=\"{}\"];",
                escape_dot(label)
            )),
            None => self.edges.push(format!("  n{from} -> n{to};")),
        }
    }

    fn stmt(&mut self, stmt: &Stmt) -> usize {
        match stmt {
            Stmt::Expresion { expr, .. } => {
                let node = self.add_node("ExprStmt");
                let expr_id = self.expr(expr);
                self.add_edge(node, expr_id, Some("expr"));
                node
            }
            Stmt::DeclWea {
                nombre,
                valor,
                es_duro,
                ..
            } => {
                let kind = if *es_duro { "DeclDuro" } else { "DeclWea" };
                let node = self.add_node(format!("{kind}({nombre})"));
                let value_id = self.expr(valor);
                self.add_edge(node, value_id, Some("value"));
                node
            }
            Stmt::DeclPega {
                nombre,
                params,
                cuerpo,
                ..
            } => {
                let node = self.add_node(format!("DeclPega({nombre})"));
                let params_id = self.add_node("params");
                self.add_edge(node, params_id, Some("params"));
                for param in params {
                    let param_id = self.add_node(format!("param:{param}"));
                    self.add_edge(params_id, param_id, None);
                }

                let body_id = self.add_node("body");
                self.add_edge(node, body_id, Some("body"));
                for stmt in cuerpo {
                    let stmt_id = self.stmt(stmt);
                    self.add_edge(body_id, stmt_id, None);
                }
                node
            }
            Stmt::Cachai {
                cond,
                entonces,
                si_no,
                ..
            } => {
                let node = self.add_node("Cachai");
                let cond_id = self.expr(cond);
                self.add_edge(node, cond_id, Some("cond"));

                let then_id = self.add_node("then");
                self.add_edge(node, then_id, Some("then"));
                for stmt in entonces {
                    let stmt_id = self.stmt(stmt);
                    self.add_edge(then_id, stmt_id, None);
                }

                if let Some(si_no) = si_no {
                    let else_id = self.add_node("else");
                    self.add_edge(node, else_id, Some("else"));
                    for stmt in si_no {
                        let stmt_id = self.stmt(stmt);
                        self.add_edge(else_id, stmt_id, None);
                    }
                }
                node
            }
            Stmt::Mientras { cond, cuerpo, .. } => {
                let node = self.add_node("Mientras");
                let cond_id = self.expr(cond);
                self.add_edge(node, cond_id, Some("cond"));
                let body_id = self.add_node("body");
                self.add_edge(node, body_id, Some("body"));
                for stmt in cuerpo {
                    let stmt_id = self.stmt(stmt);
                    self.add_edge(body_id, stmt_id, None);
                }
                node
            }
            Stmt::Para {
                var,
                iterable,
                cuerpo,
                ..
            } => {
                let node = self.add_node(format!("Para({var})"));
                let iterable_id = self.expr(iterable);
                self.add_edge(node, iterable_id, Some("iterable"));
                let body_id = self.add_node("body");
                self.add_edge(node, body_id, Some("body"));
                for stmt in cuerpo {
                    let stmt_id = self.stmt(stmt);
                    self.add_edge(body_id, stmt_id, None);
                }
                node
            }
            Stmt::Devolver { valor, .. } => {
                let node = self.add_node("Devolver");
                let value_id = self.expr(valor);
                self.add_edge(node, value_id, Some("value"));
                node
            }
            Stmt::Ojo {
                cuerpo,
                error_var,
                manejo,
                ..
            } => {
                let node = self.add_node(format!("Ojo({error_var})"));
                let body_id = self.add_node("try");
                self.add_edge(node, body_id, Some("try"));
                for stmt in cuerpo {
                    let stmt_id = self.stmt(stmt);
                    self.add_edge(body_id, stmt_id, None);
                }

                let catch_id = self.add_node("catch");
                self.add_edge(node, catch_id, Some("catch"));
                for stmt in manejo {
                    let stmt_id = self.stmt(stmt);
                    self.add_edge(catch_id, stmt_id, None);
                }
                node
            }
            Stmt::Cortala(_) => self.add_node("Cortala"),
            Stmt::Sigue(_) => self.add_node("Sigue"),
        }
    }

    fn expr(&mut self, expr: &Expr) -> usize {
        match expr {
            Expr::Numero(n, _) => self.add_node(format!("Numero({n})")),
            Expr::Texto(text, _) => self.add_node(format!("Texto({text:?})")),
            Expr::Booleano(value, _) => self.add_node(format!("Booleano({value})")),
            Expr::Nada(_) => self.add_node("Nada"),
            Expr::Ident(name, _) => self.add_node(format!("Ident({name})")),
            Expr::Binario { izq, op, der, .. } => {
                let node = self.add_node(format!("Binario({})", op_bin_label(op)));
                let left = self.expr(izq);
                let right = self.expr(der);
                self.add_edge(node, left, Some("left"));
                self.add_edge(node, right, Some("right"));
                node
            }
            Expr::Unario { op, expr, .. } => {
                let node = self.add_node(format!("Unario({})", op_un_label(op)));
                let expr_id = self.expr(expr);
                self.add_edge(node, expr_id, Some("expr"));
                node
            }
            Expr::Llamada { callee, args, .. } => {
                let node = self.add_node("Llamada");
                let callee_id = self.expr(callee);
                self.add_edge(node, callee_id, Some("callee"));
                for (idx, arg) in args.iter().enumerate() {
                    let arg_id = self.expr(arg);
                    let label = format!("arg[{idx}]");
                    self.add_edge(node, arg_id, Some(&label));
                }
                node
            }
            Expr::Indice { objeto, indice, .. } => {
                let node = self.add_node("Indice");
                let object_id = self.expr(objeto);
                let index_id = self.expr(indice);
                self.add_edge(node, object_id, Some("object"));
                self.add_edge(node, index_id, Some("index"));
                node
            }
            Expr::Lista(items, _) => {
                let node = self.add_node("Lista");
                for (idx, item) in items.iter().enumerate() {
                    let item_id = self.expr(item);
                    let label = format!("item[{idx}]");
                    self.add_edge(node, item_id, Some(&label));
                }
                node
            }
            Expr::Mapa(entries, _) => {
                let node = self.add_node("Mapa");
                for (idx, (key, value)) in entries.iter().enumerate() {
                    let pair_id = self.add_node(format!("pair[{idx}]"));
                    self.add_edge(node, pair_id, None);
                    let key_id = self.expr(key);
                    let value_id = self.expr(value);
                    self.add_edge(pair_id, key_id, Some("key"));
                    self.add_edge(pair_id, value_id, Some("value"));
                }
                node
            }
            Expr::Asignacion { nombre, valor, .. } => {
                let node = self.add_node(format!("Asignacion({nombre})"));
                let value_id = self.expr(valor);
                self.add_edge(node, value_id, Some("value"));
                node
            }
        }
    }
}

fn op_bin_label(op: &OpBin) -> &'static str {
    match op {
        OpBin::Suma => "+",
        OpBin::Resta => "-",
        OpBin::Mul => "*",
        OpBin::Div => "/",
        OpBin::Mod => "%",
        OpBin::Eq => "==",
        OpBin::Neq => "!=",
        OpBin::Lt => "<",
        OpBin::Gt => ">",
        OpBin::Lte => "<=",
        OpBin::Gte => ">=",
        OpBin::Y => "y",
        OpBin::O => "o",
    }
}

fn op_un_label(op: &OpUn) -> &'static str {
    match op {
        OpUn::No => "no",
        OpUn::Neg => "-",
    }
}

fn escape_dot(text: &str) -> String {
    text.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use wn::{lexer::tokenizar, parser::parsear};

    use super::ast_to_dot;

    fn parse(src: &str) -> Vec<wn::ast::Stmt> {
        let tokens = tokenizar(src).unwrap();
        parsear(tokens, src, "<test>").unwrap()
    }

    #[test]
    fn ast_to_dot_emits_graphviz_header() {
        let dot = ast_to_dot(&parse("wea x = 1 + 2"));
        assert!(dot.starts_with("digraph wn_ast {\n"));
        assert!(dot.contains("Program"));
        assert!(dot.contains("DeclWea(x)"));
        assert!(dot.contains("Binario(+)"));
    }

    #[test]
    fn ast_to_dot_includes_control_flow_nodes() {
        let dot = ast_to_dot(&parse(
            "cachai (verdad) { devolver 1 } si no { devolver 2 }",
        ));
        assert!(dot.contains("Cachai"));
        assert!(dot.contains("[label=\"cond\"]"));
        assert!(dot.contains("then"));
        assert!(dot.contains("else"));
        assert!(dot.contains("Devolver"));
    }

    #[test]
    fn ast_to_dot_escapes_string_literals() {
        let dot = ast_to_dot(&parse("wea x = \"hola \\\"wn\\\"\""));
        assert!(dot.contains("Texto(\\\"hola \\\\\\\"wn\\\\\\\"\\\")"));
    }
}
