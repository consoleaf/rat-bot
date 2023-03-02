use error_chain::error_chain;
use regex::Regex;
use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::interaction::application_command::{
    CommandDataOption, CommandDataOptionValue,
};

error_chain! {
    foreign_links {
        Io(std::io::Error);
        HttpRequest(reqwest::Error);
    }
}

async fn load_quotes(token: String) -> Result<Vec<String>> {
    let res = match reqwest::get(format!(
        "https://twitch.center/customapi/quote/list?token={}",
        token
    ))
    .await
    {
        Ok(it) => it,
        Err(err) => return Err(err.into()),
    };

    let RE: Regex = Regex::new(r"^\d+\.\w(?P<content>.*)$").unwrap();

    let body = res.text().await?;
    let lines: Vec<_> = body
        .lines()
        .map(str::to_string)
        .map(|line| match RE.captures(line.as_str()) {
            Some(cap) => Some(cap.name("content").unwrap().as_str().to_string()),
            None => None,
        })
        .filter(|x| match x {
            Some(_) => true,
            None => false,
        })
        .map(|x| x.unwrap())
        .collect();

    return Ok(lines);
}

pub async fn run(options: &[CommandDataOption]) -> Result<String> {
    match dotenvy::var("QUOTES_TOKEN") {
        Ok(token) => {
            let quotes = match load_quotes(token).await {
                Ok(q) => q,
                Err(err) => return Err(err.into()),
            };

            let idx = fastrand::usize(..quotes.len());

            return Ok(quotes[idx].to_string());
        }
        Err(_) => panic!("Quotes token not present"),
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("quote").description("Get a random quote")
}
