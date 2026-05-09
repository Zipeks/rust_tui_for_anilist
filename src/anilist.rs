use graphql_client::{GraphQLQuery, Response};
use reqwest::{Client, header};
use std::error::Error;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.json",
    query_path = "qraphql/get_anime.graphql",
    response_derives = "Debug"
)]

pub struct GetAnime;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.json",
    query_path = "qraphql/get_viewer.graphql",
    response_derives = "Debug"
)]

pub struct GetViewer;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.json",
    query_path = "qraphql/get_basic_viewer.graphql",
    response_derives = "Debug"
)]
pub struct GetBasicViewer;
pub struct AnilistClient {
    http_client: Client,
    api_url: &'static str,
}
impl AnilistClient {
    pub fn new(access_token: Option<&str>) -> Result<Self, Box<dyn Error>> {
        let mut headers = header::HeaderMap::new();
        if let Some(token) = access_token {
            let auth_value = format!("Bearer {}", token);
            let mut header_value = header::HeaderValue::from_str(&auth_value)?;

            header_value.set_sensitive(true);
            headers.insert(header::AUTHORIZATION, header_value);
        }
        let client = Client::builder().default_headers(headers).build()?;
        Ok(Self {
            http_client: client,
            api_url: "https://graphql.anilist.co",
        })
    }
    pub async fn get_viewer(&self) -> Result<get_viewer::ResponseData, Box<dyn Error>> {
        let variables = get_viewer::Variables;
        let request_body = GetViewer::build_query(variables);

        let res = self
            .http_client
            .post(self.api_url)
            .json(&request_body)
            .send()
            .await?;

        let response_body: Response<get_viewer::ResponseData> = res.json().await?;

        if let Some(errors) = response_body.errors {
            return Err(format!("Błędy GraphQL: {:?}", errors).into());
        }

        response_body
            .data
            .ok_or_else(|| "Brak danych w odpowiedzi".into())
    }

    pub async fn get_basic_viewer(&self) -> Result<get_basic_viewer::ResponseData, Box<dyn Error>> {
        let variables = get_basic_viewer::Variables;
        let request_body = GetBasicViewer::build_query(variables);

        let res = self
            .http_client
            .post(self.api_url)
            .json(&request_body)
            .send()
            .await?;

        let response_body: Response<get_basic_viewer::ResponseData> = res.json().await?;

        if let Some(errors) = response_body.errors {
            return Err(format!("Błędy GraphQL: {:?}", errors).into());
        }

        response_body
            .data
            .ok_or_else(|| "Brak danych w odpowiedzi".into())
    }
    
    pub async fn get_anime(&self, id: i64) -> Result<get_anime::ResponseData, Box<dyn Error>> {
        let variables = get_anime::Variables { id };
        let request_body = GetAnime::build_query(variables);

        let res = self
            .http_client
            .post(self.api_url)
            .json(&request_body)
            .send()
            .await?;

        let response_body: Response<get_anime::ResponseData> = res.json().await?;

        if let Some(errors) = response_body.errors {
            return Err(format!("Błędy GraphQL: {:?}", errors).into());
        }

        response_body
            .data
            .ok_or_else(|| "Brak danych w odpowiedzi".into())
    }
}
