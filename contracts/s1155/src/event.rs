use cosmwasm_std::{attr, Uint128};
use s_std::Response;

pub struct TransferEvent<'a> {
    pub from: Option<&'a str>,
    pub to: Option<&'a str>,
    pub token_id: &'a str,
    pub amount: Uint128,
}

pub trait Event {
    /// Append attributes to response
    fn add_attributes(&self, response: &mut Response);
}

impl<'a> Event for TransferEvent<'a> {
    fn add_attributes(&self, rsp: &mut Response) {
        rsp.attributes.push(attr("action", "transfer"));
        rsp.attributes.push(attr("token_id", self.token_id));
        rsp.attributes.push(attr("amount", self.amount));
        if let Some(from) = self.from {
            rsp.attributes.push(attr("from", from.to_string()));
        }
        if let Some(to) = self.to {
            rsp.attributes.push(attr("to", to.to_string()));
        }
    }
}
