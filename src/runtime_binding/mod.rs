use std::fmt;

use crate::runtime::exec;

impl fmt::Display for exec::Context {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Context:\n  {{ stack ptr: {}, jump ptr: {}, return_off: {}, return_len: {}, sub_call: {} }}\n",
            self.stack_ptr(), self.jump_ptr(), self.return_off(), self.return_len(), self.sub_call_ptr()
        )?;

        write!(
            f,
            "Memory:\n  {{ len: {}, cap: {} }}\n",
            self.memory_len(),
            self.memory_cap()
        )?;
        for i in 0..1 {
            let offset = (32 * i) as usize;
            let end = offset + 32;
            writeln!(
                f,
                "  {}: {}",
                i,
                self.memory()[offset..end]
                    .iter()
                    .take(32)
                    .fold(String::new(), |acc, x| acc.clone() + &format!("{:02X}", x))
            )?;
        }

        let mut stack_items = self.stack_ptr() + 3;
        if stack_items > 5 {
            stack_items = 5;
        }

        writeln!(f, "Stack:")?;
        for i in 0..stack_items {
            let offset = (32 * i) as usize;
            let end = offset + 32;
            writeln!(
                f,
                "  {}: {}",
                i,
                self.stack()[offset..end]
                    .iter()
                    .take(32)
                    .fold(String::new(), |acc, x| acc.clone() + &format!("{:02X}", x))
            )?;
        }

        match self.sub_ctx() {
            Some(sub_ctx) => {
                write!(f, "Sub Call:\n{}", sub_ctx)?;
            }
            None => {
                write!(f, "Sub Call: None\n")?;
            }
        }

        Ok(())
    }
}

impl fmt::Display for exec::ContractRun {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "ContractRun:\nResult: {:?}\n{}",
            self.result(),
            self.ctx()
        )
    }
}
