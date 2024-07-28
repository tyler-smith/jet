use log::info;
use syntect::{
    easy::HighlightLines,
    highlighting::{Color, Style, ThemeSet},
    parsing::SyntaxSet,
    util::{as_24_bit_terminal_escaped, LinesWithEndings},
};

use jet_runtime::exec;

use crate::builder::{contract, env::Env, Error};

pub struct Builder<'ctx> {
    env: Env<'ctx>,
}

impl<'ctx> Builder<'ctx> {
    pub fn new(env: Env<'ctx>) -> Self {
        Self { env }
    }

    pub fn env(&self) -> &Env<'ctx> {
        &self.env
    }

    pub fn add_contract_function(&self, addr: &str, rom: &[u8]) -> Result<(), Error> {
        let fn_name = exec::mangle_contract_fn(addr);
        info!("Building ROM into function {}", fn_name);

        contract::build(&self.env, &fn_name, rom)?;

        if self.env.opts().emit_llvm() {
            self.print_ir();
        }

        if self.env.opts().verify() {
            if !self.verify_contract(addr) {
                return Err(Error::Verify);
            }
            self.env.module().verify()?;
        }
        Ok(())
    }

    fn verify_contract(&self, addr: &str) -> bool {
        let func_name = exec::mangle_contract_fn(addr);
        let func = self.env.module().get_function(&func_name).unwrap();
        func.verify(true)
    }

    fn print_ir(&self) {
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

        let s = self.env.module().print_to_string().to_string();

        println!();
        for line in LinesWithEndings::from(s.as_str()) {
            let ranges: Vec<(Style, &str)> = h.highlight_line(line, &ps).unwrap();
            let escaped = as_24_bit_terminal_escaped(&ranges[..], true);
            print!("    {}", escaped);
        }
        println!();
    }
}
