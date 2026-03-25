use std::fmt::{self, Write};

pub struct PrettyPrinter<W> {
    out: W,
    indent: usize,
    in_middle_of_list: bool,
}

impl<W: Write> PrettyPrinter<W> {
    pub fn new(out: W) -> Self {
        Self { out, indent: 0, in_middle_of_list: false }
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

    pub fn add_field_with(
        &mut self,
        name: &str,
        then: impl FnOnce(&mut Self) -> fmt::Result,
    ) -> fmt::Result {
        self.line()?;
        write!(self, "{}: ", name)?;
        then(self)?;
        write!(self, ",")
    }

    pub fn add_field(&mut self, name: &str, value: impl fmt::Debug) -> fmt::Result {
        self.add_field_with(name, |p| write!(p, "{:?}", value))
    }

    pub fn add_comment(&mut self, comment: &str) -> fmt::Result {
        write!(self, " /* {} */", comment)
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

    pub fn add_fn_call(
        &mut self,
        fn_name: &str,
        then: impl FnOnce(&mut Self) -> fmt::Result,
    ) -> fmt::Result {
        write!(self, "{}(", fn_name)?;
        let prev_in_middle_of_list = self.in_middle_of_list;
        self.in_middle_of_list = false;
        then(self)?;
        self.in_middle_of_list = prev_in_middle_of_list;
        write!(self, ")")?;
        Ok(())
    }

    pub fn add_fn_arg_with(&mut self, then: impl FnOnce(&mut Self) -> fmt::Result) -> fmt::Result {
        if self.in_middle_of_list {
            write!(self, ", ")?;
        }
        self.in_middle_of_list = true;
        then(self)?;
        Ok(())
    }

    pub fn add_fn_arg<T: fmt::Debug>(&mut self, arg: T) -> fmt::Result {
        self.add_fn_arg_with(|p| write!(p, "{:?}", arg))
    }
}

impl<W: Write> Write for PrettyPrinter<W> {
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
