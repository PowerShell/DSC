// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use futures_util::TryStreamExt;
use oci_client::{
    Client, Reference,
    client::{ClientConfig, ClientProtocol},
    secrets::RegistryAuth,
};
use reqwest::{
    Url,
    header::{LINK, WWW_AUTHENTICATE},
};
use rust_i18n::t;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, time::Duration};

const INDEX: &str = "application/vnd.oci.image.index.v1+json";
const IMAGE: &str = "application/vnd.oci.image.manifest.v1+json";
const ARTIFACT: &str = "application/vnd.microsoft.dsc.resource.v1+zip";
const LIMIT: usize = 8 * 1024 * 1024;
#[cfg(not(windows))]
static TLS_PROVIDER: std::sync::LazyLock<Result<(), String>> = std::sync::LazyLock::new(|| {
    if rustls::crypto::CryptoProvider::get_default().is_none() {
        rustls::crypto::ring::default_provider()
            .install_default()
            .map_err(|_| t!("oci.crypto").to_string())?;
    }
    Ok(())
});

pub fn registry_url(location: &str) -> Result<Url, String> {
    let url = Url::parse(&if location.contains("://") {
        location.to_string()
    } else {
        format!("https://{location}")
    })
    .map_err(|_| t!("oci.location").to_string())?;
    let loopback = url.host_str().is_some_and(|host| {
        host == "localhost"
            || host == "[::1]"
            || host
                .parse::<std::net::IpAddr>()
                .is_ok_and(|ip| ip.is_loopback())
    });
    if url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || url.path() != "/"
        || !(url.scheme() == "https" || (url.scheme() == "http" && loopback))
    {
        return Err(t!("oci.location").to_string());
    }
    Ok(url)
}

fn transport_error<T>(_: T) -> String {
    // Registry-controlled response bodies can contain credentials.
    t!("oci.transport").to_string()
}

fn invalid() -> String {
    t!("oci.document").to_string()
}

pub fn verify(bytes: &[u8], descriptor: &Value) -> Result<(), String> {
    let digest = sha256_digest(bytes);
    if descriptor["digest"] != digest || descriptor["size"].as_u64() != Some(bytes.len() as u64) {
        return Err(t!("oci.integrity").to_string());
    }
    Ok(())
}

fn sha256_digest(bytes: &[u8]) -> String {
    format!(
        "sha256:{}",
        base16ct::lower::encode_string(Sha256::digest(bytes).as_slice())
    )
}

fn descriptor_valid(descriptor: &Value) -> bool {
    descriptor["digest"].as_str().is_some_and(|digest| {
        digest.strip_prefix("sha256:").is_some_and(|hex| {
            hex.len() == 64
                && hex
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        })
    }) && descriptor["size"].as_u64().is_some_and(|size| size > 0)
}

fn identity(document: &Value) -> Result<(&str, &str), String> {
    if document["schemaVersion"] != 2 || document["artifactType"] != ARTIFACT {
        return Err(invalid());
    }
    let name = document["annotations"]["com.microsoft.dsc.resource.id"]
        .as_str()
        .ok_or_else(invalid)?;
    let version = document["annotations"]["com.microsoft.dsc.version"]
        .as_str()
        .ok_or_else(invalid)?;
    let (namespace, resource) = name.split_once('/').ok_or_else(invalid)?;
    if namespace.is_empty()
        || resource.is_empty()
        || resource.contains('/')
        || semver::Version::parse(version).is_err()
    {
        return Err(invalid());
    }
    Ok((name, version))
}

fn host_platform() -> Result<(&'static str, &'static str), String> {
    let os = match std::env::consts::OS {
        "windows" => "windows",
        "linux" => "linux",
        "macos" => "darwin",
        _ => return Err(t!("oci.platform").to_string()),
    };
    let arch = match std::env::consts::ARCH {
        "x86_64" => "amd64",
        "aarch64" => "arm64",
        _ => return Err(t!("oci.platform").to_string()),
    };
    Ok((os, arch))
}

