# tiktok-streaks

Automate your TikTok streaks (currently only manual).

A small Rust program that opens TikTok's web messenger in a real Chrome window and sends a message to each friend you choose, then checks that every message actually went out. Run it once a day and your streaks stay alive without keeping the TikTok app installed.

> **Disclaimer:** This is an unofficial tool. TikTok has no public API for direct messages, so this drives the website the way a person would. Automating your account may violate TikTok's Terms of Service and could get your account restricted. Use it at your own risk, send only a short message once a day, and only to friends who are fine with it.

## How it works

1. Loads `config.toml` (your message and the friends to message).
2. Launches Chrome with a dedicated profile folder, so your TikTok login is remembered between runs.
3. Opens `tiktok.com/messages`.
4. For each friend: clicks their conversation, waits for the chat to load, types the message, presses Enter, and confirms the number of messages in the chat went up.
5. If one friend fails (for example their chat isn't loaded), it logs the failure and carries on with the others.

## Requirements

- [Rust](https://rustup.rs) (stable)
- Google Chrome or Chromium installed
- A TikTok account that can send messages from `tiktok.com/messages` in a normal browser (check this first)

## Setup

### 1. Clone and build

```
git clone <this-repo-url>
cd tiktok-streaks
cargo build
```

### 2. Create your config

Copy the example and edit it:

```
cp config.example.toml config.toml
```

```toml
message = "hey"

[[friends]]
name = "alice"
conversation_id = "0:1:111:222"

[[friends]]
name = "bob"
conversation_id = "0:1:111:333"
```

- `message`: the text sent to everyone.
- `name`: any label you like, used only in logs.
- `conversation_id`: how the program finds the right chat (see below).
- `send_time` is read from the config but not used yet, since running is manual for now.

### 3. Find each friend's `conversation_id`

1. Open `https://www.tiktok.com/messages` in your normal Chrome and wait for your chats to load.
2. Right-click a friend's conversation in the sidebar and choose **Inspect**.
3. Walk up through the parent elements until you find one with a `data-conversation-id="..."` attribute. That value is the ID.
4. To check it, run this in the DevTools console. It should print the element:

```js
document.querySelector("[data-conversation-id='PASTE-ID-HERE']")
```

The ID contains your own account ID as well as your friend's, so treat it as private.

### 4. Log in once

The program uses a Chrome profile stored in `./chrome-profile`. Your login is saved there after you sign in once:

1. Run `cargo run`. A Chrome window opens on TikTok.
2. Log in by hand in that window (complete any CAPTCHA yourself).
3. Let the program finish and close the browser on its own. Avoid killing it with Ctrl+C.

The program only waits 15 seconds for the sidebar to appear, which is too short to log in. For the first login, temporarily raise the `Duration::from_secs(15)` in `send_to_friend` (for example to 120), then put it back once you're logged in.

Keep `chrome-profile/` private, because it contains your logged-in session.

## Usage

```
cargo run
```

Expected output:

```
sent to alice
sent to bob
```

If a friend fails, you'll see a line like `FAILED for bob: timed out waiting for ...`, and the program carries on with the next friend.

To run it daily, start it yourself at a time that suits you. Scheduling (launchd, cron or a systemd timer) is not set up yet.

## Troubleshooting

- **Timed out waiting for a conversation:** that chat isn't in the sidebar. Only roughly the first 30 conversations are loaded without scrolling, so this works best for friends near the top, which streak chats usually are.
- **Logged out:** the sidebar never appears and the first wait times out. Repeat step 4.
- **Selectors stopped working:** TikTok changes its website from time to time. The program relies on these attributes, so check them in DevTools if something breaks:
  - `data-conversation-id` (a conversation row)
  - `[role='textbox'][contenteditable='true']` (the message box)
  - `data-e2e='dm-new-chat-item'` (a message in an open chat)

## Limitations

- Manual runs only, no built-in scheduler yet.
- Tested only with a visible browser window (`with_head()`); headless mode is untested.
- Only conversations loaded at the top of the sidebar can be reached.
- Both people in a streak need to keep messaging, so your friends' side still counts.

## Privacy and safety

`.gitignore` should contain these, so they never reach GitHub:

```
config.toml
chrome-profile/
*.png
target/
```

## License

Add a license of your choice (for example MIT).
