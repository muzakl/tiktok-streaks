use chromiumoxide::Page;
use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::page::ScreenshotParams;
use futures::StreamExt;
use serde::Deserialize;
use std::time::Duration;
use tokio::time::{Instant, sleep};

#[derive(Deserialize, Debug)]
struct Friend {
    name: String,
    conversation_id: String,
}

#[derive(Deserialize, Debug)]
struct Config {
    message: String,
    send_time: String,
    friends: Vec<Friend>,
}

fn load_config(path: &str) -> anyhow::Result<Config> {
    let text = std::fs::read_to_string(path)?;
    let config: Config = toml::from_str(&text)?;
    Ok(config)
}

fn print_plan(config: &Config) {
    for friend in &config.friends {
        println!(
            "Will text '{}' to {} at {}",
            config.message, friend.name, config.send_time
        );
    }
}

async fn wait_for_element(page: &Page, selector: &str, timeout: Duration) -> anyhow::Result<()> {
    let deadline = Instant::now() + timeout;

    loop {
        if page.find_element(selector).await.is_ok() {
            return Ok(());
        }
        if Instant::now() > deadline {
            anyhow::bail!("timed out waiting for {selector}");
        }
        sleep(Duration::from_millis(500)).await;
    }
}

async fn count_messages(page: &Page) -> anyhow::Result<usize> {
    Ok(page
        .find_elements("[data-e2e='dm-new-chat-item']")
        .await?
        .len())
}

async fn wait_for_stable_count(page: &Page, timeout: Duration) -> anyhow::Result<usize> {
    let deadline = Instant::now() + timeout;
    let mut last = count_messages(page).await?;
    let mut stable_checks = 0;

    loop {
        sleep(Duration::from_millis(500)).await;
        let current = count_messages(page).await?;

        if current == last {
            stable_checks += 1;
        } else {
            last = current;
            stable_checks = 0;
        }

        if stable_checks >= 3 {
            return Ok(current);
        }
        if Instant::now() > deadline {
            anyhow::bail!("message count never settled");
        }
    }
}

async fn wait_for_send(page: &Page, before: usize, timeout: Duration) -> anyhow::Result<()> {
    let deadline = Instant::now() + timeout;
    loop {
        if count_messages(page).await? > before {
            return Ok(());
        }
        if Instant::now() > deadline {
            anyhow::bail!("send not confirmed");
        }
        sleep(Duration::from_millis(500)).await;
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = load_config("config.toml")?;
    println!("{:?}", config);
    print_plan(&config);

    let browser_config = BrowserConfig::builder()
        .with_head()
        .user_data_dir("./chrome-profile")
        .build()
        .map_err(|e| anyhow::anyhow!(e))?;

    let (mut browser, mut handler) = Browser::launch(browser_config).await?;

    let handle = tokio::spawn(async move {
        while let Some(h) = handler.next().await {
            if h.is_err() {
                break;
            }
        }
    });

    let page = browser.new_page("https://tiktok.com/messages").await?;

    wait_for_element(
        &page,
        "[data-e2e='dm-new-conversation-avatar']",
        Duration::from_secs(15),
    )
    .await?;

    let friend = &config.friends[0];
    println!("Opening chat with {}", friend.name);

    let selector = format!("[data-conversation-id='{}']", friend.conversation_id);
    println!("{}", selector);

    page.find_element(&selector).await?.click().await?;
    wait_for_element(
        &page,
        "[role='textbox'][contenteditable='true']",
        Duration::from_secs(10),
    )
    .await?;

    let before = wait_for_stable_count(&page, Duration::from_secs(10)).await?;
    println!("messages before: {}", before);

    page.find_element("[role='textbox'][contenteditable='true']")
        .await?
        .click()
        .await?
        .type_str(&config.message)
        .await?
        .press_key("Enter")
        .await?;

    wait_for_send(&page, before, Duration::from_secs(10)).await?;
    println!("sent!");

    page.save_screenshot(
        ScreenshotParams::builder().full_page(true).build(),
        "messages.png",
    )
    .await?;

    sleep(Duration::from_secs(5)).await;
    browser.close().await?;
    handle.await?;
    Ok(())
}