fn select_child<'a>(index: &'a Value, platform: (&str, &str)) -> Result<Option<&'a Value>, String> {
    identity(index)?;
    let children = index["manifests"]
        .as_array()
        .filter(|children| !children.is_empty())
        .ok_or_else(invalid)?;
    let mut platforms = BTreeSet::new();
    let mut selected = None;
    for child in children {
        let os = child["platform"]["os"].as_str().ok_or_else(invalid)?;
        let arch = child["platform"]["architecture"]
            .as_str()
            .ok_or_else(invalid)?;
        if child["mediaType"] != IMAGE
            || !descriptor_valid(child)
            || !matches!(os, "windows" | "linux" | "darwin")
            || !matches!(arch, "amd64" | "arm64")
            || !platforms.insert((os, arch))
        {
            return Err(invalid());
        }
        if (os, arch) == platform {
            selected = Some(child);
        }
    }
    Ok(selected)
}

async fn anonymous_token(
    http: &reqwest::Client,
    base: &Url,
    response: &reqwest::Response,
) -> Result<String, String> {
    let header = response
        .headers()
        .get(WWW_AUTHENTICATE)
        .ok_or_else(|| t!("oci.catalog", status = 401).to_string())?
        .to_str()
        .map_err(transport_error)?;
    for challenge in http_auth::parser::ChallengeParser::new(header) {
        let challenge = challenge.map_err(transport_error)?;
        if !challenge.scheme.eq_ignore_ascii_case("Bearer") {
            continue;
        }
        let parameter = |name: &str| {
            challenge
                .params
                .iter()
                .find(|(key, _)| key.eq_ignore_ascii_case(name))
                .map(|(_, value)| value.to_unescaped())
        };
        let mut realm =
            Url::parse(&parameter("realm").ok_or_else(invalid)?).map_err(transport_error)?;
        if !realm.username().is_empty()
            || realm.password().is_some()
            || (realm.scheme() != "https"
                && !(realm.scheme() == "http" && realm.origin() == base.origin()))
        {
            return Err(t!("oci.token").to_string());
        }
        {
            let mut query = realm.query_pairs_mut();
            for key in ["service", "scope"] {
                if let Some(value) = parameter(key) {
                    query.append_pair(key, &value);
                }
            }
        }
        let grant = http.get(realm).send().await.map_err(transport_error)?;
        if !grant.status().is_success() {
            return Err(t!("oci.token").to_string());
        }
        let value = limited_json(grant).await?;
        return value
            .get("token")
            .or_else(|| value.get("access_token"))
            .and_then(Value::as_str)
            .filter(|token| !token.is_empty())
            .map(str::to_string)
            .ok_or_else(|| t!("oci.token").to_string());
    }
    Err(t!("oci.token").to_string())
}

async fn catalog(
    http: &reqwest::Client,
    base: &Url,
    token: Option<&str>,
) -> Result<BTreeSet<String>, String> {
    let mut next = Some(base.join("v2/_catalog?n=100").map_err(transport_error)?);
    let mut pages = BTreeSet::new();
    let mut repositories = BTreeSet::new();
    let mut catalog_token = token.map(str::to_string);
    while let Some(url) = next.take() {
        if url.origin() != base.origin()
            || url.path() != "/v2/_catalog"
            || !pages.insert(url.to_string())
        {
            return Err(t!("oci.pagination").to_string());
        }
        let mut request = http.get(url.clone());
        if let Some(token) = &catalog_token {
            request = request.bearer_auth(token);
        }
        let mut response = request.send().await.map_err(transport_error)?;
        if response.status() == reqwest::StatusCode::UNAUTHORIZED && catalog_token.is_none() {
            let token = anonymous_token(http, base, &response).await?;
            response = http
                .get(url.clone())
                .bearer_auth(&token)
                .send()
                .await
                .map_err(transport_error)?;
            catalog_token = Some(token);
        }
        if !response.status().is_success() {
            return Err(t!("oci.catalog", status = response.status().as_u16()).to_string());
        }
        if let Some(link) = response.headers().get(LINK) {
            let link = link.to_str().map_err(transport_error)?;
            for part in link.split(',') {
                if part.contains("rel=\"next\"") || part.contains("rel=next") {
                    let end = part.find('>').ok_or_else(invalid)?;
                    let start = part.find('<').ok_or_else(invalid)? + 1;
                    next = Some(
                        url.join(part.get(start..end).ok_or_else(invalid)?)
                            .map_err(transport_error)?,
                    );
                }
            }
        }
        let document = limited_json(response).await?;
        let entries = document["repositories"].as_array().ok_or_else(invalid)?;
        for entry in entries {
            let repository = entry.as_str().ok_or_else(invalid)?;
            if repository.starts_with("dsc/resource/") {
                repositories.insert(repository.to_string());
            }
        }
    }
    Ok(repositories)
}

