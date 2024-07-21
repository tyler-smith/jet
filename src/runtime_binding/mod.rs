// use std::fmt;
//
// use colored::Colorize;
//
// use jet_runtime::{BlockInfo, Context, ContractRun};
//
// pub struct WrappedContext<'a>(&'a Context);
// pub struct WrappedBlockInfo<'a>(&'a BlockInfo);
// pub struct WrappedContractRun<'a>(&'a ContractRun);
//
// impl<'a> WrappedContext<'a> {
//     pub fn new(ctx: &'a Context) -> Self {
//         WrappedContext(ctx)
//     }
// }
//
// impl<'a> WrappedBlockInfo<'a> {
//     pub fn new(block_info: &'a BlockInfo) -> Self {
//         WrappedBlockInfo(block_info)
//     }
// }
//
// impl<'a> WrappedContractRun<'a> {
//     pub fn new(contract_run: &'a ContractRun) -> Self {
//         WrappedContractRun(contract_run)
//     }
// }
//
// impl fmt::Display for WrappedContext<'_> {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(
//             f,
//             "Context:\n  {{ stack ptr: {}, jump ptr: {}, return_off: {}, return_len: {},
// sub_call: {} }}\n",             self.0.stack_ptr(), self.0.jump_ptr(), self.0.return_off(),
// self.0.return_len(), self.0.sub_call_ptr()         )?;
//
//         write!(
//             f,
//             "Memory:\n  {{ len: {}, cap: {} }}\n",
//             self.0.memory_len(),
//             self.0.memory_cap()
//         )?;
//         for i in 0..1 {
//             let offset = (32 * i) as usize;
//             let end = offset + 32;
//             writeln!(
//                 f,
//                 "  {}: {}",
//                 i,
//                 self.0.memory()[offset..end]
//                     .iter()
//                     .take(32)
//                     .fold(String::new(), |acc, x| acc.clone() + &format!("{:02X}", x))
//             )?;
//         }
//
//         writeln!(f, "Stack:")?;
//         let stack = self.0.stack();
//         let stack_size = self.0.stack_ptr() as usize;
//
//         // Print out each 32 byte word, starting from the top of the stack and working down
//         for i in 0..stack_size {
//             let word_idx = stack_size - i - 1;
//             let offset = (32 * word_idx) as usize;
//             let end = offset + 32;
//
//             let byte_formatter = |acc: (String, bool), x: &u8| {
//                 let has_been_significant = acc.1;
//                 let is_significant = has_been_significant || *x != 0;
//
//                 let mut byte_str = format!("{:02X}", x);
//
//                 if is_significant {
//                     byte_str = if !has_been_significant && x < &16 {
//                         let chars = byte_str.chars();
//                         format!(
//                             "{}{}",
//                             chars.clone().nth(0).unwrap().to_string(),
//                             chars.clone().nth(1).unwrap().to_string().blue()
//                         )
//                     } else {
//                         byte_str.blue().to_string()
//                     };
//                 }
//
//                 (acc.0.clone() + &byte_str, is_significant)
//             };
//
//             writeln!(
//                 f,
//                 "  {}: {}",
//                 word_idx + 1,
//                 stack[offset..end]
//                     .iter()
//                     .take(32)
//                     .rev()
//                     .fold((String::new(), false), byte_formatter)
//                     .0
//             )?;
//         }
//
//         // match self.0.sub_ctx() {
//         //     Some(sub_ctx) => {
//         //         writeln!(f, "Sub Call:\n{}", sub_ctx)?;
//         //     }
//         //     None => {
//         //         writeln!(f, "Sub Call: None")?;
//         //     }
//         // }
//
//         Ok(())
//     }
// }
//
// impl fmt::Display for WrappedBlockInfo<'_> {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(
//             f,
//             "BlockInfo:\n  {{ number: {}, difficulty: {}, gas_limit: {}, timestamp: {}, base_fee:
// {}, blob_base_fee: {}, chain_id: {}, hash: {}, coinbase: {} }}\n",             self.0.number(),
//             self.0.difficulty(),
//             self.0.gas_limit(),
//             self.0.timestamp(),
//             self.0.base_fee(),
//             self.0.blob_base_fee(),
//             self.0.chain_id(),
//             self.0.hash().iter().take(32).fold(String::new(), |acc, x| acc.clone() +
// &format!("{:02X}", x)),             self.0.coinbase().iter().take(20).fold(String::new(), |acc,
// x| acc.clone() + &format!("{:02X}", x))         )
//     }
// }
//
// impl fmt::Display for WrappedContractRun<'_> {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(
//             f,
//             "ContractRun:\nResult: {:?}\n{}",
//             self.0.result(),
//             WrappedContext(self.0.ctx())
//         )
//     }
// }
