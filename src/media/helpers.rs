use chrono::NaiveDate;

use crate::utils::response_body::Error;

pub struct MediaDetails {
    pub imdb_id: Option<String>,
    pub media_title: String,
    pub media_poster_uri: Option<String>,
    pub media_release_date: Option<NaiveDate>,
}

pub async fn get_details(media_type: &str, media_id: i32) -> Result<MediaDetails, Error> {
    match media_type {
        "movie" => {
            let Ok(movie) = crate::movie::Movie::find(&media_id).await else {
                return Err(Error {
                    message: "Movie not found".to_string(),
                });
            };

            Ok(MediaDetails {
                imdb_id: movie.imdb_id,
                media_title: movie.title,
                media_poster_uri: movie.poster_path,
                media_release_date: movie.release_date,
            })
        }
        "show" => {
            let Ok(show) = crate::show::Show::find(&media_id).await else {
                return Err(Error {
                    message: "Show not found".to_string(),
                });
            };

            let imdb_id = if let Some(external_ids) = show.external_ids {
                external_ids.imdb_id
            } else {
                None
            };

            Ok(MediaDetails {
                imdb_id,
                media_title: show.name,
                media_poster_uri: show.poster_path,
                media_release_date: show.first_air_date,
            })
        }
        _ => {
            return Err(Error {
                message: "Invalid media type".to_string(),
            });
        }
    }
}
