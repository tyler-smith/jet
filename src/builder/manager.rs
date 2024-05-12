use log::info;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Color, Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};

use crate::builder::contract_builder::ContractBuilder;
use crate::builder::environment::Env;
use crate::builder::errors::BuildError;

pub struct Manager<'ctx> {
    build_env: Env<'ctx>,
}

impl<'ctx> Manager<'ctx> {
    pub fn new(build_env: Env<'ctx>) -> Self {
        Self { build_env }
    }

    pub fn env(&self) -> &Env<'ctx> {
        &self.build_env
    }

    pub fn add_contract_function(&self, addr: &str, rom: &[u8]) -> Result<(), BuildError> {
        let func_name = crate::runtime::mangle_contract_fn(addr);
        info!("Building ROM into function {}", func_name);
        ContractBuilder::build(&self.build_env, &func_name, rom)?;

        if self.build_env.opts().emit_llvm() {
            let ts = ThemeSet::load_defaults();
            let ps = SyntaxSet::load_from_folder("contrib/syntaxes").unwrap();
            let syntax = ps.find_syntax_by_extension("ll").unwrap();

            let mut theme = ts.themes["base16-eighties.dark"].clone();
            theme.settings.background = Some(Color {
                r: 0,
                g: 0,
                b: 0,
                a: 0,
            });

            let mut h = HighlightLines::new(syntax, &theme);

            let s = self.build_env.module().print_to_string().to_string();

            println!("");
            for line in LinesWithEndings::from(s.as_str()) {
                let ranges: Vec<(Style, &str)> = h.highlight_line(line, &ps).unwrap();
                let escaped = as_24_bit_terminal_escaped(&ranges[..], true);
                print!("    {}", escaped);
            }
            println!("");
        }

        Ok(())
    }
}
