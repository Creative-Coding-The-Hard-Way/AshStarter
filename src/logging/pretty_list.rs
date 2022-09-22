use std::fmt;

/// Wrapper for &[T] which can be pretty-printed as a markdown-style list.
/// Nice for multiline logs which include lists.
pub struct PrettyList<'data, T>(pub &'data [T]);

impl<'data, T> fmt::Debug for PrettyList<'data, T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("\n")?;
        for entry in self.0 {
            if f.alternate() {
                f.write_fmt(format_args!("- {:#?}\n", entry))?;
            } else {
                f.write_fmt(format_args!("- {:?}\n", entry))?;
            }
        }
        Ok(())
    }
}

impl<'data, T> fmt::Display for PrettyList<'data, T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("\n")?;
        for entry in self.0 {
            f.write_fmt(format_args!("- {}\n", entry))?;
        }
        Ok(())
    }
}
