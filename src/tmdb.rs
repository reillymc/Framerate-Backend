use std::{collections::HashMap, env};

use http_cache_reqwest::{CACacheManager, Cache, CacheMode, HttpCache, HttpCacheOptions};
use reqwest::{header, Client};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};

pub type TmdbClient = ClientWithMiddleware;

pub fn generate_endpoint(path: String, query: Option<HashMap<&str, &str>>) -> String {
    let mut query = query.unwrap_or(HashMap::<&str, &str>::new());

    query.insert("language", "en-AU");
    query.insert("include_adult", "false");

    let mut query_params = query
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<String>>();

    query_params.sort_by(|a, b| a.cmp(b));

    let query_params = query_params.join("&");

    format!("https://api.themoviedb.org/3/{path}?{query_params}")
}

pub fn get_client(enable_cache: bool) -> TmdbClient {
    let tmdb_api_key = env::var("TMDB_API_KEY").expect("TMDB_API_KEY must be set");

    let mut headers = header::HeaderMap::new();
    let auth_value = header::HeaderValue::from_str(&format!("Bearer {tmdb_api_key}"))
        .expect("Unable to set up reqwest");
    headers.insert(header::AUTHORIZATION, auth_value);

    if enable_cache {
        return ClientBuilder::new(
            Client::builder()
                .default_headers(headers)
                .use_rustls_tls()
                .build()
                .expect("Unable to set up reqwest"),
        )
        .with(Cache(HttpCache {
            mode: CacheMode::Default,
            manager: CACacheManager {
                path: "./.reqwest-cache".into(),
            },
            options: HttpCacheOptions::default(),
        }))
        .build();
    }

    ClientBuilder::new(
        Client::builder()
            .default_headers(headers)
            .use_rustls_tls()
            .build()
            .expect("Unable to set up reqwest"),
    )
    .build()
}
