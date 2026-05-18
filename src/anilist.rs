use graphql_client::{GraphQLQuery, Response};
use reqwest::{Client, header};
use std::error::Error;

use crate::app_helper_structs::MediaType;

// crate::anilist::{get_user_media_list};
// crate::anilist::{get_media};

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.json",
    query_path = "qraphql/get_media_details.graphql",
    response_derives = "Debug"
)]

pub struct GetMediaDetails;

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

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.json",
    query_path = "qraphql/get_user_media_list.graphql",
    response_derives = "Debug, Clone"
)]
pub struct GetUserMediaList;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "schema.json",
    query_path = "qraphql/get_media.graphql",
    response_derives = "Debug, Clone"
)]
pub struct GetMedia;

#[derive(Clone)]
pub struct AnilistClient {
    http_client: Client,
    api_url: &'static str,
}

impl AnilistClient {
    pub fn new(access_token: Option<&str>) -> Result<Self, Box<dyn Error + Sync + Send>> {
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

    pub async fn get_viewer(
        &self,
    ) -> Result<get_viewer::ResponseData, Box<dyn Error + Sync + Send>> {
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
            return Err(format!("GraphQL Error: {:?}", errors).into());
        }

        response_body.data.ok_or_else(|| "No data".into())
    }

    pub async fn get_basic_viewer(
        &self,
    ) -> Result<get_basic_viewer::ResponseData, Box<dyn Error + Sync + Send>> {
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
            return Err(format!("GraphQL Error: {:?}", errors).into());
        }

        response_body.data.ok_or_else(|| "No data".into())
    }

    pub async fn get_user_media_list(
        &self,
        user_id: i64,
        status: Option<get_user_media_list::MediaListStatus>,
        sort: Option<Vec<get_user_media_list::MediaListSort>>,
        page: Option<i64>,
        per_page: Option<i64>,
        type_: get_user_media_list::MediaType,
    ) -> Result<get_user_media_list::ResponseData, Box<dyn std::error::Error + Sync + Send>> {
        let mapped_sort = sort.map(|s| s.into_iter().map(Some).collect());

        let variables = get_user_media_list::Variables {
            user_id: user_id,
            status: status,
            sort: mapped_sort,
            page: page,
            per_page: per_page,
            type_: type_,
        };

        let request_body = GetUserMediaList::build_query(variables);

        let res = self
            .http_client
            .post(self.api_url)
            .json(&request_body)
            .send()
            .await?;

        let response_body: graphql_client::Response<get_user_media_list::ResponseData> =
            res.json().await?;

        if let Some(errors) = response_body.errors {
            return Err(format!("GraphQL Error: {:?}", errors).into());
        }

        response_body.data.ok_or_else(|| "No data".into())
    }

    pub async fn get_media(
        &self,
        type_: get_media::MediaType,
        season: Option<get_media::MediaSeason>,
        season_year: Option<i64>,
        status: Option<Vec<get_media::MediaStatus>>,
        sort: Option<Vec<get_media::MediaSort>>,
        page: Option<i64>,
        per_page: Option<i64>,
        search: Option<String>,
        format: Option<get_media::MediaFormat>,
    ) -> Result<get_media::ResponseData, Box<dyn std::error::Error + Sync + Send>> {
        let mapped_sort = sort
            .filter(|s| !s.is_empty())
            .map(|s| s.into_iter().map(Some).collect());
        let mapped_status = status
            .filter(|s| !s.is_empty())
            .map(|s| s.into_iter().map(Some).collect());
        let clean_search = search.filter(|s| !s.trim().is_empty());

        let variables = get_media::Variables {
            season,
            season_year,
            status_in: mapped_status,
            sort: mapped_sort,
            page,
            per_page,
            type_: type_,
            search: clean_search,
            format,
        };

        let request_body = GetMedia::build_query(variables);

        let mut json_body = serde_json::to_value(&request_body)?;

        // Empty values brakes request for some reason
        if let Some(vars) = json_body
            .get_mut("variables")
            .and_then(|v| v.as_object_mut())
        {
            vars.retain(|_, v| !v.is_null());
        }

        let res = self
            .http_client
            .post(self.api_url)
            .json(&json_body)
            .send()
            .await?;

        let response_body: graphql_client::Response<get_media::ResponseData> = res.json().await?;

        if let Some(errors) = response_body.errors {
            return Err(format!("GraphQL Error: {:?}", errors).into());
        }

        response_body.data.ok_or_else(|| "No data".into())
    }

    pub async fn get_media_details(
        &self,
        media_id: i64,
        media_type: MediaType,
    ) -> Result<get_media_details::ResponseData, Box<dyn std::error::Error + Sync + Send>> {
        let variables = get_media_details::Variables {
            media_id: media_id,
            type_: media_type.to_get_media_details(),
            format: None,
        };

        let request_body = GetMediaDetails::build_query(variables);

        let res = self
            .http_client
            .post(self.api_url)
            .json(&request_body)
            .send()
            .await?;

        let response_body: graphql_client::Response<get_media_details::ResponseData> =
            res.json().await?;

        if let Some(errors) = response_body.errors {
            return Err(format!("GraphQL Error: {:?}", errors).into());
        }

        response_body.data.ok_or_else(|| "No data".into())
    }
}
