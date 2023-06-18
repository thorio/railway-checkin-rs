use anyhow::{anyhow, bail, Result};
use reqwest::{blocking::Client, header::HeaderMap};
use serde::{de::DeserializeOwned, Deserialize};

pub struct ApiClient {
	url_pattern: String,
	http_client: Client,
}

impl ApiClient {
	pub fn new(url_pattern: String, cookie: String) -> Result<Self> {
		let mut header_map = HeaderMap::new();
		header_map.insert("Cookie", cookie.parse()?);

		let client = Client::builder().default_headers(header_map).build()?;

		Ok(ApiClient {
			url_pattern,
			http_client: client,
		})
	}

	pub fn get_recommend(&self) -> Result<RecommendData> {
		self.get_request_with_error_handling::<RecommendData>("recommend")
	}

	pub fn get_info(&self) -> Result<InfoData> {
		self.get_request_with_error_handling::<InfoData>("info")
	}

	pub fn get_awards(&self) -> Result<Vec<Award>> {
		Ok(self.get_request_with_error_handling::<HomeData>("home")?.awards)
	}

	pub fn post_sign(&self) -> Result<()> {
		let response = self.post_request::<Message<SignData>>("sign")?;

		if response.retcode != 0 {
			bail!("API returned unhandled retcode `{0}`", response.retcode);
		}

		Ok(())
	}

	fn get_request_with_error_handling<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T> {
		let response = self.get_request::<Message<T>>(endpoint)?;

		if response.retcode != 0 {
			bail!("API returned unhandled retcode `{0}`", response.retcode);
		}

		response
			.data
			.ok_or(anyhow!("response has retcode 0 but is missing data"))
	}

	fn get_request<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T> {
		Ok(self.http_client.get(self.get_url(endpoint)).send()?.json::<T>()?)
	}

	fn post_request<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T> {
		Ok(self.http_client.post(self.get_url(endpoint)).send()?.json::<T>()?)
	}

	fn get_url(&self, endpoint: &str) -> String {
		self.url_pattern.clone().replace("{}", endpoint)
	}
}

#[derive(Debug, Deserialize)]
pub struct Message<T> {
	retcode: i32,
	data: Option<T>,
}

#[derive(Debug, Deserialize)]
pub struct SignData {}

#[derive(Debug, Deserialize)]
struct HomeData {
	pub awards: Vec<Award>,
}

#[derive(Debug, Deserialize)]
pub struct Award {
	pub icon: String,
	pub name: String,
	pub cnt: i32,
}

#[derive(Debug, Deserialize)]
pub struct InfoData {
	pub total_sign_day: i32,
	pub is_sign: bool,
	pub sign_cnt_missed: i32,
}

#[derive(Debug, Deserialize)]
pub struct RecommendData {
	pub now: String,
	pub refresh_time: String,
}
