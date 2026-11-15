use chrono::{Duration, Utc};
use keyring::Entry;
use serde::Serialize;

const KEYRING_SERVICE: &str = "com.adrianvale.marketview";
const KEYRING_USER: &str = "alpaca";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MarketQuote {
    symbol: String,
    current_price: f64,
    previous_close: f64,
    change_percent: f64,
    closes: Vec<f64>,
    highs: Vec<f64>,
    lows: Vec<f64>,
    volumes: Vec<f64>,
    intraday_closes: Vec<f64>,
    currency: String,
    market_state: String,
    data_source: String,
    last_updated: String,
}

fn credential_entry() -> Result<Entry, String> {
    Entry::new(KEYRING_SERVICE, KEYRING_USER)
        .map_err(|error| format!("Secure credential storage is unavailable: {error}"))
}

#[tauri::command]
fn has_alpaca_credentials() -> Result<bool, String> {
    match credential_entry()?.get_password() {
        Ok(_) => Ok(true),
        Err(keyring::Error::NoEntry) => Ok(false),
        Err(error) => Err(format!("Could not read saved Alpaca credentials: {error}")),
    }
}

#[tauri::command]
fn save_alpaca_credentials(api_key: String, api_secret: String) -> Result<(), String> {
    let api_key = api_key.trim();
    let api_secret = api_secret.trim();
    if api_key.is_empty() || api_secret.is_empty() || api_key.len() > 256 || api_secret.len() > 512 {
        return Err("Enter a valid Alpaca API key and secret.".to_string());
    }
    credential_entry()?.set_password(&format!("{api_key}\n{api_secret}"))
        .map_err(|error| format!("Could not save credentials securely: {error}"))
}

#[tauri::command]
fn clear_alpaca_credentials() -> Result<(), String> {
    match credential_entry()?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(error) => Err(format!("Could not remove saved Alpaca credentials: {error}")),
    }
}

fn load_alpaca_credentials() -> Result<(String, String), String> {
    let saved = credential_entry()?.get_password()
        .map_err(|_| "Add your Alpaca API key and secret in MarketView Settings to enable live market data.".to_string())?;
    let mut parts = saved.splitn(2, '\n');
    let key = parts.next().unwrap_or_default().to_string();
    let secret = parts.next().unwrap_or_default().to_string();
    if key.is_empty() || secret.is_empty() {
        return Err("Saved Alpaca credentials are incomplete. Please enter them again in Settings.".to_string());
    }
    Ok((key, secret))
}

#[tauri::command]
async fn get_market_quote(symbol: String) -> Result<MarketQuote, String> {
    let symbol = symbol.trim().to_uppercase();
    if symbol.is_empty() || symbol.len() > 20 || !symbol.chars().all(|c| c.is_ascii_alphanumeric() || ".-=^".contains(c)) {
        return Err("Enter a valid ticker symbol.".to_string());
    }
    let (api_key, api_secret) = load_alpaca_credentials()?;
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build().map_err(|_| "Market data client could not be initialised.".to_string())?;

    let headers = |request: reqwest::RequestBuilder| request
        .header("APCA-API-KEY-ID", &api_key)
        .header("APCA-API-SECRET-KEY", &api_secret)
        .header("Accept", "application/json");

    let start_daily = (Utc::now() - Duration::days(120)).to_rfc3339();
    let daily_url = format!(
        "https://data.alpaca.markets/v2/stocks/{symbol}/bars?timeframe=1Day&start={}&limit=1000&adjustment=split&feed=iex&sort=asc",
        urlencoding::encode(&start_daily)
    );
    let daily_response = headers(client.get(daily_url)).send().await
        .map_err(|_| "Alpaca could not be reached. Check your internet connection.".to_string())?;
    if !daily_response.status().is_success() {
        let status = daily_response.status();
        let body = daily_response.text().await.unwrap_or_default();
        return Err(if status.as_u16() == 401 || status.as_u16() == 403 {
            "Alpaca rejected the credentials or market-data subscription. Check your API key, secret and feed access in Alpaca.".to_string()
        } else {
            format!("Alpaca market data returned HTTP {status}: {}", body.chars().take(220).collect::<String>())
        });
    }
    let daily_payload: serde_json::Value = daily_response.json().await
        .map_err(|_| "Alpaca daily-bar response could not be read.".to_string())?;
    let daily_bars = daily_payload["bars"].as_array().cloned().unwrap_or_default();
    if daily_bars.is_empty() {
        return Err(format!("Alpaca returned no daily bars for {symbol}. Check the ticker symbol and market."));
    }
    let closes: Vec<f64> = daily_bars.iter().filter_map(|bar| bar["c"].as_f64()).collect();
    let highs: Vec<f64> = daily_bars.iter().filter_map(|bar| bar["h"].as_f64()).collect();
    let lows: Vec<f64> = daily_bars.iter().filter_map(|bar| bar["l"].as_f64()).collect();
    let volumes: Vec<f64> = daily_bars.iter().filter_map(|bar| bar["v"].as_f64()).collect();
    let previous_close = if closes.len() > 1 { closes[closes.len() - 2] } else { closes[0] };

    let intraday_start = (Utc::now() - Duration::days(5)).to_rfc3339();
    let intraday_url = format!(
        "https://data.alpaca.markets/v2/stocks/{symbol}/bars?timeframe=1Min&start={}&limit=10000&adjustment=raw&feed=iex&sort=asc",
        urlencoding::encode(&intraday_start)
    );
    let intraday_response = headers(client.get(intraday_url)).send().await
        .map_err(|_| "Alpaca intraday data could not be reached.".to_string())?;
    if !intraday_response.status().is_success() {
        let status = intraday_response.status();
        return Err(format!("Alpaca intraday data returned HTTP {status}. Check your data subscription and try again."));
    }
    let intraday_payload: serde_json::Value = intraday_response.json().await
        .map_err(|_| "Alpaca intraday response could not be read.".to_string())?;
    let intraday_bars = intraday_payload["bars"].as_array().cloned().unwrap_or_default();
    let intraday_closes: Vec<f64> = intraday_bars.iter().filter_map(|bar| bar["c"].as_f64()).collect();
    let current_price = intraday_closes.last().copied().unwrap_or_else(|| *closes.last().unwrap_or(&0.0));
    if current_price <= 0.0 {
        return Err("Alpaca did not return a usable price for this symbol.".to_string());
    }
    let change_percent = if previous_close > 0.0 { (current_price - previous_close) / previous_close * 100.0 } else { 0.0 };

    Ok(MarketQuote {
        symbol,
        current_price,
        previous_close,
        change_percent,
        closes,
        highs,
        lows,
        volumes,
        intraday_closes,
        currency: "USD".to_string(),
        market_state: "ALPACA IEX".to_string(),
        data_source: "Alpaca · IEX real-time feed".to_string(),
        last_updated: Utc::now().to_rfc3339(),
    })
}

#[tauri::command]
fn open_external_url(url: String) -> Result<(), String> {
    if url.len() > 2048 || !url.starts_with("https://") || url.chars().any(char::is_control) {
        return Err("Only valid HTTPS links can be opened.".to_string());
    }
    open::that(url).map_err(|error| format!("The system could not open the trading platform: {error}"))
}

#[cfg_attr(mobile,tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_market_quote,
            has_alpaca_credentials,
            save_alpaca_credentials,
            clear_alpaca_credentials,
            open_external_url
        ])
        .run(tauri::generate_context!())
        .expect("error while running MarketView");
}
