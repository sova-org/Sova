use sova_core::vm::Program;

#[derive(Debug, Default, Clone)]
pub enum ALispAtom {
    #[default]
    Nil,
    Int(i64),
    Float(f64),
    Str(String),
    Call(String),
}

impl ALispAtom {
    pub fn push(self, prog: &mut Program) {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub enum ALispAST {
    Atom(ALispAtom),
    List(Vec<ALispAST>),
}

impl ALispAST {

    pub fn push_expr(self, prog: &mut Program) {
        match self {
            ALispAST::Atom(alisp_atom) => {
                alisp_atom.push(prog);
            }
            ALispAST::List(nodes) => {
                if nodes.is_empty() {
                    todo!()
                }
                for node in nodes.into_iter().rev() {
                    node.push_expr(prog);
                }
            }
        }
    }

}

impl From<ALispAtom> for ALispAST {
    fn from(value: ALispAtom) -> Self {
        ALispAST::Atom(value)
    }
}