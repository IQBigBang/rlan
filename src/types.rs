
/* TODO
pub struct Inferrer {

}

impl Inferrer {
    pub fn new() -> Self {
        Inferrer {}
    }

    pub fn visit(&self, n: &mut Node) {
        match n {
            Node::Number(_) => self.noop(),
            Node::BinOp(lhs, op, rhs) => self.visit_binop(n),
            _ => unimplemented!()
        }
    }

    fn noop(&self) {} // for nodes where no type changing happens

    fn visit_binop(&self, lhs: &mut)
}
*/