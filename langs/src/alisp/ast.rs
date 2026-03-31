use sova_core::vm::{Program, control_asm::ControlASM, variable::Variable};

pub const CONTEXT_REG : usize = 0;
pub const LIST_LEN_REG : usize = 1;

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
                let len = nodes.len() as i64;
                if len == 0 {
                    todo!()
                }
                prog.push(ControlASM::Mov(len.into(), Variable::reg(LIST_LEN_REG)).into());
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