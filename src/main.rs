use actix_web::{middleware, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use once_cell::sync::Lazy;
use ring::hmac;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs::File;
use std::io::prelude::*;
use teloxide::prelude::*;
use teloxide::utils::command::BotCommand;

// static CONFIG: Lazy<Config> = Lazy::new(|| {
//     let config = load_config();
//     config
// });
const TELOXIDE_TOKEN: &str = "TELOXIDE_TOKEN";

static KEY: Lazy<hmac::Key> = Lazy::new(|| {
    // let config = load_config();
    let token = std::env::var(TELOXIDE_TOKEN).unwrap();
    let key = hmac::Key::new(hmac::HMAC_SHA256, token.as_bytes());
    key
});

static HOSTNAME: Lazy<String> =
    Lazy::new(|| hostname::get().unwrap().to_string_lossy().to_string());

#[derive(BotCommand, Debug)]
#[command(rename = "lowercase", description = "These commands are supported:")]
pub enum Command {
    #[command(description = "display this text.")]
    Help,

    #[command(description = "get chat id")]
    ChatId,

    #[command(description = "get push url")]
    PushUrl,
}

#[derive(Deserialize)]
pub struct Telegram {
    token: String,
}

#[derive(Deserialize)]
pub struct Config {
    telegram: Telegram,
}

#[derive(Deserialize, Serialize)]
pub struct TextMessage {
    text: String,
}

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    run().await;
}

fn load_config() -> Config {
    let mut input = String::new();
    File::open("config.toml")
        .and_then(|mut f| f.read_to_string(&mut input))
        .unwrap();
    toml::from_str(&input).unwrap()
}

async fn run() {
    teloxide::enable_logging!();
    log::info!("Start bot");

    // let bot = Bot::builder().token(CONFIG.telegram.token.clone()).build();
    let bot = Bot::from_env();

    // teloxide::commands_repl(bot.clone(), "KnockKnock", answer).await;

    // teloxide::repl(bot.clone(), |message| async move {
    //     println!("{:?}", message.update.text());

    //     if let Some(text) = message.update.text() {
    //         if let Ok(command) = Command::parse(text, "bot name") {
    //             answer(&message, command).await?;
    //         } else {
    //             message.answer("fuck").send().await?;
    //         }
    //     }

    //     ResponseResult::<()>::Ok(())
    // })
    // .await;

    let tg_bot = bot.clone();

    let _telegram_handle = tokio::spawn(async move {
        // You have to put your real bot name here
        // Or group command won't work
        teloxide::commands_repl(tg_bot, "knockknock2020_bot", answer).await;
    });

    // let web_bot = bot.clone();

    let local = tokio::task::LocalSet::new();
    let sys = actix_web::rt::System::run_in_tokio("server", &local);
    HttpServer::new(move || {
        App::new()
            .data(bot.clone())
            .wrap(middleware::Logger::default())
            .service(
                web::resource("/chatid/{chat_id}/sign/{sign}/text")
                    .route(web::post().to(send_text)),
            )
            .service(web::resource("/").to(hello))
    })
    .bind("127.0.0.1:3000")
    .unwrap()
    .run()
    .await
    .unwrap();

    sys.await.unwrap();
}

async fn send_text(
    bot: web::Data<teloxide::Bot>,
    text_message: web::Json<TextMessage>,
    _req: HttpRequest,
    web::Path((chat_id, sign)): web::Path<(i64, String)>,
) -> HttpResponse {
    if check_sign(chat_id, &sign) {
        let text = &text_message.text;
        bot.send_message(chat_id, text)
            .parse_mode(teloxide::types::ParseMode::MarkdownV2)
            .send()
            .await
            .unwrap();
        HttpResponse::Ok().json(json!({"chat_id": chat_id, "sign": sign, "status": "ok"}))
    } else {
        HttpResponse::Ok().json(json!({"chat_id": chat_id, "sign": sign, "status": "err"}))
    }
}

async fn hello() -> impl Responder {
    "Hello".to_string()
}

async fn answer(cx: UpdateWithCx<Message>, command: Command) -> ResponseResult<()> {
    match command {
        Command::Help => cx.answer(Command::descriptions()).send().await?,
        Command::ChatId => {
            cx.answer(format!("Chat id: \n{}", cx.update.chat_id()))
                .send()
                .await?
        }
        Command::PushUrl => {
            let chat_id = cx.update.chat_id();
            let sign = sign_chat_id(chat_id);
            cx.answer(format!(
                "Push url: \nhttps://{}:3000/chatid/{}/sign/{}/text",
                *HOSTNAME, chat_id, sign
            ))
            .send()
            .await?
        }
    };

    Ok(())
}

fn sign_chat_id(chat_id: i64) -> String {
    let sign = hmac::sign(&KEY, chat_id.to_string().as_bytes());
    hex::encode(sign.as_ref())
}

fn check_sign(chat_id: i64, sign: &str) -> bool {
    let tag = hex::decode(sign).unwrap();
    hmac::verify(&KEY, chat_id.to_string().as_bytes(), &tag).is_ok()
}
