pub mod deconjugator;
pub mod dictionary;
pub mod processor;

pub use deconjugator::JapaneseDeconjugator;
pub use dictionary::{JMdict, JMdictEntry};
pub use processor::JapaneseProcessor;
