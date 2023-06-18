use anyhow::{anyhow, Result};
use api::ApiClient;
use lazy_static::lazy_static;
use notify::{notify_err, notify_msg, notify_success};
use std::{cmp::max, env, process, thread::sleep, time::Duration};

mod api;
mod notify;

lazy_static! {
	static ref EVENT_URL: String = env::var("EVENT_URL").expect("EVENT_URL environment variable should be defined");
	static ref API_COOKIE: String = env::var("API_COOKIE").expect("API_COOKIE environment variable should be defined");
}

fn main() -> Result<()> {
	ctrlc::set_handler(|| process::exit(0)).expect("should be able to set SIGINT handler");

	loop {
		if let Err(error) = run() {
			println!("ERROR: {}", error);

			notify_err(error)?;
			notify_msg("sleeping for six hours...")?;

			sleep(Duration::from_secs(60 * 60 * 6));
		}
	}
}

fn run() -> Result<()> {
	let client = api::ApiClient::new(EVENT_URL.to_owned(), API_COOKIE.to_owned()).expect("config should be correct");

	loop {
		let info = client.get_info()?;

		if !info.is_sign {
			println!("INFO: attempting check-in...");
			client.post_sign()?;
			println!("INFO: check-in successful");

			notify_signed(&client)?;
		}

		sleep_until_refresh(&client)?;
	}
}

fn sleep_until_refresh(client: &ApiClient) -> Result<()> {
	let recommend = client.get_recommend()?;
	let now = recommend.now.parse::<u64>()?;
	let refresh_time = recommend.refresh_time.parse::<u64>()?;

	// wait until 5 minutes after the refresh time
	let duration = Duration::from_secs(max(refresh_time - now, 0) + 60 * 5);

	println!("INFO: sleeping for {:?}", duration);

	sleep(duration);
	Ok(())
}

fn notify_signed(client: &ApiClient) -> Result<()> {
	println!("INFO: fetching awards data");
	let awards = client.get_awards()?;
	let info = client.get_info()?;

	let current_award_index = (info.total_sign_day - 1) as usize;
	let current_award = awards
		.get(current_award_index)
		.ok_or(anyhow!("API sent invalid awards index: {}", current_award_index))?;

	println!(
		"INFO: days checked-in: {}, days missed: {}, reward: {}x {}",
		info.total_sign_day, info.sign_cnt_missed, current_award.cnt, current_award.name
	);

	let title = format!("{}x {}", current_award.cnt, current_award.name);
	let body = format!(
		"**Check-in successful!**\nTimes checked in: `{}`\nCheck-ins missed: `{}`",
		info.total_sign_day, info.sign_cnt_missed
	);
	notify_success(title, body, current_award.icon.clone())?;

	Ok(())
}
