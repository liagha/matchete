pub mod utils;

pub mod exact;
pub mod fuzzy;
pub mod phonetic;
pub mod structural;
pub mod lexical;
pub mod proximity;
pub mod ensemble;

pub use exact::*;
pub use fuzzy::*;
pub use phonetic::*;
pub use structural::*;
pub use lexical::*;
pub use proximity::*;
pub use ensemble::*;