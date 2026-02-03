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

use crate::parser::config_item::ConfigItem;

/// Configuration encapsulates the configuration of a container.  It is composed of a name
/// followed by zero or more directives.
#[derive(Debug, Default, Clone)]
pub struct Configuration {
    pub name : String,
    pub directives : Vec<ConfigItem>
}


impl Configuration {

    /// Set the container name.
    ///
    /// * `name` - The name of the container
    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    /// Add a directive to the container.
    ///
    /// * `directive` - A new directive to add.
    pub fn add_directive(&mut self, directive: &ConfigItem) {
        self.directives.push(directive.clone());
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_set_name() {
        let mut config = Configuration::default();
        config.set_name( "test");
        assert_eq!(config.name, "test");
    }

    #[test]
    fn test_add_directive() {
        let mut config = Configuration::default();
        assert_eq!(0, config.directives.len());
        config.add_directive(&ConfigItem::new("foo"));
        assert_eq!(1, config.directives.len());
        assert_eq!(ConfigItem::new("foo"), config.directives[0]);
    }
}