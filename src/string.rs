use std::ops::Deref;

pub trait StrExt {
    fn title_case(&self) -> String;
}

impl<T: Deref<Target = str>> StrExt for T {
    fn title_case(&self) -> String {
        let mut up_next = true;
        let mut out = String::new();
        for c in self.chars() {
            if up_next {
                for u in c.to_uppercase() {
                    out.push(u);
                }
            } else {
                out.push(c);
            }
            up_next = c.is_whitespace();
        }
        out
    }
}
