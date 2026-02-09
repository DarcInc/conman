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

use std::collections::HashMap;
use crate::jls::parameters::Parameters;
use crate::jls::command::JlsCommand;

pub struct Configuration {
    pub directives : HashMap<String, Parameters>,
}

impl std::ops::Index<&str> for Configuration {
    type Output = Parameters;

    fn index(&self, name: &str) -> &Self::Output {
        self.directives.get(name).unwrap()
    }
}

impl Configuration {
    pub fn new(data : Vec<Parameters>) -> Configuration {
        let mut directives : HashMap<String, Parameters> = HashMap::new();
        for parameter in data {
            directives.insert(parameter.name(), parameter.clone());
        }

        Configuration {
            directives,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_jail() {
        let line = "devfs_ruleset=5 nodying enforce_statfs=2 env=\"\" host=new ip4=disable";
        let jls = JlsCommand::new();
        let tokenized = jls.tokenize_jls_line(line).unwrap();
        let parameters = jls.convert_to_parameter_list(&tokenized).unwrap();

        let jail = Configuration::new(parameters);
        assert_eq!(jail.directives.len(), 6);
    }
}