async fn limited_json(mut response: reqwest::Response) -> Result<Value, String> {
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(transport_error)? {
        if bytes.len() + chunk.len() > LIMIT {
            return Err(t!("oci.limit").to_string());
        }
        bytes.extend_from_slice(&chunk);
    }
    serde_json::from_slice(&bytes).map_err(|_| invalid())
}

async fn manifest(
    client: &Client,
    reference: &Reference,
    auth: &RegistryAuth,
) -> Result<Vec<u8>, String> {
    let (bytes, _) = client
        .pull_manifest_raw(reference, auth, &[INDEX, IMAGE])
        .await
        .map_err(transport_error)?;
    if bytes.len() > LIMIT {
        return Err(t!("oci.limit").to_string());
    }
    Ok(bytes.to_vec())
}

async fn resource_manifest(
    client: &Client,
    reference: &Reference,
    auth: &RegistryAuth,
    child: &Value,
    expected: (&str, &str),
) -> Result<Value, String> {
    let child_bytes = manifest(client, reference, auth).await?;
    verify(&child_bytes, child)?;
    let document: Value = serde_json::from_slice(&child_bytes).map_err(|_| invalid())?;
    if document["mediaType"] != IMAGE
        || identity(&document)? != expected
        || !descriptor_valid(&document["config"])
    {
        return Err(invalid());
    }
    let layers = document["layers"]
        .as_array()
        .filter(|layers| layers.len() == 2)
        .ok_or_else(invalid)?;
    let resource_layer = layers
        .iter()
        .find(|layer| layer["mediaType"] == "application/json")
        .ok_or_else(invalid)?;
    if !layers
        .iter()
        .any(|layer| layer["mediaType"] == "application/zip")
        || layers.iter().any(|layer| !descriptor_valid(layer))
    {
        return Err(invalid());
    }
    for layer in layers {
        if layer["annotations"]["com.microsoft.dsc.resource.id"] != expected.0
            || layer["annotations"]["com.microsoft.dsc.resource.version"] != expected.1
            || layer["annotations"]["org.opencontainers.image.title"]
                .as_str()
                .is_none_or(str::is_empty)
        {
            return Err(invalid());
        }
    }
    if resource_layer["size"]
        .as_u64()
        .is_none_or(|size| size > LIMIT as u64)
    {
        return Err(t!("oci.limit").to_string());
    }
    let digest = resource_layer["digest"].as_str().ok_or_else(invalid)?;
    let mut bytes = Vec::new();
    let mut stream = client
        .pull_blob_stream(reference, digest)
        .await
        .map_err(transport_error)?;
    while let Some(chunk) = stream.try_next().await.map_err(transport_error)? {
        if bytes.len() + chunk.len() > LIMIT {
            return Err(t!("oci.limit").to_string());
        }
        bytes.extend_from_slice(&chunk);
    }
    verify(&bytes, resource_layer)?;
    let resource: Value = serde_json::from_slice(&bytes).map_err(|_| invalid())?;
    if resource["type"] != expected.0
        || resource["version"] != expected.1
        || resource.get("resources").is_some()
        || resource.get("extensions").is_some()
    {
        return Err(invalid());
    }
    Ok(resource)
}

