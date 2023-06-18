use anyhow::Result;
use lazy_static::lazy_static;
use serde::Serialize;
use std::env;

lazy_static! {
	static ref WEBHOOK_URL: Option<String> = env::var("WEBHOOK_URL").ok();
}

const SUCCESS_COLOR: i32 = 2605643;
const ERROR_COLOR: i32 = 14037554;

pub fn notify_success(title: String, body: String, thumbnail_url: String) -> Result<()> {
	let body = DiscordWebhookBody {
		content: None,
		embeds: Some(vec![DiscordEmbed {
			color: SUCCESS_COLOR,
			title,
			description: body,
			thumbnail: Some(DiscordImage { url: thumbnail_url }),
		}]),
	};

	notify(body)
}

pub fn notify_err(err: anyhow::Error) -> Result<()> {
	let body = DiscordWebhookBody {
		content: None,
		embeds: Some(vec![DiscordEmbed {
			color: ERROR_COLOR,
			title: String::from("Unhandled Error"),
			description: format!("```\n{:?}\n```", err),
			thumbnail: None,
		}]),
	};

	notify(body)
}

pub fn notify_msg(msg: &str) -> Result<()> {
	let body = DiscordWebhookBody {
		content: Some(msg.to_string()),
		embeds: None,
	};

	notify(body)
}

fn notify(data: DiscordWebhookBody) -> Result<()> {
	if WEBHOOK_URL.is_none() {
		return Ok(());
	}

	let client = reqwest::blocking::Client::new();

	client
		.post(WEBHOOK_URL.to_owned().unwrap())
		.json(&data)
		.send()?
		.error_for_status()?;

	Ok(())
}

#[derive(Debug, Serialize)]
struct DiscordWebhookBody {
	pub content: Option<String>,
	pub embeds: Option<Vec<DiscordEmbed>>,
}

#[derive(Debug, Serialize)]
struct DiscordEmbed {
	pub title: String,
	pub description: String,
	pub color: i32,
	pub thumbnail: Option<DiscordImage>,
}

#[derive(Debug, Serialize)]
struct DiscordImage {
	pub url: String,
}
