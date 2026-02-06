use std::fmt::{self, Write};

pub struct PrettyPrinter {
    out: String,
    indent: usize,
}

impl PrettyPrinter {
    pub fn new() -> Self {
        Self { out: String::new(), indent: 0 }
    }

    pub fn finish(self) -> String {
        self.out
    }

    /// Starts a new line with indentation.
    pub fn line(&mut self) -> fmt::Result {
        const INDENT: &str = "    ";
        write!(self.out, "\n{}", INDENT.repeat(self.indent))
    }

    pub fn indented(&mut self, then: impl FnOnce(&mut Self) -> fmt::Result) -> fmt::Result {
        self.indent += 1;
        then(self)?;
        self.indent -= 1;
        Ok(())
    }

    pub fn add_struct(
        &mut self,
        name: &str,
        then: impl FnOnce(&mut Self) -> fmt::Result,
    ) -> fmt::Result {
        write!(self, "{} {{", name)?;
        self.indented(then)?;
        self.line()?;
        write!(self, "}}")
    }

    pub fn add_field(
        &mut self,
        name: &str,
        then: impl FnOnce(&mut Self) -> fmt::Result,
    ) -> fmt::Result {
        self.line()?;
        write!(self, "{}: ", name)?;
        then(self)?;
        write!(self, ",")
    }

    pub fn add_map<K: Copy, V>(
        &mut self,
        kv_pairs: impl Iterator<Item = (K, V)>,
        mut fmt_key: impl FnMut(&mut Self, K) -> fmt::Result,
        mut fmt_value: impl FnMut(&mut Self, (K, V)) -> fmt::Result,
    ) -> fmt::Result {
        write!(self, "{{")?;
        self.indented(|p| {
            for (key, value) in kv_pairs {
                p.line()?;
                fmt_key(p, key)?;
                write!(p, ": ")?;
                fmt_value(p, (key, value))?;
                write!(p, ",")?;
            }
            Ok(())
        })?;
        self.line()?;
        write!(self, "}}")
    }

    pub fn add_list<T>(
        &mut self,
        items: impl Iterator<Item = T>,
        mut fmt_item: impl FnMut(&mut Self, T) -> fmt::Result,
    ) -> fmt::Result {
        write!(self, "[")?;
        self.indented(|p| {
            for item in items {
                p.line()?;
                fmt_item(p, item)?;
                write!(p, ",")?;
            }
            Ok(())
        })?;
        self.line()?;
        write!(self, "]")
    }
}

impl<'a> Write for PrettyPrinter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let mut lines = s.split("\n");
        if let Some(first_line) = lines.next() {
            write!(self.out, "{}", first_line)?;
        }
        for line in lines {
            self.line()?;
            write!(self.out, "{}", line)?;
        }
        Ok(())
    }
}
