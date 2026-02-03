//!
//! Copyright (c) 2026, Paul C. Hoehne
//!
//! Redistribution and use in source and binary forms, with or without modification, are
//! permitted provided that the following conditions are met:
//!
//!   Redistributions of source code must retain the above copyright notice, this list of
//!   conditions and the following disclaimer.
//!
//!   Redistributions in binary form must reproduce the above copyright notice, this list of
//!   conditions and the following disclaimer in the documentation and/or other materials
//!   provided with the distribution.
//!
//! THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY
//! EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF
//! MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL
//! THE COPYRIGHT OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
//! SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT
//! OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION)
//! HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
//! OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
//! SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
//!

/// Represents a configuration item, or a declaration inside the container definition.
///
/// * `raw` - The raw text encountered during tokenization
///
#[derive(Debug, Clone, PartialEq)]
pub struct ConfigItem {
    pub raw: String,
}

impl ConfigItem {

    /// Creates a new ConfigItem
    ///
    /// * `raw` - The raw string for the item
    pub fn new(raw: &str) -> Self {
        Self {
            raw: raw.to_string(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn new_test() {
        let item = ConfigItem::new("foo");
        assert_eq!("foo".to_string(), item.raw);
    }
}