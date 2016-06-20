use std::io::Read;
use std::cell::RefCell;

use hyper::Client;
use hyper::client::RequestBuilder;
use hyper::method::Method;
use hyper::header::{Headers, Authorization, Bearer};
use super::region::Region;
use super::auth::Token;

/// a commercetools client
pub struct CtpClient<'a> {
    api_url: &'a str,
    auth_url: &'a str,
    project_key: &'a str,
    client_id: &'a str,
    client_secret: &'a str,
    client: Client,
    token: RefCell<Option<Token>>,
}

impl<'a> CtpClient<'a> {
    /// Returns a commercetools client for the given arguments
    ///
    /// # Arguments
    ///
    /// * `region` - the world region the client should use
    /// * `project_key` - project key
    /// * `client_id` - client id
    /// * `client_secret` - client secret
    ///
    /// # Examples
    ///
    /// ```
    /// use commercetools::region::Region;
    /// use commercetools::client::CtpClient;
    ///
    /// let region = Region::Europe;
    /// let client = CtpClient::new(&region, "my project key", "my client id", "my client secret");
    /// ```
    pub fn new(region: &'a Region, project_key: &'a str, client_id: &'a str, client_secret: &'a str) -> CtpClient<'a> {
        CtpClient {
            api_url: region.api_url(),
            auth_url: region.auth_url(),
            project_key: project_key,
            client_id: client_id,
            client_secret: client_secret,
            client: Client::new(),
            token: RefCell::new(None),
        }
    }

    // TODO (YaSi): avoid cloning the String on each call
    pub fn get_token(&self) -> String {
        let mut cache = self.token.borrow_mut();
        if cache.is_some() {
            let token = cache.as_ref().unwrap();
            if token.is_valid() {
                return token.access_token.clone();
            }
        }

        let new_token = super::auth::retrieve_token(&self.client,
                                                    self.auth_url,
                                                    self.project_key,
                                                    self.client_id,
                                                    self.client_secret)
            .unwrap();

        *cache = Some(new_token.clone());
        new_token.access_token
    }

    pub fn get(&self, uri: &str) -> String {
        send(self.request(Method::Get, uri))
    }

    pub fn post(&self, uri: &str, body: &str) -> String {
        send(self.request(Method::Post, uri).body(body))
    }

    // TODO: this method "leaks" hyper RequestBuilder
    pub fn request(&self, method: Method, uri: &str) -> RequestBuilder {
        let client = &self.client;

        let access_token = self.get_token();

        let mut headers = Headers::new();
        headers.set(Authorization(Bearer { token: access_token }));

        let uri = format!("{}/{}{}", self.api_url, self.project_key, uri);
        client.request(method, &uri)
            .headers(headers)
    }
}

fn send(r: RequestBuilder) -> String {
    let mut projets_res = r.send()
        .unwrap();

    let mut body = String::new();
    projets_res.read_to_string(&mut body).unwrap();
    body
}
