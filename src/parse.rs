use regex::Regex;
use std::fmt;

const FUNC_REGEX: &str = r"([^\d\W]\w*)\s+([^\d\W]\w*)\s*\(([^()]*)\)\s*\{";

pub struct Function {
    pub returns: String,
    pub name: String,
    pub params: String,
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}({});", self.returns, self.name, self.params)
    }
}

pub fn list_functions(src: String) -> Vec<Function> {
    let re = Regex::new(FUNC_REGEX).unwrap();
    re.captures_iter(&src)
        .map(|cap| Function {
            returns: cap[1].to_string(),
            name: cap[2].to_string(),
            params: cap[3].to_string(),
        })
        .collect()
}
