use std::mem;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum BoinxPosition {
    #[default]
    Undefined,
    This,
    At(usize, Box<BoinxPosition>),
    Parallel(Vec<BoinxPosition>),
}

impl BoinxPosition {

    pub fn diff(self, other: &BoinxPosition) -> BoinxPosition {
        if self == *other {
            return BoinxPosition::Undefined;
        }
        if mem::discriminant(&self) != mem::discriminant(other) {
            return other.clone();
        } 
        match (self, other) {
            (BoinxPosition::This, BoinxPosition::This) |
            (BoinxPosition::Undefined, BoinxPosition::Undefined) 
                => BoinxPosition::Undefined,
            (BoinxPosition::At(i, p1), BoinxPosition::At(j, p2)) => {
                if i != *j {
                    return other.clone();
                }
                let pos = p1.diff(p2);
                if matches!(pos, BoinxPosition::Undefined) {
                    BoinxPosition::Undefined
                } else {
                    BoinxPosition::At(i, Box::new(pos))
                }
            }
            (BoinxPosition::Parallel(p1), BoinxPosition::Parallel(p2)) => {
                let mut seen_defined = false;
                let mut pos = Vec::new();
                for (i, inner1) in p1.into_iter().enumerate() {
                    let d = inner1.diff(&p2[i]);
                    seen_defined |= !matches!(d, BoinxPosition::Undefined);
                    pos.push(d);
                }
                if pos.len() < p2.len() {
                    pos.extend_from_slice(&p2[pos.len()..]);
                    seen_defined = true;
                }
                if !seen_defined {
                    BoinxPosition::Undefined
                } else {
                    BoinxPosition::Parallel(pos)
                }
            },
            _ => unreachable!()
        }
    }

}