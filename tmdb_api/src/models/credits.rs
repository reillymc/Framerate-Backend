use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Cast {
    pub id: i64,
    pub known_for_department: Option<String>,
    pub name: Option<String>,
    pub popularity: f64,
    pub profile_path: Option<String>,
    pub cast_id: i64,
    pub character: Option<String>,
    pub credit_id: Option<String>,
    pub order: i64,
}

#[derive(Deserialize, Debug)]
pub struct Crew {
    pub id: i64,
    pub known_for_department: Option<String>,
    pub name: Option<String>,
    pub popularity: f64,
    pub profile_path: Option<String>,
    pub credit_id: Option<String>,
    pub department: Option<String>,
    pub job: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Credits {
    pub cast: Vec<Cast>,
    pub crew: Vec<Crew>,
}
