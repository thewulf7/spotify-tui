# Authentication Migration Instructions for spotify-tui

## Goal
Fix `spotify-tui` authentication by adopting the PKCE OAuth flow used by `ncspot`. This removes the need for manual Client ID/Secret setup and complies with current Spotify policies.

## Strategy
1. **Use `librespot-oauth`**: Leverages the same library as `ncspot` for robust authentication.
2. **PKCE Flow**: Secure flow for desktop apps without requiring a Client Secret.
3. **Shared Client ID**: Use the established `ncspot` Client ID (`d420a117a32841c2b3474932e49fb54b`).
4. **Automated Onboarding**: The app will automatically open the browser for login on the first run.

## Tasks

### 1. Dependency Update (Done)
- Added `librespot-oauth = "0.8.0"` and `chrono = "0.4"` to `Cargo.toml`.

### 2. Implement Token Mapping & Acquisition
- Create a helper to convert `librespot_oauth::OAuthToken` to `rspotify::oauth2::TokenInfo`.
- Implement a PKCE acquisition function using `OAuthClientBuilder`.

### 3. Update `src/main.rs`
- Remove legacy `SpotifyOAuth` logic.
- Integrate the new PKCE flow into the startup sequence.
- Ensure the token is cached in `.spotify_token_cache.json`.

### 4. Update `src/network.rs`
- Update `refresh_authentication` to use `librespot-oauth` for token refreshing.
- Ensure the application state correctly handles token expiry and renewal.

### 5. Cleanup `src/config.rs`
- Simplify `ClientConfig` to stop prompting for manual credentials.

### 6. Verification
- Run `cargo check` to ensure type compatibility.
- Perform a fresh login by clearing existing config files.
- Verify playback and API data fetching.