pub async fn discover(location: &str, token: Option<&str>) -> Result<Vec<Value>, String> {
    #[cfg(not(windows))]
    TLS_PROVIDER.as_ref().map_err(Clone::clone)?;
    let base = registry_url(location)?;
    let authority = &base[url::Position::BeforeHost..url::Position::AfterPort];
    let client = Client::new(ClientConfig {
        protocol: if base.scheme() == "http" {
            ClientProtocol::Http
        } else {
            ClientProtocol::Https
        },
        read_timeout: Some(Duration::from_secs(30)),
        connect_timeout: Some(Duration::from_secs(10)),
        ..Default::default()
    });
    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(transport_error)?;
    let auth = token.map_or(RegistryAuth::Anonymous, |token| {
        RegistryAuth::Bearer(token.to_string())
    });
    let platform = host_platform()?;
    let mut resources = Vec::new();
    for repository in catalog(&http, &base, token).await? {
        let reference: Reference = format!("{authority}/{repository}:latest")
            .parse()
            .map_err(transport_error)?;
        let mut tags = BTreeSet::new();
        let mut last = None;
        loop {
            let page = client
                .list_tags(&reference, &auth, Some(100), last.as_deref())
                .await
                .map_err(transport_error)?;
            if page.tags.is_empty() {
                break;
            }
            let cursor = page.tags.last().cloned().ok_or_else(invalid)?;
            if last.as_ref() == Some(&cursor) || page.tags.iter().all(|tag| tags.contains(tag)) {
                return Err(t!("oci.pagination").to_string());
            }
            tags.extend(page.tags);
            last = Some(cursor);
        }
        for tag in tags {
            let reference: Reference = format!("{authority}/{repository}:{tag}")
                .parse()
                .map_err(transport_error)?;
            let bytes = manifest(&client, &reference, &auth).await?;
            let index: Value = serde_json::from_slice(&bytes).map_err(|_| invalid())?;
            if index["mediaType"] != INDEX || index["artifactType"] != ARTIFACT {
                continue;
            }
            let (name, version) = identity(&index)?;
            let slug = name.split_once('/').ok_or_else(invalid)?.1.to_lowercase();
            if tag != format!("{slug}-{version}") {
                return Err(invalid());
            }
            let Some(child) = select_child(&index, platform)? else {
                continue;
            };
            let child_ref: Reference = format!(
                "{authority}/{repository}@{}",
                child["digest"].as_str().ok_or_else(invalid)?
            )
            .parse()
            .map_err(transport_error)?;
            resources
                .push(resource_manifest(&client, &child_ref, &auth, child, (name, version)).await?);
        }
    }
    Ok(resources)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use serde_json::json;
    use std::{
        collections::BTreeMap,
        io::{BufRead, BufReader, Write},
        net::TcpListener,
        sync::{
            Arc, Mutex,
            atomic::{AtomicBool, Ordering},
        },
        thread,
    };

    fn descriptor(bytes: &[u8], media_type: &str) -> Value {
        json!({"mediaType": media_type, "digest": sha256_digest(bytes), "size": bytes.len()})
    }

    pub(crate) struct RegistryServer {
        pub(crate) url: String,
        requests: Arc<Mutex<Vec<String>>>,
        stop: Arc<AtomicBool>,
        worker: Option<thread::JoinHandle<()>>,
    }

    impl RegistryServer {
        pub(crate) fn new(
            routes: BTreeMap<String, (String, Vec<u8>)>,
            token: Option<&str>,
        ) -> Self {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let address = listener.local_addr().unwrap();
            listener.set_nonblocking(true).unwrap();
            let stop = Arc::new(AtomicBool::new(false));
            let requests = Arc::new(Mutex::new(Vec::new()));
            let worker_stop = stop.clone();
            let worker_requests = requests.clone();
            let token = token.map(str::to_string);
            let worker = thread::spawn(move || {
                while !worker_stop.load(Ordering::Relaxed) {
                    let Ok((mut socket, _)) = listener.accept() else {
                        thread::sleep(Duration::from_millis(5));
                        continue;
                    };
                    socket.set_nonblocking(false).unwrap();
                    socket
                        .set_read_timeout(Some(Duration::from_secs(5)))
                        .unwrap();
                    let mut reader = BufReader::new(socket.try_clone().unwrap());
                    let mut line = String::new();
                    reader.read_line(&mut line).unwrap();
                    let path = line.split_whitespace().nth(1).unwrap().to_string();
                    let mut authorized = token.is_none();
                    loop {
                        line.clear();
                        reader.read_line(&mut line).unwrap();
                        if line == "\r\n" || line.is_empty() {
                            break;
                        }
                        if let Some(token) = &token {
                            if line.trim() == format!("authorization: Bearer {token}") {
                                authorized = true;
                            }
                        }
                    }
                    worker_requests.lock().unwrap().push(path.clone());
                    let grant = routes
                        .get("/oauth2/token")
                        .filter(|_| path.starts_with("/oauth2/token?"));
                    let (status, headers, body) = if let Some((headers, body)) = grant {
                        ("200 OK", headers.clone(), body.clone())
                    } else if !authorized {
                        let challenge = if routes.contains_key("/oauth2/token") {
                            format!(
                                "WWW-Authenticate: Bearer realm=\"http://{address}/oauth2/token\",service=\"test\",scope=\"registry:catalog:*\"\r\n"
                            )
                        } else {
                            String::new()
                        };
                        ("401 Unauthorized", challenge, Vec::new())
                    } else if let Some((headers, body)) = routes.get(&path) {
                        ("200 OK", headers.clone(), body.clone())
                    } else if path == "/v2/" {
                        ("200 OK", String::new(), b"{}".to_vec())
                    } else {
                        ("404 Not Found", String::new(), b"{}".to_vec())
                    };
                    write!(socket, "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n{headers}\r\n", body.len()).unwrap();
                    socket.write_all(&body).unwrap();
                }
            });
            Self {
                url: format!("http://{address}"),
                requests,
                stop,
                worker: Some(worker),
            }
        }
    }

    impl Drop for RegistryServer {
        fn drop(&mut self) {
            self.stop.store(true, Ordering::Relaxed);
            let result = self.worker.take().unwrap().join();
            if !thread::panicking() {
                result.unwrap();
            }
        }
    }

    pub(crate) fn routes(corrupt: bool) -> BTreeMap<String, (String, Vec<u8>)> {
        let resource = serde_json::to_vec(&json!({
            "$schema":"https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.json",
            "type":"Test.Oci/Example", "version":"1.0.0",
            "get":{"executable":"not-downloaded"},
            "schema":{"embedded":{"type":"object"}}
        }))
        .unwrap();
        let mut layer = descriptor(&resource, "application/json");
        layer["annotations"] = json!({
            "com.microsoft.dsc.resource.id":"Test.Oci/Example",
            "com.microsoft.dsc.resource.version":"1.0.0",
            "org.opencontainers.image.title":"example.dsc.resource.json"
        });
        let mut zip = descriptor(b"not-downloaded", "application/zip");
        zip["annotations"] = json!({
            "com.microsoft.dsc.resource.id":"Test.Oci/Example",
            "com.microsoft.dsc.resource.version":"1.0.0",
            "org.opencontainers.image.title":"Test.Oci.zip"
        });
        let annotations = json!({
            "com.microsoft.dsc.resource.id":"Test.Oci/Example",
            "com.microsoft.dsc.version":"1.0.0",
            "com.microsoft.dsc.kind":"resource",
            "org.opencontainers.image.created":"2026-11-22T00:00:00Z"
        });
        let image = serde_json::to_vec(&json!({
            "schemaVersion":2,"mediaType":IMAGE,"artifactType":ARTIFACT,
            "config":descriptor(b"{}", "application/vnd.oci.empty.v1+json"),
            "annotations":annotations,"layers":[layer,zip]
        }))
        .unwrap();
        let (os, arch) = host_platform().unwrap();
        let mut child = descriptor(&image, IMAGE);
        child["platform"] = json!({"os":os,"architecture":arch});
        let index = json!({"schemaVersion":2,"mediaType":INDEX,"artifactType":ARTIFACT,
                        "annotations":annotations,"manifests":[child]});
        let prefix = "/v2/dsc/resource/test.oci";
        BTreeMap::from([
            (
                "/v2/_catalog?n=100".into(),
                (
                    "Link: </v2/_catalog?n=100&last=one>; rel=\"next\"\r\n".into(),
                    serde_json::to_vec(
                        &json!({"repositories":["dsc/resource/test.oci","unrelated"]}),
                    )
                    .unwrap(),
                ),
            ),
            (
                "/v2/_catalog?n=100&last=one".into(),
                (
                    String::new(),
                    serde_json::to_vec(&json!({"repositories":["dsc/resource/test.oci"]})).unwrap(),
                ),
            ),
            (
                format!("{prefix}/tags/list?n=100"),
                (
                    String::new(),
                    serde_json::to_vec(
                        &json!({"name":"dsc/resource/test.oci","tags":["example-1.0.0"]}),
                    )
                    .unwrap(),
                ),
            ),
            (
                format!("{prefix}/tags/list?n=100&last=example-1.0.0"),
                (
                    String::new(),
                    serde_json::to_vec(&json!({"name":"dsc/resource/test.oci","tags":[]})).unwrap(),
                ),
            ),
            (
                format!("{prefix}/manifests/example-1.0.0"),
                (
                    format!("Content-Type: {INDEX}\r\n"),
                    serde_json::to_vec(&index).unwrap(),
                ),
            ),
            (
                format!("{prefix}/manifests/{}", child["digest"].as_str().unwrap()),
                (format!("Content-Type: {IMAGE}\r\n"), image),
            ),
            (
                format!("{prefix}/blobs/{}", layer["digest"].as_str().unwrap()),
                (
                    "Content-Type: application/json\r\n".into(),
                    if corrupt { b"{}".to_vec() } else { resource },
                ),
            ),
        ])
    }

    #[tokio::test]
    async fn paginated_discovery_downloads_only_resource_manifest_for_both_auth_modes() {
        for token in [None, Some("test-token")] {
            let server = RegistryServer::new(routes(false), token);
            let resources = discover(&server.url, token).await.unwrap();
            assert_eq!(resources.len(), 1);
            assert_eq!(resources[0]["type"], "Test.Oci/Example");
            let requests = server.requests.lock().unwrap();
            assert_eq!(
                requests
                    .iter()
                    .filter(|path| path.contains("/blobs/"))
                    .count(),
                1
            );
            assert!(
                requests
                    .iter()
                    .any(|path| path.contains("_catalog?n=100&last=one"))
            );
        }
    }

    #[tokio::test]
    async fn anonymous_catalog_negotiates_and_reuses_token_for_pagination() {
        for key in ["token", "access_token"] {
            let mut routes = routes(false);
            routes.insert(
                "/oauth2/token".into(),
                (
                    String::new(),
                    serde_json::to_vec(&json!({key: "catalog-token"})).unwrap(),
                ),
            );
            let server = RegistryServer::new(routes, Some("catalog-token"));
            let http = reqwest::Client::new();
            let repositories = catalog(&http, &registry_url(&server.url).unwrap(), None)
                .await
                .unwrap();
            assert_eq!(
                repositories,
                BTreeSet::from(["dsc/resource/test.oci".into()])
            );
            let requests = server.requests.lock().unwrap();
            let grants: Vec<_> = requests
                .iter()
                .filter(|path| path.starts_with("/oauth2/token?"))
                .collect();
            assert_eq!(grants.len(), 1);
            let grant = Url::parse(&format!("{}{}", server.url, grants[0])).unwrap();
            let parameters: BTreeMap<_, _> = grant.query_pairs().collect();
            assert_eq!(parameters["service"], "test");
            assert_eq!(parameters["scope"], "registry:catalog:*");
        }
    }

    #[tokio::test]
    async fn anonymous_catalog_rejects_invalid_grant_without_echoing_body() {
        let mut routes = routes(false);
        routes.insert(
            "/oauth2/token".into(),
            (
                String::new(),
                br#"{"error":"confidential-response"}"#.to_vec(),
            ),
        );
        let server = RegistryServer::new(routes, Some("catalog-token"));
        let error = catalog(
            &reqwest::Client::new(),
            &registry_url(&server.url).unwrap(),
            None,
        )
        .await
        .unwrap_err();
        assert_eq!(error, t!("oci.token").to_string());
        assert!(!error.contains("confidential-response"));
    }

    #[tokio::test]
    async fn catalog_permissions_and_corrupt_content_are_errors() {
        let server = RegistryServer::new(routes(false), Some("required-token"));
        assert!(
            discover(&server.url, None)
                .await
                .unwrap_err()
                .contains("401")
        );
        let server = RegistryServer::new(routes(true), None);
        assert!(discover(&server.url, None).await.is_err());
    }

    #[test]
    fn rejects_duplicate_platforms_invalid_digests_and_wrong_blob_sizes() {
        let routes = routes(false);
        let mut index: Value =
            serde_json::from_slice(&routes["/v2/dsc/resource/test.oci/manifests/example-1.0.0"].1)
                .unwrap();
        let child = index["manifests"][0].clone();
        index["manifests"].as_array_mut().unwrap().push(child);
        assert!(select_child(&index, host_platform().unwrap()).is_err());
        assert!(verify(b"{}", &descriptor(b"other", "application/json")).is_err());
        assert!(!descriptor_valid(&json!({"digest":"sha256:bad","size":1})));
        assert!(registry_url("http://remote.example").is_err());
    }
}
