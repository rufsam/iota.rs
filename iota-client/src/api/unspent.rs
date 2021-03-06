// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{Client, Error, Result};

use bee_message::prelude::Address;
use bee_signing_ext::{binary::BIP32Path, Seed};

/// Builder of get_unspent_address API
pub struct GetUnspentAddressBuilder<'a> {
    client: &'a Client,
    seed: &'a Seed,
    path: Option<&'a BIP32Path>,
    index: Option<usize>,
}

impl<'a> GetUnspentAddressBuilder<'a> {
    /// Create get_unspent_address builder
    pub fn new(client: &'a Client, seed: &'a Seed) -> Self {
        Self {
            client,
            seed,
            path: None,
            index: None,
        }
    }

    /// Set path to the builder
    pub fn path(mut self, path: &'a BIP32Path) -> Self {
        self.path = Some(path);
        self
    }

    /// Set index to the builder
    pub fn index(mut self, index: usize) -> Self {
        self.index = Some(index);
        self
    }

    /// Consume the builder and get the API result
    pub async fn get(self) -> Result<(Address, usize)> {
        let path = match self.path {
            Some(p) => p,
            None => return Err(Error::MissingParameter(String::from("BIP32 path"))),
        };

        let mut index = match self.index {
            Some(r) => r,
            None => 0,
        };

        let result = loop {
            let addresses = self
                .client
                .find_addresses(self.seed)
                .path(path)
                .range(index..index + 20)
                .get()?;

            // TODO we assume all addressees are unspent and valid if balance > 0
            let mut address = None;
            for a in addresses {
                let address_balance = self.client.get_address().balance(&a).await?;
                match address_balance {
                    0 => {
                        address = Some(a);
                        break;
                    }
                    _ => index += 1,
                }
            }

            if let Some(a) = address {
                break (a, index);
            }
        };

        Ok(result)
    }
}
