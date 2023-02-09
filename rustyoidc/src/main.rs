use std::env;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::process::exit;

use serde::{Deserialize, Serialize};
use url::Url;

use openidconnect::core::{
    CoreAuthDisplay, CoreClaimName, CoreClaimType, CoreClient, CoreClientAuthMethod, CoreGrantType,
    CoreIdTokenClaims, CoreIdTokenVerifier, CoreJsonWebKey, CoreJsonWebKeyType, CoreJsonWebKeyUse,
    CoreJweContentEncryptionAlgorithm, CoreJweKeyManagementAlgorithm, CoreJwsSigningAlgorithm,
    CoreResponseMode, CoreResponseType, CoreRevocableToken, CoreSubjectIdentifierType,
};
use openidconnect::reqwest::http_client;
use openidconnect::{
    AdditionalProviderMetadata, AuthenticationFlow, AuthorizationCode, ClientId, ClientSecret,
    CsrfToken, IssuerUrl, Nonce, OAuth2TokenResponse, ProviderMetadata, RedirectUrl, RevocationUrl,
    Scope,
};

fn handle_error<T: std::error::Error>(fail: &T, msg: &'static str) {
    let mut err_msg = format!("ERROR: {}", msg);
    let mut cur_fail: Option<&dyn std::error::Error> = Some(fail);
    while let Some(cause) = cur_fail {
        err_msg += &format!("\n    caused by: {}", cause);
        cur_fail = cause.source();
    }
    println!("{}", err_msg);
    exit(1);
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct RevocationEndpointProviderMetadata {
    revocation_endpoint: String,
}
impl AdditionalProviderMetadata for RevocationEndpointProviderMetadata {}
type GoogleProviderMetadata = ProviderMetadata<
    RevocationEndpointProviderMetadata,
    CoreAuthDisplay,
    CoreClientAuthMethod,
    CoreClaimName,
    CoreClaimType,
    CoreGrantType,
    CoreJweContentEncryptionAlgorithm,
    CoreJweKeyManagementAlgorithm,
    CoreJwsSigningAlgorithm,
    CoreJsonWebKeyType,
    CoreJsonWebKeyUse,
    CoreJsonWebKey,
    CoreResponseMode,
    CoreResponseType,
    CoreSubjectIdentifierType,
    >;


fn main() {
    env_logger::init();

    let google_client_id = ClientId::new(
        env::var("GOOGLE_CLIENT_ID").expect("Missing the GOOGLE_CLIENT_ID environment variable."),
    );
    let google_client_secret = ClientSecret::new(
        env::var("GOOGLE_CLIENT_SECRET")
            .expect("Missing the GOOGLE_CLIENT_SECRET environment variable."),
        );
    let issuer_url =
        IssuerUrl::new("https://accounts.google.com".to_string()).expect("Invalid issuer URL");

    let provider_metadata = GoogleProviderMetadata::discover(&issuer_url, http_client)
        .unwrap_or_else(|err| {
            handle_error(&err, "Failed to discover OpenID Provider");
            unreachable!();
        });

    let revocation_endpoint = provider_metadata
        .additional_metadata()
        .revocation_endpoint
        .clone();
    println!(
        "Discovered Google revocation endpoint: {}",
        revocation_endpoint
    );

    let client = CoreClient::from_provider_metadata(
        provider_metadata,
        google_client_id,
        Some(google_client_secret),
    ).set_redirect_uri(
        RedirectUrl::new("http://localhost:8080".to_string()).expect("Invalid redirect URL"),
    );

    let (authorize_url, csrf_state, nonce) = client
        .authorize_url(
            AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .url();

    println!(
        "Open this URL in your browser:\n{}\n",
        authorize_url.to_string()
    );

    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    for stream in listener.incoming() {
        if let Ok(mut stream) = stream {
            let code;
            let state;
            {
                let mut reader = BufReader::new(&stream);
                let mut request_line = String::new();
                reader.read_line(&mut request_line).unwrap();

                let redirect_url = request_line.split_whitespace().nth(1).unwrap();
                let url = Url::parse(&("http://localhost".to_string() + redirect_url)).unwrap();

                let code_pair = url
                    .query_pairs()
                    .find(|pair| {
                        let &(ref key, _) = pair;
                        key == "code"
                    })
                    .unwrap();

                let (_, value) = code_pair;
                code = AuthorizationCode::new(value.into_owned());

                let state_pair = url
                    .query_pairs()
                    .find(|pair| {
                        let &(ref key, _) = pair;
                        key == "state"
                    })
                    .unwrap();

                let (_, value) = state_pair;
                state = CsrfToken::new(value.into_owned());

                let message = "Go back to your terminal :)";
                let response = format!(
                    "Http/1.1 200 OK\r\ncontent-length: {}\r\n\r\n{}",
                    message.len(),
                    message
                );
                stream.write_all(response.as_bytes()).unwrap();

                println!("Google returned the follwing code:\n{}\n", code.secret());
                println!(
                    "Google returned new following state:\n{} (expected `{}`)\n",
                    state.secret(),
                    csrf_state.secret()
                );

                let token_response = client
                    .exchange_code(code)
                    .request(http_client)
                    .unwrap_or_else(|err| {
                        handle_error(&err, "Failed to contact token endpoint");
                        unreachable!();
                    });

                println!(
                    "Gooogle returned access token:\n{}\n",
                    token_response.access_token().secret()
                );
                println!("Google returned scopes: {:?}", token_response.scopes());

                let id_token_verifier: CoreIdTokenVerifier = client.id_token_verifier();
                let id_token_claims: &CoreIdTokenClaims = token_response
                    .extra_fields()
                    .id_token()
                    .expect("Server did not return an ID token")
                    .claims(&id_token_verifier, &nonce)
                    .unwrap_or_else(|err| {
                        handle_error(&err, "Failed to verify ID token");
                        unreachable!();
                    });
                println!("Google returned ID token: {:?}", id_token_claims);

                let token_to_revoke: CoreRevocableToken = match token_response.refresh_token() {
                    Some(token) => token.into(),
                    None => token_response.access_token().into(),
                };

                client
                    .revoke_token(token_to_revoke)
                    .expect("no revocation_uri_configured")
                    .request(http_client)
                    .unwrap_or_else(|err| {
                        handle_error(&err, "Failed to contact token revocation endpoint");
                        unreachable!();
                    });

                break;
            }
        }
    }
}
