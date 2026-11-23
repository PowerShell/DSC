// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use futures_util::TryStreamExt;
use oci_client::{
    Client, Reference, RegistryOperation,
    client::{ClientConfig, ClientProtocol},
    secrets::RegistryAuth,
};
use reqwest::{
    Url,
    header::{ACCEPT, LINK, WWW_AUTHENTICATE},
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

async fn limited_bytes(mut response: reqwest::Response) -> Result<Vec<u8>, String> {
    if response
        .content_length()
        .is_some_and(|size| size > LIMIT as u64)
    {
        return Err(t!("oci.limit").to_string());
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(transport_error)? {
        if bytes.len() + chunk.len() > LIMIT {
            return Err(t!("oci.limit").to_string());
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}

async fn limited_json(response: reqwest::Response) -> Result<Value, String> {
    serde_json::from_slice(&limited_bytes(response).await?).map_err(|_| invalid())
}

async fn manifest(
    http: &reqwest::Client,
    base: &Url,
    reference: &Reference,
    token: Option<&str>,
) -> Result<Vec<u8>, String> {
    let mut url = base.clone();
    url.path_segments_mut()
        .map_err(|()| invalid())?
        .clear()
        .push("v2")
        .extend(reference.repository().split('/'))
        .push("manifests")
        .push(
            reference
                .digest()
                .or_else(|| reference.tag())
                .ok_or_else(invalid)?,
        );
    let mut request = http.get(url).header(ACCEPT, format!("{INDEX}, {IMAGE}"));
    if let Some(token) = token {
        request = request.bearer_auth(token);
    }
    let response = request.send().await.map_err(transport_error)?;
    if !response.status().is_success() {
        return Err(transport_error(response.status()));
    }
    limited_bytes(response).await
}

async fn resource_manifest(
    client: &Client,
    http: &reqwest::Client,
    base: &Url,
    reference: &Reference,
    token: Option<&str>,
    child: &Value,
    expected: (&str, &str),
) -> Result<Value, String> {
    let child_bytes = manifest(http, base, reference, token).await?;
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

pub async fn discover(
    location: &str,
    token: Option<&str>,
    mut warn: impl FnMut(&str, &str, &str) -> Result<(), String>,
) -> Result<Vec<Value>, String> {
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
        let repository_token = client
            .auth(&reference, &auth, RegistryOperation::Pull)
            .await
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
            let result = async {
                let reference: Reference = format!("{authority}/{repository}:{tag}")
                    .parse()
                    .map_err(transport_error)?;
                let bytes = manifest(&http, &base, &reference, repository_token.as_deref()).await?;
                let index: Value = serde_json::from_slice(&bytes).map_err(|_| invalid())?;
                if index["mediaType"] != INDEX || index["artifactType"] != ARTIFACT {
                    return Ok(None);
                }
                let (name, version) = identity(&index)?;
                let slug = name.split_once('/').ok_or_else(invalid)?.1.to_lowercase();
                if tag != format!("{slug}-{version}") {
                    return Err(invalid());
                }
                let Some(child) = select_child(&index, platform)? else {
                    return Ok(None);
                };
                let child_ref: Reference = format!(
                    "{authority}/{repository}@{}",
                    child["digest"].as_str().ok_or_else(invalid)?
                )
                .parse()
                .map_err(transport_error)?;
                resource_manifest(
                    &client,
                    &http,
                    &base,
                    &child_ref,
                    repository_token.as_deref(),
                    child,
                    (name, version),
                )
                .await
                .map(Some)
            }
            .await;
            match result {
                Ok(Some(resource)) => resources.push(resource),
                Ok(None) => {}
                Err(error) => warn(&repository, &tag, &error)?,
            }
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
                    if let Err(error) = socket.write_all(&body) {
                        assert!(matches!(
                            error.kind(),
                            std::io::ErrorKind::BrokenPipe
                                | std::io::ErrorKind::ConnectionReset
                                | std::io::ErrorKind::ConnectionAborted
                        ));
                    }
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
        version_routes(corrupt, "1.0.0")
    }

    fn version_routes(corrupt: bool, version: &str) -> BTreeMap<String, (String, Vec<u8>)> {
        let resource = serde_json::to_vec(&json!({
            "$schema":"https://aka.ms/dsc/schemas/v3/bundled/resource/manifest.json",
            "type":"Test.Oci/Example", "version":version,
            "get":{"executable":"not-downloaded"},
            "schema":{"embedded":{"type":"object"}}
        }))
        .unwrap();
        let mut layer = descriptor(&resource, "application/json");
        layer["annotations"] = json!({
            "com.microsoft.dsc.resource.id":"Test.Oci/Example",
            "com.microsoft.dsc.resource.version":version,
            "org.opencontainers.image.title":"example.dsc.resource.json"
        });
        let mut zip = descriptor(b"not-downloaded", "application/zip");
        zip["annotations"] = json!({
            "com.microsoft.dsc.resource.id":"Test.Oci/Example",
            "com.microsoft.dsc.resource.version":version,
            "org.opencontainers.image.title":"Test.Oci.zip"
        });
        let annotations = json!({
            "com.microsoft.dsc.resource.id":"Test.Oci/Example",
            "com.microsoft.dsc.version":version,
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
                        &json!({"name":"dsc/resource/test.oci","tags":[format!("example-{version}")]}),
                    )
                    .unwrap(),
                ),
            ),
            (
                format!("{prefix}/tags/list?n=100&last=example-{version}"),
                (
                    String::new(),
                    serde_json::to_vec(&json!({"name":"dsc/resource/test.oci","tags":[]})).unwrap(),
                ),
            ),
            (
                format!("{prefix}/manifests/example-{version}"),
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
            let resources = discover(&server.url, token, |_, _, error| panic!("{error}"))
                .await
                .unwrap();
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
    async fn catalog_permissions_are_errors_and_corrupt_content_is_warned() {
        let server = RegistryServer::new(routes(false), Some("required-token"));
        assert!(
            discover(&server.url, None, |_, _, error| panic!("{error}"))
                .await
                .unwrap_err()
                .contains("401")
        );
        let server = RegistryServer::new(routes(true), None);
        let mut warnings = Vec::new();
        assert!(
            discover(&server.url, None, |_, _, error| {
                warnings.push(error.to_string());
                Ok(())
            })
            .await
            .unwrap()
            .is_empty()
        );
        assert_eq!(warnings, [t!("oci.transport").to_string()]);
    }

    #[tokio::test]
    async fn malformed_metadata_retains_healthy_versions_before_and_after_failures() {
        let prefix = "/v2/dsc/resource/test.oci";
        let mut all_routes = BTreeMap::new();
        for version in 1..=6 {
            let version = format!("{version}.0.0");
            let mut routes = version_routes(version == "5.0.0", &version);
            let index_path = format!("{prefix}/manifests/example-{version}");
            let mut index: Value = serde_json::from_slice(&routes[&index_path].1).unwrap();
            match version.as_str() {
                "2.0.0" => routes.get_mut(&index_path).unwrap().1 = b"confidential-body".to_vec(),
                "3.0.0" => {
                    index["manifests"][0]["platform"]["architecture"] = json!("invalid");
                    routes.get_mut(&index_path).unwrap().1 = serde_json::to_vec(&index).unwrap();
                }
                "4.0.0" => {
                    let child_path = format!(
                        "{prefix}/manifests/{}",
                        index["manifests"][0]["digest"].as_str().unwrap()
                    );
                    routes.get_mut(&child_path).unwrap().1 = b"confidential-body".to_vec();
                }
                _ => {}
            }
            all_routes.extend(routes);
        }
        all_routes.get_mut(&format!("{prefix}/tags/list?n=100")).unwrap().1 =
            serde_json::to_vec(&json!({"name": "dsc/resource/test.oci", "tags": (1..=6).map(|version| format!("example-{version}.0.0")).collect::<Vec<_>>()})).unwrap();
        let server = RegistryServer::new(all_routes, None);
        let mut warnings = Vec::new();
        let resources = discover(&server.url, None, |repository, tag, error| {
            warnings.push((repository.to_string(), tag.to_string(), error.to_string()));
            Ok(())
        })
        .await
        .unwrap();
        assert_eq!(
            resources
                .iter()
                .map(|resource| resource["version"].as_str().unwrap())
                .collect::<Vec<_>>(),
            ["1.0.0", "6.0.0"]
        );
        assert_eq!(warnings.len(), 4);
        for (warning, version) in warnings.iter().zip(2..=5) {
            assert_eq!(warning.0, "dsc/resource/test.oci");
            assert_eq!(warning.1, format!("example-{version}.0.0"));
            assert!(!warning.2.contains("confidential-body"));
        }
    }

    #[tokio::test]
    async fn manifest_accepts_exact_limit_and_rejects_declared_oversize() {
        for size in [LIMIT, LIMIT + 1] {
            let path = "/v2/dsc/resource/test.oci/manifests/example-1.0.0";
            let server = RegistryServer::new(
                BTreeMap::from([(
                    path.into(),
                    (format!("Content-Type: {INDEX}\r\n"), vec![b' '; size]),
                )]),
                Some("test-token"),
            );
            let base = registry_url(&server.url).unwrap();
            let authority = &base[url::Position::BeforeHost..url::Position::AfterPort];
            let reference: Reference = format!("{authority}/dsc/resource/test.oci:example-1.0.0")
                .parse()
                .unwrap();
            let result = manifest(
                &reqwest::Client::new(),
                &base,
                &reference,
                Some("test-token"),
            )
            .await;
            if size == LIMIT {
                assert_eq!(result.unwrap().len(), LIMIT);
            } else {
                assert_eq!(result.unwrap_err(), t!("oci.limit").to_string());
            }
        }
    }

    #[tokio::test]
    async fn tag_and_digest_manifest_streams_stop_at_limit_before_eof() {
        for selector in ["example-1.0.0".to_string(), sha256_digest(b"child")] {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let address = listener.local_addr().unwrap();
            let (release, wait) = std::sync::mpsc::channel();
            let worker = thread::spawn(move || {
                let (mut socket, _) = listener.accept().unwrap();
                socket
                    .set_write_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                let mut reader = BufReader::new(socket.try_clone().unwrap());
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                let path = line.split_whitespace().nth(1).unwrap().to_string();
                loop {
                    line.clear();
                    reader.read_line(&mut line).unwrap();
                    if line == "\r\n" {
                        break;
                    }
                }
                socket.write_all(b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n").unwrap();
                for chunk in [vec![b' '; LIMIT], vec![b' '; 1]] {
                    if write!(socket, "{:x}\r\n", chunk.len())
                        .and_then(|()| socket.write_all(&chunk))
                        .and_then(|()| socket.write_all(b"\r\n"))
                        .is_err()
                    {
                        break;
                    }
                }
                // Keep EOF withheld until the bounded reader has returned.
                wait.recv_timeout(Duration::from_secs(10)).unwrap();
                path
            });
            let base = registry_url(&format!("http://{address}")).unwrap();
            let separator = if selector.starts_with("sha256:") {
                "@"
            } else {
                ":"
            };
            let reference: Reference =
                format!("{address}/dsc/resource/test.oci{separator}{selector}")
                    .parse()
                    .unwrap();
            let result = tokio::time::timeout(
                Duration::from_secs(5),
                manifest(&reqwest::Client::new(), &base, &reference, None),
            )
            .await;
            release.send(()).unwrap();
            assert_eq!(
                worker.join().unwrap(),
                format!("/v2/dsc/resource/test.oci/manifests/{selector}")
            );
            assert_eq!(result.unwrap().unwrap_err(), t!("oci.limit").to_string());
        }
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
