use error_chain::error_chain;
use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{
    CommandDataOption, CommandDataOptionValue,
};

error_chain! {
    foreign_links {
        Io(std::io::Error);
        HttpRequest(reqwest::Error);
    }
}

async fn load_quotes(token: &String) -> Result<Vec<String>> {
    let list_url = format!("https://twitch.center/customapi/quote/list?token={}", token);
    let res = match reqwest::get(list_url).await {
        Ok(response) => response,
        Err(err) => return Err(err.into()),
    };

    let body = res.text().await?;
    let lines: Vec<_> = body.lines().map(str::to_string).collect();

    return Ok(lines);
}

pub async fn run(options: &[CommandDataOption]) -> Result<String> {
    match dotenvy::var("QUOTES_TOKEN") {
        Ok(token) => {
            let quotes = match load_quotes(&token).await {
                Ok(q) => q,
                Err(err) => return Err(err.into()),
            };

            const MAX: i64 = u32::MAX as i64;
            const MIN: i64 = usize::MIN as i64;

            match options.get(0) {
                Some(x) => match x.resolved.as_ref() {
                    Some(x) => match x {
                        CommandDataOptionValue::String(text) => match text.trim() {
                            "list" => {
                                return Ok(format!(
                                    "https://twitch.center/customapi/quote/list?token={}",
                                    token
                                ))
                            }
                            _ => return Err("Invalid usage".into()),
                        },
                        CommandDataOptionValue::Integer(idx) => match idx {
                            MIN..=MAX => match quotes.get(*idx as usize) {
                                Some(quote) => return Ok(quote.to_string()),
                                None => return Err("Couldn't find the quote".into()),
                            },
                            _ => return Err("Invalid usage".into()),
                        },
                        _ => return Err("Invalid usage".into()),
                    },
                    None => return Err("Something went wrong".into()),
                },
                None => return random_quote(quotes),
            }
        }
        Err(_) => panic!("Quotes token not present"),
    }
}

fn random_quote(quotes: Vec<String>) -> std::result::Result<String, Error> {
    let idx = fastrand::usize(..quotes.len());

    return Ok(quotes[idx].to_string());
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("quote")
        .description("Get a random quote")
        .create_option(|option| {
            option
                .name("list")
                .description("Get the list of quotes")
                .kind(CommandOptionType::SubCommand)
        })
        .create_option(|option| {
            option
                .name("number")
                .description("Number of the quote")
                .kind(CommandOptionType::Integer)
        })
}
